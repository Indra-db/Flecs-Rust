#![allow(clippy::all)]
#![allow(warnings)]

#[cfg(feature = "bindgen")]
fn generate_bindings() {
    use std::{env, path::PathBuf};

    #[derive(Debug)]
    struct CommentsCallbacks;

    impl bindgen::callbacks::ParseCallbacks for CommentsCallbacks {
        fn process_comment(&self, comment: &str) -> Option<String> {
            // 1: trimming the comments
            let comment = comment.trim();
            // 2: brackets do not entail intra-links
            let comment = comment.replace("[", "\\[");
            let comment = comment.replace("]", "\\]");

            // ensure all links are padded with < and >
            let url_re = regex::Regex::new(r"(?P<url>https?://[^\s]+)").unwrap();
            let comment = url_re
                .replace_all(comment.as_str(), |caps: &regex::Captures| {
                    format!("<{}>", &caps["url"])
                })
                .into_owned();

            Some(comment)
        }
    }

    let target = env::var("TARGET").unwrap();
    if target.contains("wasm") {
        panic!(
            "WASM target binding generation causes issues, don't use `bindgen` feature. generate bindings with `wasm32` feature instead and then target `wasm32-unknown-unknown` afterwards"
        );
    }

    let mut bindings = bindgen::Builder::default()
        .header("src/flecs_rust.h")
        // Only keep things that we've allowlisted rather than
        // recursively keeping nested uses around.
        .allowlist_file("src/flecs.h")
        .allowlist_file("src/flecs_rust.h")
        .allowlist_recursively(false);

    // Use appropriate ABI based on target platform
    // WASM doesn't support unwinding, so use "C" ABI
    // Other platforms can use "C-unwind" ABI

    #[cfg(feature = "wasm32")]
    {
        bindings = bindings.override_abi(bindgen::Abi::C, ".*");
        bindings = bindings.clang_arg("-DFLECS_NO_OS_API_IMPL");
    }
    #[cfg(not(feature = "wasm32"))]
    {
        bindings = bindings.override_abi(bindgen::Abi::CUnwind, ".*");
    }

    let mut bindings = bindings
        // Keep comments and keep all of them, not just doc comments.
        .generate_comments(true)
        // Prefer core::* over std::*
        .use_core()
        // Don't generate a bunch of `link_name` attributes
        .trust_clang_mangling(false)
        .clang_arg("-fparse-all-comments")
        .parse_callbacks(Box::new(CommentsCallbacks))
        .blocklist_item("FLECS_HI_COMPONENT_ID")
        .blocklist_item("FLECS_TERM_COUNT_MAX")
        .blocklist_item("ECS_PAIR")
        .blocklist_item("ECS_OVERRIDE")
        .blocklist_item("ECS_TOGGLE")
        .blocklist_item("ECS_AND")
        // We'll use `libc::FILE` instead.
        .blocklist_type("FILE")
        // These have doc comments that trigger doc tests.
        .blocklist_type("ecs_alert_desc_t")
        .blocklist_function("ecs_http_server_http_request")
        .blocklist_function("ecs_log_enable_timedelta")
        .blocklist_function("ecs_get_world_info")
        // These use va_list
        .blocklist_function("ecs_logv_")
        .blocklist_function("ecs_printv_")
        .blocklist_function("ecs_parser_errorv_")
        .blocklist_function("ecs_strbuf_vappend")
        .blocklist_function("flecs_vasprintf")
        .blocklist_function("ecs_parser_warningv_")
        .layout_tests(false)
        .raw_line("#![allow(clippy::all)]")
        .raw_line("#![allow(warnings)]")
        .raw_line("use super::*;")
        .raw_line("#[cfg(not(target_arch = \"wasm32\"))] use libc::FILE;")
        .raw_line("#[cfg(target_arch = \"wasm32\")] type FILE = core::ffi::c_void;")
        .clang_arg("-DFLECS_CUSTOM_BUILD")
        .clang_arg("-DFLECS_CPP");

    #[cfg(feature = "flecs_default_to_uncached_queries")]
    {
        bindings = bindings.clang_arg("-DFLECS_DEFAULT_TO_UNCACHED_QUERIES");
    }

    #[cfg(feature = "flecs_script_math")]
    {
        bindings = bindings.clang_arg("-DFLECS_SCRIPT_MATH");
    }

    #[cfg(feature = "flecs_perf_trace")]
    {
        bindings = bindings.clang_arg("-DFLECS_PERF_TRACE");
    }

    #[cfg(feature = "flecs_module")]
    {
        bindings = bindings.clang_arg("-DFLECS_MODULE");
    }

    #[cfg(feature = "flecs_script")]
    {
        bindings = bindings.clang_arg("-DFLECS_SCRIPT");
    }

    #[cfg(feature = "flecs_stats")]
    {
        bindings = bindings.clang_arg("-DFLECS_STATS");
    }

    #[cfg(feature = "flecs_metrics")]
    {
        bindings = bindings.clang_arg("-DFLECS_METRICS");
    }

    #[cfg(feature = "flecs_alerts")]
    {
        bindings = bindings.clang_arg("-DFLECS_ALERTS");
    }

    #[cfg(feature = "flecs_system")]
    {
        bindings = bindings.clang_arg("-DFLECS_SYSTEM");
    }

    #[cfg(feature = "flecs_pipeline")]
    {
        bindings = bindings.clang_arg("-DFLECS_PIPELINE");
    }

    #[cfg(feature = "flecs_timer")]
    {
        bindings = bindings.clang_arg("-DFLECS_TIMER");
    }

    #[cfg(feature = "flecs_meta")]
    {
        bindings = bindings.clang_arg("-DFLECS_META");
    }

    #[cfg(feature = "flecs_meta_c")]
    {
        bindings = bindings.clang_arg("-DFLECS_META_C");
    }

    #[cfg(feature = "flecs_units")]
    {
        bindings = bindings.clang_arg("-DFLECS_UNITS");
    }

    #[cfg(feature = "flecs_json")]
    {
        bindings = bindings.clang_arg("-DFLECS_JSON");
    }

    #[cfg(feature = "flecs_doc")]
    {
        bindings = bindings.clang_arg("-DFLECS_DOC");
    }

    #[cfg(feature = "flecs_log")]
    {
        bindings = bindings.clang_arg("-DFLECS_LOG");
    }

    #[cfg(feature = "flecs_app")]
    {
        bindings = bindings.clang_arg("-DFLECS_APP");
    }

    #[cfg(feature = "flecs_os_api_impl")]
    {
        #[cfg(feature = "wasm32")]
        {
            panic!("FLECS_OS_API_IMPL is not supported on wasm32");
        }
        bindings = bindings.clang_arg("-DFLECS_OS_API_IMPL");
    }

    #[cfg(not(feature = "flecs_os_api_impl"))]
    {
        bindings = bindings.clang_arg("-DFLECS_NO_OS_API_IMPL");
    }

    #[cfg(feature = "flecs_http")]
    {
        bindings = bindings.clang_arg("-DFLECS_HTTP");
    }

    #[cfg(feature = "flecs_rest")]
    {
        bindings = bindings.clang_arg("-DFLECS_REST");
    }

    #[cfg(feature = "flecs_journal")]
    {
        bindings = bindings.clang_arg("-DFLECS_JOURNAL");
    }

    #[cfg(feature = "flecs_safety_locks")]
    {
        bindings = bindings.clang_arg("-DFLECS_SAFETY_LOCKS");
    }

    let term_count_max = if cfg!(feature = "flecs_term_count_64") {
        64
    } else {
        32 // default value
    };

    let term_arg = format!("-DFLECS_TERM_COUNT_MAX={}", term_count_max);

    bindings = bindings
        .clang_arg(term_arg)
        .raw_line(format!("pub const FLECS_TERM_COUNT_MAX: u32 = {};", term_count_max).as_str());

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    bindings
        .write_to_file(crate_root.join("src/bindings.rs"))
        .unwrap();
}

/// Attempts to find wasi-libc headers using multiple detection methods
fn find_wasi_headers() -> Option<String> {
    use std::process::Command;

    // Method 1: Check environment variable
    if let Ok(path) = std::env::var("WASI_SYSROOT_INCLUDE") {
        if std::path::Path::new(&path).exists() {
            println!("cargo:rerun-if-env-changed=WASI_SYSROOT_INCLUDE");
            return Some(path);
        }
    }

    // Method 2: Try Homebrew (macOS)
    if let Ok(output) = Command::new("brew")
        .args(["--prefix", "wasi-libc"])
        .output()
    {
        if output.status.success() {
            let prefix_str = String::from_utf8_lossy(&output.stdout);
            let prefix = prefix_str.trim();
            let wasi_path = format!("{}/share/wasi-sysroot/include/wasm32-wasi", prefix);
            if std::path::Path::new(&wasi_path).exists() {
                return Some(wasi_path);
            }
        }
    }

    // Method 3: Try pkg-config (Linux)
    if let Ok(output) = Command::new("pkg-config")
        .args(["--variable=includedir", "wasi-libc"])
        .output()
    {
        if output.status.success() {
            let include_str = String::from_utf8_lossy(&output.stdout);
            let include_dir = include_str.trim();
            let wasi_path = format!("{}/wasm32-wasi", include_dir);
            if std::path::Path::new(&wasi_path).exists() {
                return Some(wasi_path);
            }
        }
    }

    // Method 4: Check common installation paths
    let common_paths = [
        // Homebrew paths (both Intel and Apple Silicon)
        "/opt/homebrew/share/wasi-sysroot/include/wasm32-wasi",
        "/usr/local/share/wasi-sysroot/include/wasm32-wasi",
        // Linux package manager paths
        "/usr/include/wasm32-wasi",
        "/usr/local/include/wasm32-wasi",
        // wasi-sdk installation paths
        "/opt/wasi-sdk/share/wasi-sysroot/include/wasm32-wasi",
        "/usr/local/wasi-sdk/share/wasi-sysroot/include/wasm32-wasi",
        // Windows paths (if using wasi-sdk)
        "C:/wasi-sdk/share/wasi-sysroot/include/wasm32-wasi",
        // Common manual installation paths
        "/opt/wasi-libc/include/wasm32-wasi",
        "/usr/local/wasi-libc/include/wasm32-wasi",
    ];

    for path in &common_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // Method 5: Try to find wasi-sdk binary and infer path
    if let Ok(output) = Command::new("which").arg("wasi-clang").output() {
        if output.status.success() {
            let clang_str = String::from_utf8_lossy(&output.stdout);
            let wasi_clang_path = clang_str.trim();
            // wasi-clang is typically in bin/, so go up and look for share/wasi-sysroot
            if let Some(parent) = std::path::Path::new(wasi_clang_path).parent() {
                if let Some(grandparent) = parent.parent() {
                    let wasi_path = grandparent.join("share/wasi-sysroot/include/wasm32-wasi");
                    if wasi_path.exists() {
                        return Some(wasi_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=src/flecs.h");
    println!("cargo:rerun-if-changed=src/flecs.c");
    println!("cargo:rerun-if-changed=src/flecs_rust.h");
    println!("cargo:rerun-if-changed=src/flecs_rust.c");
    println!("cargo:rerun-if-changed=build.rs");

    let target_is_wasm = std::env::var("TARGET")
        .map(|t| t.starts_with("wasm"))
        .unwrap_or(false);

    #[cfg(not(feature = "disable_build_c"))]
    {
        let mut build = cc::Build::new();
        build
            .file("src/flecs_rust.c")
            .warnings(true)
            .extra_warnings(true)
            .define("FLECS_CUSTOM_BUILD", None)
            .define("FLECS_CPP", None);

        if target_is_wasm {
            // Add WASM-specific stub file
            build.file("src/wasm_stubs.c");

            // Try to find wasi-libc headers using multiple methods
            let wasi_include = find_wasi_headers().unwrap_or_else(|| {
                eprintln!("Error: Could not find wasi-libc headers.");
                eprintln!("wasi-libc is required for WASM compilation. Please install it:");
                eprintln!();
                eprintln!("  macOS (Homebrew):");
                eprintln!("    brew install wasi-libc");
                eprintln!();
                eprintln!("  Ubuntu/Debian:");
                eprintln!("    sudo apt install wasi-libc-dev");
                eprintln!();
                eprintln!("  Arch Linux:");
                eprintln!("    sudo pacman -S wasi-libc");
                eprintln!();
                eprintln!("  Manual installation:");
                eprintln!("    Download wasi-sdk from https://github.com/WebAssembly/wasi-sdk/releases");
                eprintln!();
                eprintln!("  Or set the WASI_SYSROOT_INCLUDE environment variable:");
                eprintln!("    export WASI_SYSROOT_INCLUDE=/path/to/wasi-libc/include/wasm32-wasi");
                eprintln!();
                panic!("wasi-libc headers not found. See error message above for installation instructions.");
            });

            build
                .include(&wasi_include)
                .include("src") // Always include src for our custom headers
                .flag("-ffreestanding")
                .flag("-fno-exceptions")
                .flag("-fno-unwind-tables")
                .flag("-fno-asynchronous-unwind-tables")
                .flag("-fvisibility=hidden")
                // Define macros to bypass WASI platform checks
                .define("__wasi__", None)
                .define("_WASI_EMULATED_MMAN", None)
                .define("_WASI_EMULATED_SIGNAL", None)
                .define("_WASI_EMULATED_PROCESS_CLOCKS", None)
                // Provide missing safe string functions as macros
                .define("strcpy_s(dst, dsz, src)", "strcpy(dst, src)")
                .define("strcat_s(dst, dsz, src)", "strcat(dst, src)")
                .define("strncpy_s(dst, dsz, src, cnt)", "strncpy(dst, src, cnt)")
                .define(
                    "sprintf_s(dst, dsz, ...)",
                    "snprintf(dst, dsz, __VA_ARGS__)",
                )
                .define(
                    "fopen_s(pFile, filename, mode)",
                    "(*pFile = fopen(filename, mode)) ? 0 : errno",
                )
                .define("FLECS_NO_OS_API_IMPL", None);
        }

        #[cfg(feature = "flecs_perf_trace")]
        build.define("FLECS_PERF_TRACE", None);

        #[cfg(feature = "flecs_module")]
        build.define("FLECS_MODULE", None);

        #[cfg(feature = "flecs_script")]
        build.define("FLECS_SCRIPT", None);

        #[cfg(feature = "flecs_stats")]
        build.define("FLECS_STATS", None);

        #[cfg(feature = "flecs_metrics")]
        build.define("FLECS_METRICS", None);

        #[cfg(feature = "flecs_alerts")]
        build.define("FLECS_ALERTS", None);

        #[cfg(feature = "flecs_system")]
        build.define("FLECS_SYSTEM", None);

        #[cfg(feature = "flecs_pipeline")]
        build.define("FLECS_PIPELINE", None);

        #[cfg(feature = "flecs_timer")]
        build.define("FLECS_TIMER", None);

        #[cfg(feature = "flecs_meta")]
        build.define("FLECS_META", None);

        #[cfg(feature = "flecs_meta_c")]
        build.define("FLECS_META_C", None);

        #[cfg(feature = "flecs_units")]
        build.define("FLECS_UNITS", None);

        #[cfg(feature = "flecs_json")]
        build.define("FLECS_JSON", None);

        #[cfg(feature = "flecs_doc")]
        build.define("FLECS_DOC", None);

        #[cfg(feature = "flecs_log")]
        build.define("FLECS_LOG", None);

        #[cfg(feature = "flecs_app")]
        build.define("FLECS_APP", None);

        #[cfg(feature = "flecs_os_api_impl")]
        build.define("FLECS_OS_API_IMPL", None);

        #[cfg(feature = "flecs_http")]
        build.define("FLECS_HTTP", None);

        #[cfg(feature = "flecs_rest")]
        build.define("FLECS_REST", None);

        #[cfg(feature = "flecs_journal")]
        build.define("FLECS_JOURNAL", None);

        #[cfg(feature = "flecs_safety_locks")]
        build.define("FLECS_SAFETY_LOCKS", None);

        #[cfg(any(
            all(not(debug_assertions), not(feature = "force_build_debug"),),
            feature = "force_build_release"
        ))]
        {
            build
                .opt_level(3)
                .define("NDEBUG", None)
                .define("flto", None);
        }

        #[cfg(feature = "use_os_alloc")]
        {
            build.define("FLECS_USE_OS_ALLOC", None);
        }

        #[cfg(feature = "flecs_force_enable_ecs_asserts")]
        {
            build.define("FLECS_KEEP_ASSERTS", None);
        }

        let term_count_max = if cfg!(feature = "flecs_term_count_64") {
            64
        } else {
            32 // default value
        };

        build.define(
            "FLECS_TERM_COUNT_MAX",
            Some(term_count_max.to_string().as_str()),
        );

        build.compile("flecs");

        #[cfg(all(not(target_family = "wasm"), feature = "bindgen"))]
        generate_bindings();
    }
}
