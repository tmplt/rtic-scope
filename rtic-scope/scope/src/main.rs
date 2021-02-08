use probe_rs::{Probe, Error, MemoryInterface};
use std::time::Duration;

fn main() -> Result<(), Error> {
    // Get a list of all available debug probes
    let probes = Probe::list_all();
    if probes.is_empty() {
        return Err(Error::UnableToOpenProbe("No probes available"));
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

    // Enable ITM exception tracing
    {
        // Enable exception tracing
        const DWT_CTRL_ADDR: u32 = 0xE0001000;
        let mut ctrl: u32 = core.read_word_32(DWT_CTRL_ADDR)?;
        println!("before\tctrl = {:x}", ctrl);
        ctrl |= 1 << 16;    // set EXCTRENA
        ctrl &= !(1 << 12); // clear PCSAMLENA
        println!("after\tctrl = {:x}", ctrl);
        core.write_word_32(DWT_CTRL_ADDR, ctrl)?;
        let ctrl: u32 = core.read_word_32(DWT_CTRL_ADDR)?;
        println!("readb\tctrl = {:x}", ctrl);
        // assert!(ctrl == core.read_word_32(DWT_CTRL_ADDR)?);

        // Enable ITM port 0
        const ITM_TER0: u32 = 0xE0000E00;
        core.write_word_32(ITM_TER0, 1 << 0)?;
    }

    Ok(())
}
