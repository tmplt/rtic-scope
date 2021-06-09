// We want to be able to run
//
//     $ cargo rtic-trace --bin blinky --serial /dev/ttyUSB3
//

use proc_macro2::{TokenStream, TokenTree};
use std::env;
use std::fs;
use std::io::Read;
use syn;

use anyhow::{bail, Context, Result};
use itm_decode::{self, DecoderState};
use probe_rs::{flashing, Probe};
use structopt::StructOpt;

mod building;
mod serial;

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-rtic-trace", about = "TODO")]
struct Opt {
    /// Binary to flash and trace.
    #[structopt(long = "bin")]
    bin: String,

    // TODO handle --example (or forward unknown arguments to rustc)
    /// Serial device over which trace stream is expected.
    #[structopt(long = "serial")]
    serial: String,

    /// Don't attempt to resolve hardware or software tasks.
    #[structopt(long = "dont-resolve")]
    dont_resolve: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_iter(
        // NOTE(skip): first argument is the subcommand name
        env::args().skip(1),
    );

    // Attach to target and prepare serial. We want to fail fast on any
    // I/O issues.
    //
    // TODO allow user to specify probe and target
    let probes = Probe::list_all();
    if probes.is_empty() {
        bail!("No supported target probes found");
    }
    println!("Opening first probe and attaching...");
    let probe = probes[0].open().context("Unable to open first probe")?;
    let mut session = probe
        .attach("stm32f401re")
        .context("Unable to attach to stm32f401re")?;
    let mut trace_tty = serial::configure(opt.serial)?;

    // Build the wanted binary
    let artifact = building::cargo_build(&opt.bin)?;

    // resolve the data we need from RTIC app decl.
    if !opt.dont_resolve {
        // parse the RTIC app from the source file
        let src = fs::read_to_string(artifact.target.src_path)
            .context("Failed to open RTIC app source file")?;
        let mut rtic_app = syn::parse_str::<TokenStream>(&src)
            .context("Failed to parse RTIC app source file")?
            .into_iter()
            .skip_while(|token| {
                // TODO improve this
                if let TokenTree::Group(g) = token {
                    return g.stream().into_iter().nth(0).unwrap().to_string().as_str() != "app";
                }
                true
            });
        let args = {
            let mut args: Option<TokenStream> = None;
            if let TokenTree::Group(g) = rtic_app.next().unwrap() {
                // TODO improve this
                if let TokenTree::Group(g) = g.stream().into_iter().nth(1).unwrap() {
                    args = Some(g.stream());
                }
            }
            args.unwrap()
        };
        let app = rtic_app.collect::<TokenStream>();

        println!("Hardware tasks:");
        let (int, ext) = rtic_trace::parsing::hardware_tasks(app.clone(), args)?;
        println!("int: {:?}, ext: {:?}", int, ext);
        // for (int, (fun, ex_ident)) in rtic_trace::parsing::hardware_tasks(app.clone(), args)? {
        //     println!("{} binds {} ({})", fun[1], ex_ident, int);
        // }

        println!("Software tasks:");
        for (k, v) in rtic_trace::parsing::software_tasks(app)? {
            println!("({}, {:?})", k, v);
        }
    }

    // Flash binary to target
    //
    // TODO use a progress bar alike cargo-flash
    println!("Flashing {}...", opt.bin);
    flashing::download_file(
        &mut session,
        &artifact.executable.unwrap(),
        flashing::Format::Elf,
    )
    .context("Unable to flash target firmware")?;
    println!("Flashed.");

    // Reset the target and execute flashed binary
    println!("Resetting target...");
    let mut core = session.core(0)?;
    core.reset().context("Unable to reset target")?;
    println!("Reset.");

    // TODO collect trace until some stop condition
    // TODO start collecting before target is reset
    let mut decoder = itm_decode::Decoder::new();
    let mut buf: [u8; 256] = [0; 256];
    while let Ok(n) = trace_tty.read(&mut buf) {
        println!("I read {} bytes", n);
        decoder.push(buf[..n].to_vec());
        buf = [0; 256];

        loop {
            match decoder.pull() {
                Ok(None) => break,
                Ok(Some(packet)) => println!("{:?}", packet),
                Err(e) => {
                    println!("Error: {:?}", e);
                    decoder.state = DecoderState::Header;
                }
            }
        }
    }

    // TODO save trace somewhere for offline analysis.

    Ok(())
}
