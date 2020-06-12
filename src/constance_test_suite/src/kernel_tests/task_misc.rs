//! Validates error codes returned by task manipulation methods. Also, checks
//! miscellaneous properties of `Task`.
use constance::{kernel::Task, prelude::*};
use core::num::NonZeroUsize;
use wyhash::WyHash;

use super::Driver;

pub struct App<System> {
    task1: Task<System>,
    task2: Task<System>,
}

impl<System: Kernel> App<System> {
    constance::configure! {
        pub fn new<D: Driver<Self>>(_: CfgBuilder<System>) -> Self {
            let task1 = new_task! { start = task1_body::<System, D>, priority = 2, active = true };
            let task2 = new_task! { start = task2_body::<System, D>, priority = 1 };

            App { task1, task2 }
        }
    }
}

fn task1_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    // `PartialEq`
    let app = D::app();
    assert_ne!(app.task1, app.task2);
    assert_eq!(app.task1, app.task1);
    assert_eq!(app.task2, app.task2);

    // `Hash`
    let hash = |x: Task<System>| {
        use core::hash::{Hash, Hasher};
        let mut hasher = WyHash::with_seed(42);
        x.hash(&mut hasher);
        hasher.finish()
    };
    assert_eq!(hash(app.task1), hash(app.task1));
    assert_eq!(hash(app.task2), hash(app.task2));

    // Invalid task ID
    let bad_task: Task<System> = unsafe { Task::from_id(NonZeroUsize::new(42).unwrap()) };
    assert_eq!(
        bad_task.activate(),
        Err(constance::kernel::ActivateTaskError::BadId)
    );

    // The task is already active
    assert_eq!(
        app.task1.activate(),
        Err(constance::kernel::ActivateTaskError::QueueOverflow)
    );

    D::success();
}

fn task2_body<System: Kernel, D: Driver<App<System>>>(_: usize) {
    unreachable!();
}