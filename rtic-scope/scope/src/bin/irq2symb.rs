#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables, unreachable_code, unused_mut)]

// use addr2line::{self, fallible_iterator::FallibleIterator, object::Object, object::ObjectSection};
use anyhow::{anyhow, Context, Result};
use fallible_iterator::FallibleIterator;
use gimli::{self, read::AttributeValue};
use object::{Object, ObjectSection};
use probe_rs::{
    flashing::{self, Format},
    Core, MemoryInterface, Probe,
};
use probe_rs_cli_util;
use std::borrow;
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
    let obj = object::File::parse(include_bytes!(
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
        (1..16 + (vt.len() / WORD_LEN)) // the number of interrupt vectors and their offsets
            .zip(vt.chunks(WORD_LEN_IN_BYTES).skip(1)) // first entry is initial SP
            .map(|(i, word)| (i, u32::from_le_bytes(word.try_into().unwrap())))
    };

    let dwarf_cow = {
        let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
            match obj.section_by_name(id.name()) {
                Some(ref section) => Ok(section
                    .uncompressed_data()
                    .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
                None => Ok(borrow::Cow::Borrowed(&[][..])),
            }
        };

        // Load a supplementary section. We don't have a supplementary object file,
        // so always return an empty slice.
        let load_section_sup = |_| Ok(borrow::Cow::Borrowed(&[][..]));

        gimli::Dwarf::load(&load_section, &load_section_sup)?
    };

    let dwarf = {
        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section: &dyn for<'a> Fn(
            &'a borrow::Cow<[u8]>,
        )
            -> gimli::EndianSlice<'a, gimli::RunTimeEndian> = &|section| {
            gimli::EndianSlice::new(
                &*section,
                if obj.is_little_endian() {
                    gimli::RunTimeEndian::Little
                } else {
                    gimli::RunTimeEndian::Big
                },
            )
        };

        // Create `EndianSlice`s for all of the sections.
        dwarf_cow.borrow(&borrow_section)
    };

    let mut unit_entries = dwarf
        .units()
        .map(|header| dwarf.unit(header))
        .for_each(|unit| {
            let mut entries = unit.entries();
            while let Some((_, entry)) = entries.next_dfs()? {
                if entry.tag() == gimli::DW_TAG_subprogram {
                    // parse_subprogram here
                    let mut attrs = entry.attrs();
                    while let Some(attr) = attrs.next()? {
                        if attr.name() == gimli::constants::DW_AT_name {
                            if let AttributeValue::DebugStrRef(offset) = attr.value() {
                                println!(
                                    "function name: {}",
                                    dwarf
                                        .string(offset)
                                        .unwrap()
                                        .to_string()
                                        .unwrap()
                                        .to_string()
                                );
                            }
                        }
                    }

                    // println!("function");
                    // panic!();
                }
            }

            Ok(())
        });

    return Ok(());

    // // Figure out the handler function names and print them
    // let ctx = addr2line::Context::new(&obj)?;
    // for (i, handler) in int_handlers {
    //     let function_name: String = if let Some(frame) = ctx.find_frames(handler as u64)?.next()? {
    //         frame.function.unwrap().demangle()?.into()
    //     } else {
    //         continue;
    //     };
    //     println!("VTABLE[{:3}]: 0x{:x}: {}", i, handler, function_name);
    // }

    // TODO figure out the actual RTIC task name

    Ok(())
}
