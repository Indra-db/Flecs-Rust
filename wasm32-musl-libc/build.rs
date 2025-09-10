use build_print::{info, warn};

#[cfg(feature = "bindgen")]
fn generate_bindings() {
    use std::{env, path::PathBuf};

    // This build script generates Rust FFI bindings for wasi-libc headers using bindgen

    // Generate alltypes.h if it doesn't exist (needed for bindgen)
    let alltypes_path = "src/generated_headers/bits/alltypes.h";
    if !std::path::Path::new(alltypes_path).exists() {
        generate_alltypes_h();
    }

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
        // Add include paths so bindgen can find headers - put generated headers first
        .clang_arg("-Isrc/generated_headers")
        .clang_arg("-Isrc/custom_headers")
        .clang_arg("-Isrc/libc-top-half/musl/include")
        .clang_arg("-Isrc/libc-top-half/musl/arch/wasm32")
        .clang_arg("-Isrc/libc-top-half/headers")
        // Define the macro to use upstream musl definitions instead of WASI headers
        .clang_arg("-D__wasilibc_unmodified_upstream")
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
        .blocklist_item("LONG_BIT")
        .blocklist_item("__builtin_va_list")
        .raw_line("//manually set const, types for correct wasm32 target")
        .raw_line("pub const LONG_BIT: u32 = 32;")
        .raw_line("pub type __builtin_va_list = *mut ::core::ffi::c_void;")
        // Finish the builder and generate the bindings
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to src/bindings.rs in the crate root (same as flecs_ecs_sys)
    let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    bindings
        .write_to_file(crate_root.join("src/bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[allow(dead_code)]
fn generate_alltypes_h() {
    use std::path::Path;
    use std::process::Command;

    let output_path = "src/generated_headers/bits/alltypes.h";
    let musl_dir = "src/libc-top-half/musl";

    println!(
        "cargo:rerun-if-changed={}/arch/wasm32/bits/alltypes.h.in",
        musl_dir
    );
    println!("cargo:rerun-if-changed={}/include/alltypes.h.in", musl_dir);

    // Create the generated_headers directory if it doesn't exist
    let output_dir = Path::new(output_path).parent().unwrap();
    std::fs::create_dir_all(output_dir).expect("Failed to create generated_headers directory");

    // Try to use the official musl mkalltypes.sed tool first
    let sed_script = format!("{}/tools/mkalltypes.sed", musl_dir);
    if Path::new(&sed_script).exists() {
        // Combine the arch and generic templates
        let arch_template = format!("{}/arch/wasm32/bits/alltypes.h.in", musl_dir);
        let generic_template = format!("{}/include/alltypes.h.in", musl_dir);

        if Path::new(&arch_template).exists() && Path::new(&generic_template).exists() {
            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "cat {} {} | sed -f {} > {}",
                    arch_template, generic_template, sed_script, output_path
                ))
                .status()
                .expect("Failed to run sed script");

            if status.success() {
                // Post-process the generated file to handle __wasilibc_unmodified_upstream
                fix_alltypes_for_upstream(output_path);
                info!("Generated alltypes.h using musl tools with upstream fixes");
                return;
            }
        }
    }

    // Fall back to manual generation if musl tools fail
    warn!("Musl build tools not available, falling back to manual alltypes.h generation");
    generate_alltypes_h_manual();
}

#[allow(dead_code)]
fn fix_alltypes_for_upstream(alltypes_path: &str) {
    use std::fs;

    let content = fs::read_to_string(alltypes_path).expect("Failed to read generated alltypes.h");

    // Replace the problematic wchar_t definition for upstream mode
    let fixed_content = content.replace(
        "#if defined(__NEED_wchar_t) && !defined(__DEFINED_wchar_t)\n#define __need_wchar_t\n#include <stddef.h>\n#define __DEFINED_wchar_t\n#endif",
        "#if defined(__NEED_wchar_t) && !defined(__DEFINED_wchar_t)\n#ifdef __wasilibc_unmodified_upstream\ntypedef int wchar_t;\n#else\n#define __need_wchar_t\n#include <stddef.h>\n#endif\n#define __DEFINED_wchar_t\n#endif"
    ).replace(
        "#if defined(__NEED_wint_t) && !defined(__DEFINED_wint_t)\n#define __need_wint_t\n#include <stddef.h>\n#define __DEFINED_wint_t\n#endif",
        "#if defined(__NEED_wint_t) && !defined(__DEFINED_wint_t)\n#ifdef __wasilibc_unmodified_upstream\ntypedef unsigned int wint_t;\n#else\n#define __need_wint_t\n#include <stddef.h>\n#endif\n#define __DEFINED_wint_t\n#endif"
    );

    fs::write(alltypes_path, fixed_content).expect("Failed to write fixed alltypes.h");
}

#[allow(dead_code)]
fn generate_alltypes_h_manual() {
    use std::fs;

    let arch_template = "src/libc-top-half/musl/arch/wasm32/bits/alltypes.h.in";
    let generic_template = "src/libc-top-half/musl/include/alltypes.h.in";
    let output_path = "src/generated_headers/bits/alltypes.h";

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

    info!("Generated alltypes.h manually from templates");
}

#[allow(dead_code)]
fn apply_typedef_transformations(content: &str) -> String {
    let mut result = String::new();

    // Add conditional block for upstream vs WASI
    result.push_str("#ifdef __wasilibc_unmodified_upstream\n");
    result.push_str("/* Use upstream musl definitions */\n\n");

    // First pass - handle the arch-specific defines and macros
    for line in content.lines() {
        // Keep #define lines that set up basic constants
        if line.starts_with("#define _") || line.starts_with("#define __") {
            result.push_str(line);
            result.push('\n');
        }
    }

    result.push_str("\n");

    // Second pass - process TYPEDEF and STRUCT for upstream mode
    for line in content.lines() {
        if line.starts_with("TYPEDEF ") {
            if let Some(rest) = line.strip_prefix("TYPEDEF ") {
                if let Some(semicolon_pos) = rest.rfind(';') {
                    let typedef_part = &rest[..semicolon_pos].trim();
                    if let Some(last_space) = typedef_part.rfind(' ') {
                        let type_part = &typedef_part[..last_space].trim();
                        let name_part = &typedef_part[last_space + 1..].trim();

                        // For upstream mode, provide direct typedefs with proper C types
                        let upstream_type = match *name_part {
                            "wchar_t" => "int", // wchar_t is int in musl for wasm32
                            "wint_t" => "unsigned int",
                            "size_t" => "unsigned long",
                            "ssize_t" => "long",
                            _ => type_part,
                        };

                        result.push_str(&format!(
                            "#if defined(__NEED_{}) && !defined(__DEFINED_{})\n\
                            typedef {} {};\n\
                            #define __DEFINED_{}\n\
                            #endif\n\n",
                            name_part, name_part, upstream_type, name_part, name_part
                        ));
                    }
                }
            }
        } else if line.starts_with("STRUCT ") {
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
                            #endif\n\n",
                            name_part, name_part, name_part, body_part, name_part
                        ));
                    }
                }
            }
        }
    }

    result.push_str("#else\n");
    result.push_str("/* Use WASI-specific definitions */\n\n");

    // Third pass - handle WASI mode (original templates)
    for line in content.lines() {
        // Keep #define lines that set up basic constants
        if line.starts_with("#define _") || line.starts_with("#define __") {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if line.starts_with("TYPEDEF ") {
            if let Some(rest) = line.strip_prefix("TYPEDEF ") {
                if let Some(semicolon_pos) = rest.rfind(';') {
                    let typedef_part = &rest[..semicolon_pos].trim();
                    if let Some(last_space) = typedef_part.rfind(' ') {
                        let name_part = &typedef_part[last_space + 1..].trim();

                        result.push_str(&format!(
                            "#if defined(__NEED_{}) && !defined(__DEFINED_{})\n\
                            #define __need_{}\n\
                            #include <stddef.h>\n\
                            #define __DEFINED_{}\n\
                            #endif\n\n",
                            name_part, name_part, name_part, name_part
                        ));
                    }
                }
            }
        } else if line.starts_with("STRUCT ") {
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
                            #endif\n\n",
                            name_part, name_part, name_part, body_part, name_part
                        ));
                    }
                }
            }
        } else if line.contains("#include")
            && (line.contains("__typedef_") || line.contains("__struct_"))
        {
            // Skip WASI-specific includes that don't exist in the top half
            continue;
        } else if !line.starts_with("/*")
            && !line.starts_with("*")
            && !line.starts_with("TYPEDEF")
            && !line.starts_with("STRUCT")
        {
            // Pass through other lines (comments, etc.) but skip TYPEDEF/STRUCT lines we already processed
            if !line.trim().is_empty() {
                result.push_str(line);
                result.push('\n');
            }
        }
    }

    result.push_str("\n#endif /* __wasilibc_unmodified_upstream */\n");
    result
}

#[cfg(feature = "build-libc")]
fn build_libc() {
    // Generate alltypes.h from templates first
    generate_alltypes_h();

    // Only build C library when targeting WASM
    let target = env::var("TARGET").unwrap_or_default();
    if target != "wasm32-unknown-unknown" {
        warn!(
            "Skipping C library build for non-WebAssembly target: {}",
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
        .include("src/libc-top-half/musl/src/internal")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-bitwise-op-parentheses")
        .flag("-Wno-shift-op-parentheses")
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
        .define("BULK_MEMORY_THRESHOLD", "8192")
        .define("hidden", "__attribute__((__visibility__(\"hidden\")))");

    // Build only core memory functions that compile cleanly
    let core_files = [
        "src/libc-top-half/musl/src/string/memcpy.c",
        "src/libc-top-half/musl/src/string/memmove.c",
        "src/libc-top-half/musl/src/string/memset.c",
        "src/libc-top-half/musl/src/string/memcmp.c",
        "src/libc-top-half/musl/src/string/memchr.c",
        "src/libc-top-half/musl/src/string/strlen.c",
        "src/libc-top-half/musl/src/string/strcmp.c",
        // Note: malloc/free functions require syscalls not available in WASM
        // Users needing memory allocation should use Rust's allocator or wee_alloc
    ];

    for file in &core_files {
        if std::path::Path::new(file).exists() {
            println!("cargo:rerun-if-changed={}", file);
            build.file(file);
        }
    }

    // Compile the static library
    build.compile("wasm32_musl_libc");

    // Copy the compiled library to a distribution directory
    let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_dir = PathBuf::from(&crate_root).join("lib");
    std::fs::create_dir_all(&dist_dir).expect("Failed to create lib directory");

    let lib_path = PathBuf::from(&out_dir).join("libwasm32_musl_libc.a");
    let dist_lib_path = dist_dir.join("libwasm32_musl_libc.a");

    if lib_path.exists() {
        std::fs::copy(&lib_path, &dist_lib_path)
            .expect("Failed to copy library to distribution directory");
        info!("Copied library to {}", dist_lib_path.display());
    }

    // Output library path for linking
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=wasm32_musl_libc");
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
            // Expose root & include paths to dependent build scripts (e.g. flecs_ecs_sys)
            println!("cargo:root={}", crate_root);
            println!(
                "cargo:include_custom_headers={}/src/custom_headers",
                crate_root
            );
            println!(
                "cargo:include_generated_headers={}/src/generated_headers",
                crate_root
            );
            println!(
                "cargo:include_musl_top={}/src/libc-top-half/musl/include",
                crate_root
            );
            println!(
                "cargo:include_musl_arch={}/src/libc-top-half/musl/arch/wasm32",
                crate_root
            );
            println!(
                "cargo:include_top_half_headers={}/src/libc-top-half/headers",
                crate_root
            );
            // Aggregate include search path list (':' separated) for convenience
            println!(
                "cargo:include_paths={}:{}:{}:{}:{}",
                format!("{}/src/custom_headers", crate_root),
                format!("{}/src/generated_headers", crate_root),
                format!("{}/src/libc-top-half/musl/include", crate_root),
                format!("{}/src/libc-top-half/musl/arch/wasm32", crate_root),
                format!("{}/src/libc-top-half/headers", crate_root),
            );
            // Provide canonical DEP style vars (Cargo uppercases key after links name)
            println!(
                "cargo:rustc-env=DEP_WASM32_MUSL_LIBC_INCLUDE_PATHS={}:{}:{}:{}:{}",
                format!("{}/src/custom_headers", crate_root),
                format!("{}/src/generated_headers", crate_root),
                format!("{}/src/libc-top-half/musl/include", crate_root),
                format!("{}/src/libc-top-half/musl/arch/wasm32", crate_root),
                format!("{}/src/libc-top-half/headers", crate_root),
            );
            println!("cargo:rustc-env=DEP_WASM32_MUSL_LIBC_WASILIBC_UPSTREAM=1");
            // Signal consumers to use upstream musl style (avoids __struct_* split headers)
            println!("cargo:wasilibc_upstream=1");
            let lib_path = PathBuf::from(&crate_root)
                .join("lib")
                .join("libwasm32_musl_libc.a");

            if lib_path.exists() {
                let lib_dir = PathBuf::from(&crate_root).join("lib");
                println!("cargo:rustc-link-search=native={}", lib_dir.display());
                println!("cargo:rustc-link-lib=static=wasm32_musl_libc");
                info!("Linking with pre-compiled library");
            } else {
                warn!("Pre-compiled library not found in lib/ directory");
                warn!("Run 'cargo build --features build-libc' first to generate the library");
            }
        } else {
            warn!("link-libc feature only supports wasm32-unknown-unknown target");
        }
    }
}
