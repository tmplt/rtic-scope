#![no_std]

use stm32f4::stm32f401::{CorePeripherals, Peripherals};

/// Enables tracing over async SWO at 115200 Bd.
pub fn enable_tracing(core: &mut CorePeripherals, device: &mut Peripherals) {
    unsafe {
        // enable tracing
        core.DCB.enable_trace();

        // set baud rate
        core.TPIU.acpr.write((16_000_000 / 115200) - 1);

        // Async SWO, NRZ encoding
        core.TPIU.sppr.write(0b10);

        // configure TPIU formatter
        core.TPIU.ffcr.modify(|r| r & !(1 << 1)); // clear EnFCont; drops ETM packets

        #[rustfmt::skip]
        device.DBGMCU.cr.modify(|_, w| w
                                    .trace_ioen().set_bit()
                                    .trace_mode().bits(0b00) // TRACE pin assignment for async mode
                                    .dbg_sleep().clear_bit() // disable clocks in STOP mode
        );

        core.DWT.ctrl.modify(|mut r| {
            r |= 1 << 16; // set EXCTRCENA; generate exception traces
            r &= !(1 << 12); // clear PCSAMPLENA; don't generate periodic timestamps
            r
        });

        core.ITM.lar.write(0xc5acce55); // unlock ITM register
        core.ITM.tcr.modify(|mut r| {
            r |= 1 << 0; // ITMENA: master enable
            r |= 1 << 3; // TXENA: forward DWT event packets to ITM
            r |= 1 << 16; // TraceBusID=1
            r
        });

        // enable stimulus port 0
        core.ITM.ter[0].write(1 << 0);

        // Set TSENA (enable trace timestamps)
        // ctx.core.ITM.tcr.modify(|r| r | (1 << 1));
    }
}
