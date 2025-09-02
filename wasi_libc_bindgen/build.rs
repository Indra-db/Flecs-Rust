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
        // Don't target WASM in bindgen to avoid missing headers, but disable layout tests
        .layout_tests(false)
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

#[cfg(feature = "build-libc")]
fn build_libc() {
    // Only build C library when targeting WASM
    let target = env::var("TARGET").unwrap_or_default();
    if target != "wasm32-unknown-unknown" {
        println!("cargo:warning=Skipping C library build for non-WASM target: {}", target);
        return;
    }
    
    println!("cargo:rerun-if-changed=src/libc-top-half");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    
    let mut build = cc::Build::new();
    
    // Configure for wasm32-unknown-unknown target
    build
        .target("wasm32-unknown-unknown")
        .include("src/libc-top-half/musl/include")
        .include("src/libc-top-half/musl/arch/wasm32")
        .include("src/libc-top-half/headers")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-nostdlib")
        .flag("-fno-builtin")
        .flag("-ffreestanding")
        .flag("-fvisibility=hidden")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .define("__wasm32__", None)
        .define("__wasm__", None)
        .define("__wasilibc_unmodified_upstream", None)
        .define("_WASI_EMULATED_MMAN", None)
        .define("_WASI_EMULATED_SIGNAL", None)
        .define("_WASI_EMULATED_PROCESS_CLOCKS", None)
        .define("BULK_MEMORY_THRESHOLD", "8192");
    
    // Build only core memory functions that compile cleanly
    let core_files = [
        "src/libc-top-half/musl/src/string/memcpy.c",
        "src/libc-top-half/musl/src/string/memmove.c", 
        "src/libc-top-half/musl/src/string/memset.c",
        "src/libc-top-half/musl/src/string/memcmp.c",
        "src/libc-top-half/musl/src/string/memchr.c",
        "src/libc-top-half/musl/src/string/strlen.c",
        "src/libc-top-half/musl/src/string/strcmp.c",
    ];
    
    for file in &core_files {
        if std::path::Path::new(file).exists() {
            println!("cargo:rerun-if-changed={}", file);
            build.file(file);
        }
    }
    
    // Compile the static library
    build.compile("wasi_libc");
    
    // Copy the compiled library to a distribution directory
    let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_dir = PathBuf::from(&crate_root).join("lib");
    std::fs::create_dir_all(&dist_dir).expect("Failed to create lib directory");
    
    let lib_path = PathBuf::from(&out_dir).join("libwasi_libc.a");
    let dist_lib_path = dist_dir.join("libwasi_libc.a");
    
    if lib_path.exists() {
        std::fs::copy(&lib_path, &dist_lib_path)
            .expect("Failed to copy library to distribution directory");
        println!("cargo:warning=Copied libwasi_libc.a to {}", dist_lib_path.display());
    }
    
    // Output library path for linking
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=wasi_libc");
}

use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "bindgen")]
    generate_bindings();
    
    #[cfg(feature = "build-libc")]
    build_libc();

    // Handle optional linking with pre-compiled libc.a
    if cfg!(feature = "link-libc") {
        let target = env::var("TARGET").unwrap_or_default();
        if target == "wasm32-unknown-unknown" {
            let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
            let lib_path = PathBuf::from(&crate_root).join("lib").join("libwasi_libc.a");
            
            if lib_path.exists() {
                let lib_dir = PathBuf::from(&crate_root).join("lib");
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                println!("cargo:rustc-link-lib=static=wasi_libc");
                println!("cargo:warning=Linking with pre-compiled libwasi_libc.a");
            } else {
                println!("cargo:warning=Pre-compiled libwasi_libc.a not found in lib/ directory");
                println!("cargo:warning=Run with --features build-libc first to generate the library");
            }
        } else {
            println!("cargo:warning=link-libc feature is only supported for wasm32-unknown-unknown target");
        }
    }
}
