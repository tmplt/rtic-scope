//! This example enables exception tracing and ...

#![no_std]
#![no_main]

use panic_halt as _;            // panic handler
use stm32f4::stm32f401;
use rtic::app;
use cortex_m::asm;
use stm32f401::Interrupt;

#[app(device = stm32f401)]
const APP: () = {
    #[init]
    fn init(mut ctx: init::Context) {
        rtic::pend(Interrupt::EXTI0);

        unsafe {
            // Set TSENA (enable trace timestamps)
            ctx.core.ITM.tcr.modify(|r| r | (1 << 1));
        }
    }

    // taben after `init` returns
    #[task(binds = EXTI0, priority = 1)]
    fn exti0(_: exti0::Context) {
        rtic::pend(Interrupt::EXTI2);

        // wait until all ITM packets are flushed
        asm::delay(256);

        asm::bkpt();            // stop tracing
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
