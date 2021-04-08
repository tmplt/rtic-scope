//! This example enables exception tracing and ...

#![no_std]
#![no_main]

use cortex_m::asm;
use panic_halt as _; // panic handler
use rtic::app;
use stm32f4::{self, stm32f401::Interrupt};
use trace_examples;

#[app(device = stm32f4::stm32f401, peripherals = true)]
const APP: () = {
    #[init]
    fn init(mut ctx: init::Context) {
        trace_examples::enable_tracing(&mut ctx.core, &mut ctx.device);

        rtic::pend(Interrupt::EXTI0);
    }

    // taben after `init` returns
    #[task(binds = EXTI0, priority = 1)]
    fn exti0(_: exti0::Context) {
        loop {
            rtic::pend(Interrupt::EXTI2);

            // wait until all ITM packets are flushed
            asm::delay(256);
        }
    }

    #[task(binds = EXTI1, priority = 2)]
    fn exti1(_: exti1::Context) {
        asm::delay(256);
    }

    #[task(binds = EXTI2, priority = 3)]
    fn exti2(_: exti2::Context) {
        // NOTE: EXTI1 has lower priority
        rtic::pend(Interrupt::EXTI1);

        asm::delay(512);
    }
};
