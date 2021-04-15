#![allow(unreachable_code)]
use anyhow::Result;
use proc_macro2::{TokenStream, TokenTree};
use rtic_syntax::{self, Settings};
use syn;
use std::io::{self, Write};

fn main() -> Result<()> {
    // Parse the RTIC app from the source file
    let src = String::from_utf8_lossy(include_bytes!(
        "/home/tmplt/exjobb/rtic-scope/playground/src/bin/tracing.rs"
    ));
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

    let mut settings = Settings::default();
    settings.parse_binds = true;
    let (app, _analysis) = rtic_syntax::parse2(args, app, settings).unwrap();

    app.hardware_tasks
        .iter()
        .map(|(name, ht)| (name.to_string(), ht.args.binds.to_string()))
        .for_each(|(name, bind)| {
            println!("{} binds {}", name, bind);
        });

    for ident in app
        .args
        .device
        .as_ref()
        .unwrap()
        .segments
        .iter()
        .map(|ps| ps.ident.to_string())
    {
        print!("{}::", ident);
        io::stdout().flush().unwrap();
    }
    println!("Interrupt");

    Ok(())
}
