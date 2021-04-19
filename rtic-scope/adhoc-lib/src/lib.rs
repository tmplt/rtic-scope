#![allow(improper_ctypes_definitions)]
use stm32f4::stm32f401::Interrupt;
use cortex_m::interrupt::Nr;
// use quote::{quote, format_ident};

#[no_mangle]
pub extern fn rtic_scope_func(name: &str) -> u8 {
    Interrupt::EXTI0.nr()
}
