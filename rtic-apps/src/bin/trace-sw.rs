#![no_std]
#![no_main]

use panic_halt as _; // panic handler
use rtic::app;

#[app(device = stm32f4::stm32f401, peripherals = true, dispatchers = [EXTI1])]
mod app {
    use cortex_m::asm;
    use rtic_trace::{self, trace};
    use stm32f4::stm32f401::Interrupt;

    #[init]
    fn init(mut ctx: init::Context) -> (init::LateResources, init::Monotonics) {
        rtic_trace::setup::core_peripherals(
            &mut ctx.core.DCB,
            &mut ctx.core.TPIU,
            &mut ctx.core.DWT,
            &mut ctx.core.ITM,
        );
        rtic_trace::setup::device_peripherals(&mut ctx.device.DBGMCU);
        rtic_trace::setup::assign_dwt_unit(&ctx.core.DWT.c[1]);

        rtic::pend(Interrupt::EXTI0);

        (init::LateResources {}, init::Monotonics())
    }

    #[task(binds = EXTI0, priority = 1)]
    fn spawner(_ctx: spawner::Context) {
        software_task::spawn().unwrap();
    }

    #[task]
    fn software_task(_ctx: software_task::Context) {

        #[trace]
        fn func() {}

        rtic_trace::set_current_task_id(42);
        asm::delay(1024);
        // rtic::pend(Interrupt::EXTI0);
    }
}
