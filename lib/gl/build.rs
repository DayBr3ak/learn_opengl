use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let dest = PathBuf::from("./src/");
    println!("cargo:rerun-if-changed=build.rs");

    let mut file = File::create(&dest.join("bindings.rs")).unwrap();
    let registry = Registry::new(Api::Gles2, (3, 3), Profile::Core, Fallbacks::All, []);
    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        registry
            .write_bindings(gl_generator::DebugStructGenerator, &mut file)
            .unwrap();
    } else {
        registry
            .write_bindings(gl_generator::StructGenerator, &mut file)
            .unwrap();
    }
}
