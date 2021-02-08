//! This example simply prints a string to an ITM stimulus port.

#![no_std]
#![no_main]

use panic_halt as _;            // panic handler
use stm32f4::stm32f401;
use rtic::app;
use cortex_m::asm;
use stm32f401::Interrupt;
use cortex_m::iprintln;

#[app(device = stm32f401)]
const APP: () = {
    #[init]
    fn init(mut ctx: init::Context) {
        let stim = &mut ctx.core.ITM.stim[0];

        iprintln!(stim, "Hello, again!");
    }
};
