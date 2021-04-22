#![no_std]

use stm32f4::stm32f401 as pac;

static mut WATCH_VARIABLE: u32 = 0;

pub fn set_current_task_id(id: u32) {
    unsafe {
        WATCH_VARIABLE = id;
    }
}

pub fn setup_dwt(core: &mut pac::CorePeripherals) {
    let watch_address: u32 = unsafe { &WATCH_VARIABLE as *const _ } as u32;
    let dwt = &core.DWT.c[1];

    // TODO do we need to clear the MATCHED, bit[24] after every match?
    unsafe {
        dwt.function.modify(|mut r| {
            r &= !(1 << 8); // clear DATAVMATCH; perform address comparison
            r &= !(1 << 5); // clear EMITRANGE; dont emit data trace address packets
            r |= 0b0010; // data trace valu packet on RW match TODO change to 0b1101 (WO)
            r
        });

        dwt.comp.write(watch_address);
        dwt.mask.write(0);
    }
}
