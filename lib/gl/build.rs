use gl_generator::{Api, Fallbacks, Profile, Registry};
// use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    // let dest = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let dest = PathBuf::from("./src/");

    println!("cargo:rerun-if-changed=build.rs");
    println!("{:?}", dest);

    let mut file = File::create(&dest.join("lib.rs")).unwrap();
    Registry::new(Api::Gles2, (3, 3), Profile::Core, Fallbacks::All, [])
        .write_bindings(gl_generator::StructGenerator, &mut file)
        .unwrap();
}
