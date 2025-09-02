#[cfg(feature = "bindgen")]
fn generate_bindings() {
    use std::{env, path::PathBuf};

    // This build script generates Rust FFI bindings for wasi-libc headers using bindgen

    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/libc-top-half");
    println!("cargo:rerun-if-changed=libc.a");
    println!("cargo:rerun-if-changed=build.rs");

    // Generate FFI bindings for C functions
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        // Keep comments and keep all of them, not just doc comments.
        .generate_comments(true)
        // Prefer core::* over std::*
        .use_core()
        // Don't generate a bunch of `link_name` attributes
        .trust_clang_mangling(false)
        .clang_arg("-fparse-all-comments")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Generate safe Rust style
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_hash(true)
        // Generate raw identifier names
        .raw_line("#[allow(non_upper_case_globals)]")
        .raw_line("#[allow(non_camel_case_types)]")
        .raw_line("#[allow(non_snake_case)]")
        .raw_line("#[allow(dead_code)]")
        .raw_line("#[allow(improper_ctypes)]")
        // Finish the builder and generate the bindings
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings.rs in the crate root (same as flecs_ecs_sys)
    let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    bindings
        .write_to_file(crate_root.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    #[cfg(feature = "bindgen")]
    generate_bindings();

    // Handle optional linking with libc.a when building for wasm32-unknown-unknown
    if cfg!(feature = "link-libc") {
        use std::env;
        if env::var("TARGET").unwrap_or_default() == "wasm32-unknown-unknown" {
            // Check if we have a libc.a file to link with
            if std::path::Path::new("libc.a").exists() {
                println!("cargo:rustc-link-search=native=.");
                println!("cargo:rustc-link-lib=static=c");
                println!("cargo:rustc-link-arg=--allow-undefined");
                println!("cargo:warning=Linking with libc.a for WASM target");
            } else {
                println!("cargo:warning=libc.a not found - enable 'link-libc' feature and provide libc.a for linking");
            }
        }
    }
}
