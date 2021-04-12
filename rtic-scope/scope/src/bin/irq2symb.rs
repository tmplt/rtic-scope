#![allow(unused_imports)]
#![allow(dead_code)]

use addr2line::{self, object::Object, object::ObjectSection};
use anyhow::{anyhow, Context, Result};
// use object::{Object, ObjectSection, ObjectSymbol};
// use object;
use probe_rs::{
    flashing::{self, Format},
    Core, MemoryInterface, Probe,
};
use probe_rs_cli_util;
use std::cmp;
use std::convert::TryInto;
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

    // Find the interrupt handlers from the vector table
    let int_handlers = {
        let vt = obj
            .section_by_name(".vector_table")
            .context(".vector_table missing")?
            .data()?;

        const WORD_LEN: usize = 32;
        const WORD_LEN_IN_BYTES: usize = WORD_LEN / 8;
        (1..16 + (vt.len() / WORD_LEN))
            .zip(vt.chunks(WORD_LEN_IN_BYTES).skip(1)) // first entry is initial SP
            .map(|(i, word)| (i, u32::from_le_bytes(word.try_into().unwrap())))
    };

    // Figure out the handler function names and print them
    let ctx = addr2line::Context::new(&obj)?;
    for (i, handler) in int_handlers {
        let function_name: String = if let Some(frame) = ctx.find_frames(handler as u64)?.next()? {
            frame.function.unwrap().demangle()?.into()
        } else {
            continue;
        };
        println!("VTABLE[{:3}]: 0x{:x}: {}", i, handler, function_name);
    }

    // TODO figure out the actual RTIC task name

    Ok(())
}
