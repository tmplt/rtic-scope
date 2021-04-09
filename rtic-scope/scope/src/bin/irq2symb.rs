#![allow(unused_imports)]
#![allow(dead_code)]

use addr2line;
use anyhow::{anyhow, Context, Result};
// use object::{Object, ObjectSection, ObjectSymbol};
// use object;
use probe_rs::{
    flashing::{self, Format},
    Core, MemoryInterface, Probe,
};
use probe_rs_cli_util;
use std::cmp;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "An ad-hoc \"scope\" for debugging and tracing purposes with RTIC.")]
struct Opt {
    // #[structopt(
//     name = "BINARY",
//     help = "The binary to flash and trace on the target",
// )]
// example: String,
}

fn main() -> Result<()> {
    let _opt = Opt::from_args();

    // Load the ELF
    let obj = addr2line::object::File::parse(include_bytes!(
        "/home/tmplt/exjobb/rtic-scope/target/thumbv7em-none-eabihf/debug/tracing"
    ))?;
    let ctx = addr2line::Context::new(&obj)?;

    // Get a list of all available debug probes
    let probes = Probe::list_all();
    if probes.is_empty() {
        return Err(anyhow!("No probes available"));
    }

    // Use the first probe found
    let probe = probes[0].open()?;

    // Attach to a chip
    let mut session = probe.attach_under_reset("stm32f4")?;

    // Select a core
    let mut core = session.core(0)?;

    // Halt the attached core
    core.halt(Duration::from_secs(5))?;
    assert!(core.core_halted()?);

    // Figure out where the vector table is
    const VTOR_ADDR: u32 = 0xE000_ED08;
    let vtor_tlboff = core.read_word_32(VTOR_ADDR)? >> 6;
    let vtable = (vtor_tlboff << 6) & 0b000_0000;

    // Figure out how long the vector table is
    const ICTR_ADDR: u32 = 0xE000_E004;
    let ictr_intlinesnum = core.read_word_32(ICTR_ADDR)? & 0b1111;
    let int_lines = cmp::max(32 * (ictr_intlinesnum + 1), 496);

    // Read entire vector table; assoc INT number with its handler address
    // TODO find the handlers from the ELF file instead
    let int_handlers =
        (1..=16 + int_lines).map(|i| (i, core.read_word_32(vtable + 0x4 * i).unwrap()));

    // Figure out the handler function names and print them
    for (i, handler) in int_handlers {
        let function_name: String = if let Some(frame) = ctx.find_frames(handler as u64)?.next()? {
            frame.function.unwrap().demangle()?.into()
        } else {
            continue;
        };
        println!("VTABLE[{:3}]: 0x{:016x}: {}", i, handler, function_name);
    }

    // TODO figure out the actual RTIC task name

    Ok(())
}
