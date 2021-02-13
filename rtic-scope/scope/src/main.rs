use anyhow::{anyhow, Context, Result};
use probe_rs::{
    architecture::arm::swo::SwoConfig,
    architecture::arm::component::Dwt,
    flashing::{self, Format},
    Core, MemoryInterface, Probe,
};
use probe_rs_cli_util;
use std::path::PathBuf;
use std::time::Duration;
use std::fs::File;
use std::io::Write;

fn main() -> Result<()> {
    // Get a list of all available debug probes
    let probes = Probe::list_all();
    if probes.is_empty() {
        return Err(anyhow!("No probes available"));
    }
    println!("Found {} probe(s): {:#?}", probes.len(), probes);

    // Use the first probe found
    println!("Opening the first probe...");
    let probe = probes[0].open()?;

    // Attach to a chip
    println!("Attaching...");
    let mut session = probe.attach_under_reset("stm32f4")?;
    // session.setup_swv(...)
    println!("Found {} core(s).", session.list_cores().len());

    session.setup_swv(
        &SwoConfig::new(16_000_000)
            .set_baud(2_000_000)
            .set_continuous_formatting(false)
    )?;

    let component = session.get_arm_component()?;

    // Select a core
    let mut core = session.core(0)?;

    // Dwt::new(&mut core, &component).enable_exception_trace()?;

    // Halt the attached core
    core.halt(Duration::from_secs(5))?;
    assert!(core.core_halted()?);

    // General debug settings
    {
        const DBGMCU_APB1_FZ_ADDR: u32 = 0xE0042008;
        let mut dbgmcu_apb1_fz = core.read_word_32(DBGMCU_APB1_FZ_ADDR)?;
        dbgmcu_apb1_fz |= 0x1800; // stop watchdog counters during halt
        ensure_write_word_32(&mut core, DBGMCU_APB1_FZ_ADDR, dbgmcu_apb1_fz)?;
    }

    // Enable TRACE
    {
        const DEMCR_ADDR: u32 = 0xE000EDFC;
        let mut demcr: u32 = core.read_word_32(DEMCR_ADDR)?;
        demcr |= 1 << 24; // set TRCENA
        ensure_write_word_32(&mut core, DEMCR_ADDR, demcr)?;
    }

    // monitor tpiu config internal itm.bin uart off 16000000
    {
        // Set trace port size = 1
        const TPIU_CSPSR_ADDR: u32 = 0xe0040004;
        ensure_write_word_32(&mut core, TPIU_CSPSR_ADDR, 0x1)?;

        // Configure clock prescalar and thus SWO baud rate
        const TPIU_ACPR_ADDR: u32 = 0xe0040010;
        ensure_write_word_32(&mut core, TPIU_ACPR_ADDR, 0x7)?;

        // Configure trace output protocol: Async SWO, NRZ encoding
        const TPIU_SPPR_ADDR: u32 = 0xe00400f0;
        ensure_write_word_32(&mut core, TPIU_SPPR_ADDR, 0x2)?;

        // Configure TPIU formatter
        const TPIU_FFCR_ADDR: u32 = 0xe0040304;
        let mut tpiu_ffcr = core.read_word_32(TPIU_FFCR_ADDR)?;
        tpiu_ffcr &= !(1 << 1); // clear EnFCont; drops ETM packets
        ensure_write_word_32(&mut core, TPIU_FFCR_ADDR, tpiu_ffcr)?;

        // Configure debug settings
        const DBGMCU_CR_ADDR: u32 = 0xE0042004;
        let mut dbgmcu_cr = core.read_word_32(DBGMCU_CR_ADDR)?;
        dbgmcu_cr |= 1 << 5; // set TRACE_IOEN; use default trace pin mode and TRACEDATA size
        dbgmcu_cr &= !(1 << 0); // clear DBG_SLEEP: all clocks are disabled in STOP mode
        ensure_write_word_32(&mut core, DBGMCU_CR_ADDR, dbgmcu_cr)?;
    }

    // monitor mmw 0xE0001000 65536 4096
    {
        const DWT_CTRL_ADDR: u32 = 0xE0001000;
        let mut ctrl: u32 = core.read_word_32(DWT_CTRL_ADDR)?;
        ctrl |= 1 << 16; // set EXCTRENA
        ctrl &= !(1 << 12); // clear PCSAMLENA
        ensure_write_word_32(&mut core, DWT_CTRL_ADDR, ctrl)?;
    }

    // monitor itm port 0 on
    {
        // Before we do anything ITM we must first unlock the registers.
        // TODO only do this if LAR is implemented.
        const ITM_LAR: u32 = 0xe0000fb0;
        const ITM_LAR_KEY: u32 = 0xc5acce55;
        core.write_word_32(ITM_LAR, ITM_LAR_KEY)?;

        // Configure trace control register
        const ITM_TCR_ADDR: u32 = 0xE0000E80;
        let itm_tcr = (1 << 0)  // ITMENA; master enable
            | (1 << 3)          // TXENA; forward DWT event packets to ITM
            | (1 << 16); // TraceBusID = 1
        ensure_write_word_32(&mut core, ITM_TCR_ADDR, itm_tcr)?;

        // Enable ITM stimulus port 0, disable all other.
        const ITM_TER0_ADDR: u32 = 0xE0000E00;
        const ITM_TER1_ADDR: u32 = 0xE0000E04;
        const ITM_TER7_ADDR: u32 = 0xE0000E1C;
        ensure_write_word_32(&mut core, ITM_TER0_ADDR, 1 << 0)?; // enable port 0
        for addr in (ITM_TER1_ADDR..=ITM_TER7_ADDR).step_by(0x4) {
            ensure_write_word_32(&mut core, addr, 0)?; // disable all other
        }
    }

    core.run()?;
    drop(core);

    flash_program(&mut session)?;

    let mut f = File::create("/tmp/itm.bin")?;

    while let Ok(bytes) = session.read_swo() {
        if bytes.len() > 0 {
            f.write_all(&bytes)?;
        }
    }

    Ok(())
}

fn ensure_write_word_32(core: &mut Core, addr: u32, val: u32) -> Result<()> {
    core.write_word_32(addr, val)?;
    if core.read_word_32(addr)? != val {
        return Err(anyhow!("readback of register {} is unexpected!"));
    }

    Ok(())
}

fn flash_program(session: &mut probe_rs::Session) -> Result<()> {
    let work_dir = PathBuf::from("../playground/");
    // XXX always debug
    let path = probe_rs_cli_util::build_artifact(
        &work_dir,
        &["--example".to_string(), "tracing".to_string()],
    )?;
    flashing::download_file(session, &path, Format::Elf).context("failed to flash target")
}
