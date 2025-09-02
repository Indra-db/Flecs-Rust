#[cfg(feature = "bindgen")]
fn generate_bindings() {
    use std::{env, path::PathBuf};

    // This build script generates Rust FFI bindings for wasi-libc headers using bindgen

    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/custom_headers");
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
fn generate_alltypes_h() {
    use std::fs;

    let arch_template = "src/libc-top-half/musl/arch/wasm32/bits/alltypes.h.in";
    let generic_template = "src/libc-top-half/musl/include/alltypes.h.in";
    let output_path = "src/generated_headers/bits/alltypes.h";

    println!("cargo:rerun-if-changed={}", arch_template);
    println!("cargo:rerun-if-changed={}", generic_template);

    // Create the generated_headers directory if it doesn't exist
    let output_dir = std::path::Path::new(output_path).parent().unwrap();
    fs::create_dir_all(output_dir).expect("Failed to create generated_headers directory");

    // Read both template files
    let arch_content =
        fs::read_to_string(arch_template).expect("Failed to read arch-specific alltypes.h.in");
    let generic_content =
        fs::read_to_string(generic_template).expect("Failed to read generic alltypes.h.in");

    // Combine the templates (arch-specific first, then generic)
    let combined = format!("{}\n{}", arch_content, generic_content);

    // Apply sed-like transformations for TYPEDEF macros
    let processed = apply_typedef_transformations(&combined);

    // Add header comment to indicate this is a generated file
    let final_content = format!(
        "/* This file is generated automatically by build.rs from templates.\n\
         * Do not edit manually - changes will be overwritten.\n\
         * Generated from:\n\
         *   - {}\n\
         *   - {}\n\
         */\n\n{}",
        arch_template, generic_template, processed
    );

    // Write the generated file
    fs::write(output_path, final_content).expect("Failed to write generated alltypes.h");

    println!("cargo:warning=Generated alltypes.h from templates");
}

#[allow(dead_code)]
fn apply_typedef_transformations(content: &str) -> String {
    let mut result = String::new();

    for line in content.lines() {
        if line.starts_with("TYPEDEF ") {
            // Transform: TYPEDEF <type> <name>;
            // Into: #ifdef __NEED_<name> ... conditional typedef
            if let Some(rest) = line.strip_prefix("TYPEDEF ") {
                if let Some(semicolon_pos) = rest.rfind(';') {
                    let typedef_part = &rest[..semicolon_pos].trim();
                    if let Some(last_space) = typedef_part.rfind(' ') {
                        let type_part = &typedef_part[..last_space].trim();
                        let name_part = &typedef_part[last_space + 1..].trim();

                        result.push_str(&format!(
                            "#if defined(__NEED_{}) && !defined(__DEFINED_{})\n\
                            typedef {} {};\n\
                            #define __DEFINED_{}\n\
                            #endif\n",
                            name_part, name_part, type_part, name_part, name_part
                        ));
                        continue;
                    }
                }
            }
        } else if line.starts_with("STRUCT ") {
            // Transform STRUCT patterns similarly
            if let Some(rest) = line.strip_prefix("STRUCT ") {
                if let Some(semicolon_pos) = rest.rfind(';') {
                    let struct_part = &rest[..semicolon_pos].trim();
                    if let Some(first_space) = struct_part.find(' ') {
                        let name_part = &struct_part[..first_space].trim();
                        let body_part = &struct_part[first_space + 1..].trim();

                        result.push_str(&format!(
                            "#if defined(__NEED_struct_{}) && !defined(__DEFINED_struct_{})\n\
                            struct {} {};\n\
                            #define __DEFINED_struct_{}\n\
                            #endif\n",
                            name_part, name_part, name_part, body_part, name_part
                        ));
                        continue;
                    }
                }
            }
        }

        // Pass through all other lines unchanged
        result.push_str(line);
        result.push('\n');
    }

    result
}

#[cfg(feature = "build-libc")]
fn build_libc() {
    // Generate alltypes.h from templates first
    generate_alltypes_h();

    // Only build C library when targeting WASM
    let target = env::var("TARGET").unwrap_or_default();
    if target != "wasm32-unknown-unknown" {
        println!(
            "cargo:warning=Skipping C library build for non-WASM target: {}",
            target
        );
        return;
    }

    println!("cargo:rerun-if-changed=src/custom_headers");
    println!("cargo:rerun-if-changed=src/generated_headers");
    println!("cargo:rerun-if-changed=src/libc-top-half");

    let out_dir = env::var("OUT_DIR").unwrap();

    let mut build = cc::Build::new();

    // Configure for wasm32-unknown-unknown target
    build
        .target("wasm32-unknown-unknown")
        .include("src/custom_headers")
        .include("src/generated_headers")
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
        println!(
            "cargo:warning=Copied libwasi_libc.a to {}",
            dist_lib_path.display()
        );
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
            let lib_path = PathBuf::from(&crate_root)
                .join("lib")
                .join("libwasi_libc.a");

            if lib_path.exists() {
                let lib_dir = PathBuf::from(&crate_root).join("lib");
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                println!("cargo:rustc-link-lib=static=wasi_libc");
                println!("cargo:warning=Linking with pre-compiled libwasi_libc.a");
            } else {
                println!("cargo:warning=Pre-compiled libwasi_libc.a not found in lib/ directory");
                println!(
                    "cargo:warning=Run with --features build-libc first to generate the library"
                );
            }
        } else {
            println!("cargo:warning=link-libc feature is only supported for wasm32-unknown-unknown target");
        }
    }
}
