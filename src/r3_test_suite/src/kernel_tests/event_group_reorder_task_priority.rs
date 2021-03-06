//! Sets an event group, waking up multiple tasks in a task priority order.
//! Changes the tasks' priorities while they are in a wait queue, triggering a
//! reorder.
//!
//! 1. (`seq`: 0 → 1) `task0` activates `task[1-4]` in a particular order.
//! 2. (`seq`: 1 → 5) `task[1-4]` start waiting for a event bit to be set.
//! 3. (`seq`: 5 → 9) `task0` sets the event bit for four times. `task[1-4]`
//!    should be unblocked in a task priority order.
//!
use r3::{
    hunk::Hunk,
    kernel::{cfg::CfgBuilder, EventGroup, EventGroupWaitFlags, QueueOrder, Task},
    prelude::*,
};

use super::Driver;
use crate::utils::SeqTracker;

pub struct App<System> {
    eg: EventGroup<System>,
    task1: Task<System>,
    task2: Task<System>,
    task3: Task<System>,
    task4: Task<System>,
    seq: Hunk<System, SeqTracker>,
}

impl<System: Kernel> App<System> {
    pub const fn new<D: Driver<Self>>(b: &mut CfgBuilder<System>) -> Self {
        b.num_task_priority_levels(5);

        Task::build()
            .start(task0_body::<System, D>)
            .priority(4)
            .active(true)
            .finish(b);
        let task1 = Task::build()
            .start(task1_body::<System, D>)
            .priority(1)
            .finish(b);
        let task2 = Task::build()
            .start(task2_body::<System, D>)
            .priority(2)
            .finish(b);
        let task3 = Task::build()
            .start(task3_body::<System, D>)
            .priority(1)
            .finish(b);
        let task4 = Task::build()
            .start(task4_body::<System, D>)
            .priority(2)
            .finish(b);

        let eg = EventGroup::build()
            .queue_order(QueueOrder::TaskPriority)
            .finish(b);
        let seq = Hunk::<_, SeqTracker>::build().finish(b);

        App {
            eg,
            task1,
            task2,
            task3,
            task4,
            seq,
        }
    }
}

fn task0_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().seq.expect_and_replace(0, 1);

    D::app().task3.activate().unwrap(); // [3]
    D::app().task1.activate().unwrap(); // [3, 1]
    D::app().task2.activate().unwrap(); // [3, 1, 2]
    D::app().task4.activate().unwrap(); // [3, 1, 2, 4]

    D::app().task1.set_priority(1).unwrap(); // [3, 1, 2, 4] (no-op)
    D::app().task3.set_priority(2).unwrap(); // [1, 2, 4, 3]
    D::app().task4.set_priority(3).unwrap(); // [1, 2, 3, 4]

    D::app().eg.set(0b1).unwrap(); // unblocks `task1`
    D::app().eg.set(0b1).unwrap(); // unblocks `task2`
    D::app().eg.set(0b1).unwrap(); // unblocks `task3`
    D::app().eg.set(0b1).unwrap(); // unblocks `task4`

    D::app().seq.expect_and_replace(9, 10);
    D::success();
}

fn task1_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().seq.expect_and_replace(2, 3);

    D::app().eg.wait(0b1, EventGroupWaitFlags::CLEAR).unwrap(); // start waiting, switching to `task0`

    D::app().seq.expect_and_replace(5, 6);
    // return the control to `task0`
}

fn task2_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().seq.expect_and_replace(3, 4);

    D::app().eg.wait(0b1, EventGroupWaitFlags::CLEAR).unwrap(); // start waiting, switching to `task0`

    D::app().seq.expect_and_replace(6, 7);
    // return the control to `task0`
}

fn task3_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().seq.expect_and_replace(1, 2);

    D::app().eg.wait(0b1, EventGroupWaitFlags::CLEAR).unwrap(); // start waiting, switching to `task0`

    D::app().seq.expect_and_replace(7, 8);
    // return the control to `task0`
}

fn task4_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().seq.expect_and_replace(4, 5);

    D::app().eg.wait(0b1, EventGroupWaitFlags::CLEAR).unwrap(); // start waiting, switching to `task0`

    D::app().seq.expect_and_replace(8, 9);
    // return the control to `task0`
}
