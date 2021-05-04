#![allow(unreachable_code)]
use anyhow::Result;
use proc_macro2::{TokenStream, TokenTree};
use syn;
use std::path::PathBuf;
use structopt::StructOpt;
use std::fs;

#[derive(StructOpt)]
struct Opt {
    #[structopt(
        name = "RTIC-SOURCE-FILE",
        parse(from_os_str),
        help = "The RTIC source file that should be parsed."
    )]
    file: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    // Parse the RTIC app from the source file
    let src = fs::read_to_string(opt.file).unwrap();
    let mut rtic_app = syn::parse_str::<TokenStream>(&src)
        .expect("Unable to parse file")
        .into_iter()
        .skip_while(|token| {
            if let TokenTree::Group(g) = token {
                return g.stream().into_iter().nth(0).unwrap().to_string().as_str() != "app";
            }
            true
        });
    let args = {
        let mut args: Option<TokenStream> = None;
        if let TokenTree::Group(g) = rtic_app.next().unwrap() {
            if let TokenTree::Group(g) = g.stream().into_iter().nth(1).unwrap() {
                args = Some(g.stream());
            }
        }
        args.unwrap()
    };
    let app = rtic_app.collect::<TokenStream>();


    for (int, (fun, ex_ident)) in rtic_trace::parsing::hardware_tasks(app.clone(), args)? {
        println!("{} binds {} ({})", fun[1], ex_ident, int);
    }

    for (k, v) in rtic_trace::parsing::software_tasks(app)? {
        println!("({}, {:?})", k, v);
    }

    Ok(())
}
