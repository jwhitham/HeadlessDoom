extern crate bindgen;
use std::fs;
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    // Use the `cc` crate to build a C file and statically link it.
    let mut compiler = cc::Build::new();
    compiler.define("HEADLESS", "");

    let mut bindings = bindgen::Builder::default();

    if let Ok(entries) = fs::read_dir("headless_doom") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        let name_str = path.file_name().unwrap().to_str().unwrap();
                        let path_str = path.as_path().to_str().unwrap();
                        if name_str != "i_main.c" {
                            println!("cargo:rerun-if-changed={}", path_str);
                            if name_str.ends_with(".c") {
                                compiler.file(path_str);
                            }
                            if name_str.ends_with(".h") {
                                bindings = bindings.header(path_str);
                            }
                        }
                    }
                }
            }
        }
    }

    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate().expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    compiler.compile("headless_doom_c");
}

