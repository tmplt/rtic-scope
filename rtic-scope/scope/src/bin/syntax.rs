use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{self, Settings};
use syn;

fn main() -> Result<()> {
    // let src = String::from_utf8_lossy(include_bytes!(
    //     "/home/tmplt/exjobb/rtic-scope/playground/src/bin/tracing.rs"
    // ));
    // let _syntax: TokenStream = syn::parse_str::<TokenStream>(&src).expect("Unable to parse file");

    let mut settings = Settings::default();
    settings.parse_binds = true;
    let (app, _analysis) = rtic_syntax::parse2(
        quote!(),
        quote!(
            mod app {
                #[task(binds = UART0)]
                fn foo(_: foo::Context) {}

                #[task(binds = UART1)]
                fn bar(_: bar::Context) {}

                #[task(binds = UART2)]
                fn baz(_: baz::Context) {}
            }
        ),
        settings,
    )
    .unwrap();

    app.hardware_tasks
        .iter()
        .map(|(name, ht)| (name.to_string(), ht.args.binds.to_string()))
        .for_each(|(name, bind)| {
            println!("{} binds {}", name, bind);
        });

    Ok(())
}
