//! This example enables exception tracing and ...

#![no_std]
#![no_main]


use panic_halt as _; // panic handler
use rtic::app;

#[app(device = stm32f4::stm32f401, peripherals = true)]
mod app {
    use stm32f4::stm32f401::Interrupt;
    use trace_examples;
    use cortex_m::asm;

    #[init]
    fn init(mut ctx: init::Context)  -> (init::LateResources, init::Monotonics) {
        trace_examples::enable_tracing(&mut ctx.core, &mut ctx.device);
        rtic::pend(Interrupt::EXTI0);

        (init::LateResources {}, init::Monotonics())
    }

    // taben after `init` returns
    #[task(binds = EXTI0, priority = 1)]
    fn blah(_: blah::Context) {
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
}
