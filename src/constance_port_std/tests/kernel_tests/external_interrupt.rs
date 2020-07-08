//! Pends an interrupt from an external thread.
use constance::{
    kernel::{Hunk, InterruptHandler, InterruptLine, Task},
    prelude::*,
};
use constance_test_suite::kernel_tests::Driver;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::{sleep, spawn},
    time::Duration,
};

use constance_port_std::PortInstance;

pub struct App<System> {
    int: Option<InterruptLine<System>>,
    done: Hunk<System, AtomicBool>,
}

impl<System: PortInstance> App<System> {
    constance::configure! {
        pub const fn new<D: Driver<Self>>(_: &mut CfgBuilder<System>) -> Self {
            new! { Task<_>, start = task_body1::<System, D>, priority = 1, active = true };

            let int = if let [int_line, ..] = *D::INTERRUPT_LINES {
                new! { InterruptHandler<_>,
                    line = int_line, start = isr::<System, D>};

                Some(new! { InterruptLine<_>,
                    line = int_line, priority = D::INTERRUPT_PRIORITY_LOW , enabled = true })
            } else {
                None
            };

            let done = new! { Hunk<_, AtomicBool> };

            App { int, done }
        }
    }
}

fn task_body1<System: PortInstance, D: Driver<App<System>>>(_: usize) {
    let int = if let Some(int) = D::app().int {
        int
    } else {
        log::warn!("No interrupt lines defined, skipping the test");
        D::success();
        return;
    };

    // Spawn a host thread
    log::debug!("spawning an external thread");
    spawn(move || {
        sleep(Duration::from_millis(100));
        log::debug!("pending {:?}", int);
        constance_port_std::pend_interrupt_line::<System>(int.num()).unwrap();
    });

    log::debug!("waiting for `done` to be set...");
    while !D::app().done.load(Ordering::Relaxed) {}
    log::debug!("success!");

    D::success();
}

fn isr<System: Kernel, D: Driver<App<System>>>(_: usize) {
    D::app().done.store(true, Ordering::Relaxed);
}
