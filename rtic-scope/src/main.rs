#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    let x = 42;

    loop {
        cortex_m::asm::nop;
    }
}
