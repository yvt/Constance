//! Tasks
use core::{cell::UnsafeCell, fmt, marker::PhantomData, sync::atomic::Ordering};
use num_traits::ToPrimitive;

use super::{hunk::Hunk, utils, ActivateTaskError, ExitTaskError, Id, Kernel, KernelCfg1, Port};
use crate::utils::{
    intrusive_list::{CellLike, Ident, ListAccessorCell, Static, StaticLink, StaticListHead},
    Init, PrioBitmap, RawCell,
};

/// Represents a single task in a system.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Task<System>(Id, PhantomData<System>);

impl<System> fmt::Debug for Task<System> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Task").field(&self.0).finish()
    }
}

impl<System> Task<System> {
    /// Construct a `Task` from `Id`.
    ///
    /// # Safety
    ///
    /// The kernel can handle invalid IDs without a problem. However, the
    /// constructed `Task` may point to an object that is not intended to be
    /// manipulated except by its creator. This is usually prevented by making
    /// `Task` an opaque handle, but this safeguard can be circumvented by
    /// this method.
    pub const unsafe fn from_id(id: Id) -> Self {
        Self(id, PhantomData)
    }
}

impl<System: Kernel> Task<System> {
    /// Get the raw `Id` value representing this task.
    pub const fn id(self) -> Id {
        self.0
    }

    /// Start the execution of the task.
    pub fn activate(self) -> Result<(), ActivateTaskError> {
        let lock = utils::lock_cpu::<System>()?;
        let task_cb = System::get_task_cb(self.0.get() - 1).ok_or(ActivateTaskError::BadId)?;
        activate(lock, task_cb)
    }
}

/// [`Hunk`] for a task stack.
#[repr(transparent)]
pub struct StackHunk<System>(Hunk<System, [UnsafeCell<u8>]>);

// Safety: Safe code can't access the contents. Also, the port is responsible
// for making sure `StackHunk` is used in the correct way.
unsafe impl<System> Sync for StackHunk<System> {}

impl<System: Kernel> fmt::Debug for StackHunk<System> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StackHunk").field(&self.0.as_ptr()).finish()
    }
}

// TODO: Preferably `StackHunk` shouldn't be `Clone` as it strengthens the
//       safety obligation of `StackHunk::from_hunk`.
impl<System> Clone for StackHunk<System> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<System> Copy for StackHunk<System> {}

// TODO: Should we allow zero-sized `StackHunk`?
impl<System> Init for StackHunk<System> {
    const INIT: Self = Self(Init::INIT);
}

impl<System> StackHunk<System> {
    /// Construct a `StackHunk` from `Hunk`.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure the region represented by
    /// `hunk` is solely used for a single task's stack.
    ///
    /// Also, `hunk` must be properly aligned for a stack region.
    pub const unsafe fn from_hunk(hunk: Hunk<System, [UnsafeCell<u8>]>) -> Self {
        Self(hunk)
    }

    /// Get the inner `Hunk`, consuming `self`.
    pub fn into_inner(self) -> Hunk<System, [UnsafeCell<u8>]> {
        self.0
    }
}

impl<System: Kernel> StackHunk<System> {
    /// Get a raw pointer to the hunk's contents.
    pub fn as_ptr(&self) -> *mut [u8] {
        &*self.0 as *const _ as _
    }
}

/// *Task control block* - the state data of a task.
#[repr(C)]
pub struct TaskCb<
    System: Port,
    PortTaskState: 'static = <System as Port>::PortTaskState,
    TaskPriority: 'static = <System as KernelCfg1>::TaskPriority,
> {
    /// Get a reference to `PortTaskState` in the task control block.
    ///
    /// This is guaranteed to be placed at the beginning of the struct so that
    /// assembler code can refer to this easily.
    pub port_task_state: PortTaskState,

    /// The static properties of the task.
    pub attr: &'static TaskAttr<System>,

    pub priority: TaskPriority,

    pub(super) st: utils::CpuLockCell<System, TaskSt>,

    /// Allows `TaskCb` to participate in one of linked lists.
    ///
    ///  - In a `Runnable` state, this forms the linked list headed by
    ///    [`State::task_ready_queue`].
    ///
    /// [`State::task_ready_queue`]: crate::kernel::State::task_ready_queue
    pub(super) link: utils::CpuLockCell<System, Option<StaticLink<Self>>>,

    pub(super) _force_int_mut: RawCell<()>,
}

impl<System: Port, PortTaskState: Init + 'static, TaskPriority: Init + 'static> Init
    for TaskCb<System, PortTaskState, TaskPriority>
{
    const INIT: Self = Self {
        port_task_state: Init::INIT,
        attr: &TaskAttr::INIT,
        priority: Init::INIT,
        st: Init::INIT,
        link: Init::INIT,
        _force_int_mut: RawCell::new(()),
    };
}

impl<System: Kernel, PortTaskState: fmt::Debug + 'static, TaskPriority: fmt::Debug + 'static>
    fmt::Debug for TaskCb<System, PortTaskState, TaskPriority>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskCb")
            .field("port_task_state", &self.port_task_state)
            .field("attr", self.attr)
            .field("priority", &self.priority)
            .finish()
    }
}

/// The static properties of a task.
pub struct TaskAttr<System> {
    /// The entry point of the task.
    ///
    /// # Safety
    ///
    /// This is only meant to be used by a kernel port, as a task entry point,
    /// not by user code. Using this in other ways may cause an undefined
    /// behavior.
    pub entry_point: unsafe fn(usize),

    /// The parameter supplied for `entry_point`.
    pub entry_param: usize,

    // FIXME: Ideally, `stack` should directly point to the stack region. But
    //        this is blocked by <https://github.com/rust-lang/const-eval/issues/11>
    /// The hunk representing the stack region for the task.
    pub stack: StackHunk<System>,
}

impl<System> Init for TaskAttr<System> {
    const INIT: Self = Self {
        entry_point: |_| {},
        entry_param: 0,
        stack: StackHunk::INIT,
    };
}

impl<System: Kernel> fmt::Debug for TaskAttr<System> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskAttr")
            .field("entry_point", &self.entry_point)
            .field("entry_param", &self.entry_param)
            .field("stack", &self.stack)
            .finish()
    }
}

/// Task state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskSt {
    /// The task is in the Dormant state.
    Dormant,

    /// The task is in the Runnable state.
    // TODO: Rename all mentions of "Runnable" to "Ready"
    Runnable,

    /// The task is in the Running state.
    Running,

    /// The task should be activated at startup. This will transition into
    /// `Runnable` or `Running` before the first task is scheduled.
    PendingActivation,
}

impl Init for TaskSt {
    const INIT: Self = Self::Dormant;
}

/// Implements [`Kernel::exit_task`].
pub(super) unsafe fn exit_current_task<System: Kernel>() -> Result<!, ExitTaskError> {
    // TODO: Deny interrupt context

    // If CPU Lock is inactive, activate it.
    // TODO: If `is_cpu_lock_active() == true`, assert that it was an
    //       application that has the lock. It's illegal for it to be a
    //       kernel-owned CPU Lock.
    let mut lock = unsafe {
        if !System::is_cpu_lock_active() {
            System::enter_cpu_lock();
        }
        utils::assume_cpu_lock::<System>()
    };

    // Transition the current task to Dormant
    let running_task = System::state().running_task().unwrap();
    assert_eq!(*running_task.st.read(&*lock), TaskSt::Running);
    running_task.st.replace(&mut *lock, TaskSt::Dormant);

    // Erase `running_task`
    System::state().running_task.store(None, Ordering::Relaxed);

    core::mem::forget(lock);

    // Safety: (1) The user of `exit_task` acknowledges that all preexisting
    // data on the task stack will be invalidated and has promised that this
    // will not cause any UBs. (2) CPU Lock active
    unsafe {
        System::exit_and_dispatch(running_task);
    }
}

/// Initialize a task at boot time.
pub(super) fn init_task<System: Kernel>(
    lock: &mut utils::CpuLockGuard<System>,
    task_cb: &'static TaskCb<System>,
) {
    if let TaskSt::PendingActivation = task_cb.st.read(&**lock) {
        // `PendingActivation` is equivalent to `Dormant` but serves as a marker
        // indicating tasks that should be activated by `init_task`.

        // Safety: CPU Lock active, the task is (essentially) in a Dormant state
        unsafe { System::initialize_task_state(task_cb) };

        // Safety: The previous state is PendingActivation (which is equivalent
        // to Dormant) and we just initialized the task state, so this is safe
        unsafe { make_runnable(lock, task_cb) };
    }
}

/// Get a `ListAccessorCell` used to access a task ready queue.
macro_rules! list_accessor {
    (<$sys:ty>::state().task_ready_queue[$i:expr], $key:expr) => {
        ListAccessorCell::new(
            TaskReadyQueueHeadAccessor($i, &<$sys>::state().task_ready_queue),
            &Static,
            |task_cb: &TaskCb<$sys>| &task_cb.link,
            $key,
        )
    };
}

/// A helper type for `list_accessor`, implementing
/// `CellLike<StaticListHead<TaskCb<System>>>`.
struct TaskReadyQueueHeadAccessor<System: Port, TaskReadyQueue: 'static>(
    usize,
    &'static utils::CpuLockCell<System, TaskReadyQueue>,
);

impl<'a, System, TaskReadyQueue> CellLike<&'a mut utils::CpuLockGuard<System>>
    for TaskReadyQueueHeadAccessor<System, TaskReadyQueue>
where
    System: Kernel,
    TaskReadyQueue: core::borrow::BorrowMut<[StaticListHead<TaskCb<System>>]> + 'static,
{
    type Target = StaticListHead<TaskCb<System>>;

    fn get(&self, key: &&'a mut utils::CpuLockGuard<System>) -> Self::Target {
        self.1.read(&***key).borrow()[self.0]
    }
    fn set(&self, key: &mut &'a mut utils::CpuLockGuard<System>, value: Self::Target) {
        self.1.write(&mut ***key).borrow_mut()[self.0] = value;
    }
}

/// Implements `Task::activate`.
fn activate<System: Kernel>(
    mut lock: utils::CpuLockGuard<System>,
    task_cb: &'static TaskCb<System>,
) -> Result<(), ActivateTaskError> {
    if *task_cb.st.read(&*lock) != TaskSt::Dormant {
        return Err(ActivateTaskError::QueueOverflow);
    }

    // Safety: CPU Lock active, the task is in a Dormant state
    unsafe { System::initialize_task_state(task_cb) };

    // Safety: The previous state is Dormant, and we just initialized the task
    // state, so this is safe
    unsafe { make_runnable(&mut lock, task_cb) };

    // If `task_cb` has a higher priority, perform a context switch.
    unlock_cpu_and_check_preemption(lock);

    Ok(())
}

/// Transition the task into the Runnable state. This function doesn't do any
/// proper cleanup for a previous state. If the previous state is `Dormant`, the
/// caller must initialize the task state first by calling
/// `initialize_task_state`.
unsafe fn make_runnable<System: Kernel>(
    // FIXME: It's inefficient to pass `&mut CpuLockGuard` because it's pointer-sized
    lock: &mut utils::CpuLockGuard<System>,
    task_cb: &'static TaskCb<System>,
) {
    // Make the task runnable
    task_cb.st.replace(&mut **lock, TaskSt::Runnable);

    // Insert the task to a ready queue
    let pri = task_cb.priority.to_usize().unwrap();
    list_accessor!(<System>::state().task_ready_queue[pri], lock).push_back(Ident(task_cb));

    // Update `task_ready_bitmap` accordingly
    <System>::state()
        .task_ready_bitmap
        .write(&mut **lock)
        .set(pri);
}

/// Relinquish CPU Lock. After that, if there's a higher-priority task than
/// `running_task`, call `Port::yield_cpu`.
///
/// System services that transition a task into a Runnable state should call
/// this before returning to the caller.
fn unlock_cpu_and_check_preemption<System: Kernel>(lock: utils::CpuLockGuard<System>) {
    let prev_task_priority = if let Some(running_task) = System::state().running_task() {
        running_task.priority.to_usize().unwrap()
    } else {
        usize::max_value()
    };

    // The priority of the next task to run
    let next_task_priority = System::state()
        .task_ready_bitmap
        .read(&*lock)
        .find_set()
        .unwrap_or(usize::max_value());

    // Relinquish CPU Lock
    drop(lock);

    if next_task_priority < prev_task_priority {
        // Safety: CPU Lock inactive
        unsafe { System::yield_cpu() };
    }
}

/// Implements `PortToKernel::choose_running_task`.
pub(super) fn choose_next_running_task<System: Kernel>(lock: &mut utils::CpuLockGuard<System>) {
    // The priority of `running_task`
    let prev_running_task = System::state().running_task();
    let prev_task_priority = if let Some(running_task) = prev_running_task {
        running_task.priority.to_usize().unwrap()
    } else {
        usize::max_value()
    };

    // The priority of the next task to run
    let next_task_priority = System::state()
        .task_ready_bitmap
        .read(&**lock)
        .find_set()
        .unwrap_or(usize::max_value());

    // Return if there's no task willing to take over the current one.
    if prev_task_priority <= next_task_priority {
        return;
    }

    // Find the next task to run
    let next_running_task = if next_task_priority < System::NUM_TASK_PRIORITY_LEVELS {
        // Take the first task in the ready queue for `next_task_priority`
        let mut accessor =
            list_accessor!(<System>::state().task_ready_queue[next_task_priority], lock);
        let task = accessor.pop_front().unwrap().0;

        // Update `task_ready_bitmap` accordingly
        if accessor.is_empty() {
            <System>::state()
                .task_ready_bitmap
                .write(&mut **lock)
                .clear(next_task_priority);
        }

        // Transition `next_running_task` into the Running state
        task.st.replace(&mut **lock, TaskSt::Running);

        Some(task)
    } else {
        None
    };

    // Put `prev_running_task` (which is currently in the Running state) into the
    // Runnable state
    if let Some(running_task) = prev_running_task {
        assert!(*running_task.st.read(&**lock) == TaskSt::Running);

        // Safety: The previous state is Running, so this is safe
        unsafe { make_runnable(lock, running_task) };
    }

    System::state()
        .running_task
        .store(next_running_task, Ordering::Relaxed);
}
