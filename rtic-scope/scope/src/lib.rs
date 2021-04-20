use cargo;
use include_dir::include_dir;
use libloading;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use tempdir::TempDir;

pub fn resolve_int_nrs(binds: &[String]) -> BTreeMap<String, u8> {
    // generate a temporary directory
    let tmpdir = TempDir::new("rtic-scope-libadhoc").unwrap();

    // extract the included directory
    let libadhoc_tree = include_dir!("../libadhoc");
    for dir in libadhoc_tree.dirs() {
        fs::create_dir_all(tmpdir.path().join(dir.path())).unwrap();
    }
    let mut lib_src: Option<fs::File> = None;
    for file in libadhoc_tree
        .dirs()
        .iter()
        .flat_map(|d| d.files())
        .chain(libadhoc_tree.files())
    {
        let mut fsf = fs::File::create(tmpdir.path().join(file.path())).unwrap();
        fsf.write_all(file.contents()).unwrap();
        fsf.sync_all().unwrap();

        if file.path().to_str().unwrap() == "src/lib.rs" {
            lib_src = Some(fsf);
        }
    }

    // add the functions we need
    let mut lib_src = lib_src.unwrap();
    for bind in binds {
        let func = format_ident!("rtic_scope_func_{}", bind);
        let int_field = format_ident!("{}", bind);
        let src = quote!(
            #[no_mangle]
            pub extern fn #func() -> u8 {
                Interrupt::#int_field.nr()
            }
        );
        lib_src
            .write_all(format!("\n{}\n", src).as_bytes())
            .unwrap();
    }

    // cargo build the adhoc cdylib library
    // TODO use user-local cargo cache instead of the manifest directory
    let cc = cargo::util::config::Config::default().unwrap();
    let ws = cargo::core::Workspace::new(&tmpdir.path().join("Cargo.toml"), &cc).unwrap();
    let build = cargo::ops::compile(
        &ws,
        &cargo::ops::CompileOptions::new(&cc, cargo::core::compiler::CompileMode::Build).unwrap(),
    )
    .unwrap();
    assert!(build.cdylibs.len() == 1);

    // Load the library and find the bind mappings
    let lib = unsafe {
        libloading::Library::new(build.cdylibs.first().unwrap().path.as_os_str()).unwrap()
    };
    binds
        .into_iter()
        .map(|b| {
            let func: libloading::Symbol<extern "C" fn() -> u8> = unsafe {
                lib.get(format!("rtic_scope_func_{}", b).as_bytes())
                    .unwrap()
            };
            (b.clone(), func())
        })
        .collect()
}
