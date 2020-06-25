Simulator for running [`::constance`] on a hosted environment

# Usage

```rust
#![feature(const_loop)]
#![feature(const_fn)]
#![feature(const_if_match)]
#![feature(const_mut_refs)]

// Require `unsafe` even in `unsafe fn` - highly recommended
#![feature(unsafe_block_in_unsafe_fn)]
#![deny(unsafe_op_in_unsafe_fn)]

use constance::kernel::Task;

// Use the simulator port
constance_port_std::use_port!(unsafe struct System);

const COTTAGE: () = constance::build!(System, configure_app => ());

constance::configure! {
    const fn configure_app(_: &mut CfgBuilder<System>) -> () {
        new! { Task<_>, start = task_body, priority = 1, active = true };
    }
}

fn task_body(_: usize) {
    // The simulator initializes `env_logger` automatically
    log::warn!("yay");
}
```

# Interrupts

This port fully supports [the standard interrupt handling framework].

 - The full range of priority values is available. The default priority is `0`.
 - The simulated hardware exposes `1024` (= [`NUM_INTERRUPT_LINES`]) interrupt
   lines.
 - Smaller priority values are prioritized.
 - Negative priority values are considered unmanaged.

[the standard interrupt handling framework]: ::constance#interrupt-handling-framework
[`NUM_INTERRUPT_LINES`]: crate::NUM_INTERRUPT_LINES

## Implementation

All interrupt handlers execute in the main thread. Whenever an interrupt is pended or enabled, preemption checking code will run, and under the right condition, will yield the control to the dispatcher.

The dispatcher loop handles top-level interrupts and calls the interrupt handlers directly.

In an interrupt handler, the preemption checking code handles nested interrupts and calls their interrupt handlers.

**To be implemented:** True asynchronous interrupts aren't supported yet.