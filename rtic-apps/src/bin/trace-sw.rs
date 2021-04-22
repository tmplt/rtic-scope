#![no_std]
#![no_main]

use panic_halt as _; // panic handler
use rtic::app;

#[app(device = stm32f4::stm32f401, peripherals = true, dispatchers = [EXTI1])]
mod app {
    use cortex_m::asm;
    use rtic_sw_task_trace as swtrace;
    use stm32f4::stm32f401::Interrupt;
    use trace_examples;

    #[init]
    fn init(mut ctx: init::Context) -> (init::LateResources, init::Monotonics) {
        trace_examples::enable_tracing(&mut ctx.core, &mut ctx.device);
        swtrace::setup_dwt(&mut ctx.core);

        rtic::pend(Interrupt::EXTI0);

        (init::LateResources {}, init::Monotonics())
    }

    #[task(binds = EXTI0, priority = 1)]
    fn spawner(_ctx: spawner::Context) {
        software_task::spawn().unwrap();
    }

    #[task]
    fn software_task(_ctx: software_task::Context) {
        swtrace::set_current_task_id(42);
        asm::delay(1024);
        // rtic::pend(Interrupt::EXTI0);
    }
}
