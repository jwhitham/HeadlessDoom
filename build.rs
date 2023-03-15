use std::fs;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    // Use the `cc` crate to build a C file and statically link it.
    let mut compiler = cc::Build::new();
    compiler.define("HEADLESS", "");

    if let Ok(entries) = fs::read_dir("src/c") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let path = entry.path();
                        if let Some(path) = path.as_path().to_str() {
                            println!("cargo:rerun-if-changed={}", path);
                            if path.ends_with(".c") {
                                compiler.file(path);
                            }
                        }
                    }
                }
            }
        }
    }
    compiler.compile("hello");
}

