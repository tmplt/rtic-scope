use probe_rs::{Core, MemoryInterface, Probe};
use std::time::Duration;

fn main() -> Result<(), probe_rs::Error> {
    // Get a list of all available debug probes
    let probes = Probe::list_all();
    if probes.is_empty() {
        return Err(probe_rs::Error::UnableToOpenProbe("No probes available"));
    }
    println!("Found {} probe(s): {:#?}", probes.len(), probes);

    // Use the first probe found
    println!("Opening the first probe...");
    let probe = probes[0].open()?;

    // Attach to a chip
    println!("Attaching...");
    let mut session = probe.attach("stm32")?;
    // session.setup_swv(...)
    println!("Found {} core(s).", session.list_cores().len());

    // Select a core
    let mut core = session.core(0)?;

    // Halt the attached core
    // XXX do we need to loop this?
    core.halt(Duration::from_secs(5))?;
    assert!(core.core_halted()?);

    // Check DEMCR
    const DEMCR_ADDR: u32 = 0xE000EDFC;
    let mut demcr: u32 = core.read_word_32(DEMCR_ADDR)?;
    demcr |= 1 << 24; // set TRCENA
    ensure_write_word_32(&mut core, DEMCR_ADDR, demcr)?;

    // Enable ITM exception tracing
    {
        // Enable exception tracing
        const DWT_CTRL_ADDR: u32 = 0xE0001000;
        let mut ctrl: u32 = core.read_word_32(DWT_CTRL_ADDR)?;
        ctrl |= 1 << 16; // set EXCTRENA
        ctrl &= !(1 << 12); // clear PCSAMLENA
        ensure_write_word_32(&mut core, DWT_CTRL_ADDR, ctrl)?;

        // openocd: monitor itm port 0 on

        // Before we do anything ITM we must first unlock the registers.
        // TODO only do this if LAR is implemented.
        const ITM_LAR: u32 = 0xe0000fb0;
        const ITM_LAR_KEY: u32 = 0xc5acce55;
        core.write_word_32(ITM_LAR, ITM_LAR_KEY)?;

        // ITM_TCR
        const ITM_TCR_ADDR: u32 = 0xE0000E80;
        let itm_tcr = (1 << 0)  // ITMENA; master enable
            | (1 << 3)          // TXENA; forward DWT event packets to ITM
            | (1 << 16); // TraceBusID = 1
        ensure_write_word_32(&mut core, ITM_TCR_ADDR, itm_tcr)?;

        // ITM_TER
        const ITM_TER0_ADDR: u32 = 0xE0000E00;
        const ITM_TER1_ADDR: u32 = 0xE0000E04;
        const ITM_TER7_ADDR: u32 = 0xE0000E1C;
        ensure_write_word_32(&mut core, ITM_TER0_ADDR, 1 << 0)?; // enable port 0
        for addr in (ITM_TER1_ADDR..=ITM_TER7_ADDR).step_by(0x4) {
            ensure_write_word_32(&mut core, addr, 0)?; // disable all other
        }
    }

    Ok(())
}

fn ensure_write_word_32(core: &mut Core, addr: u32, val: u32) -> Result<(), probe_rs::Error> {
    core.write_word_32(addr, val)?;
    if core.read_word_32(addr)? != val {
        return Err(probe_rs::Error::UnableToOpenProbe("readback failed"));
    }

    Ok(())
}
