use std::env;

fn main() {
    // Link with the libc.a file for WASM target (but not for wasm-bindgen builds)
    if std::env::var("TARGET").unwrap_or_default() == "wasm32-unknown-unknown" {
        // Check if this is a wasm-bindgen build
        let is_bindgen_build = std::env::var("CARGO_FEATURE_WASM_BINDGEN").is_ok();
        
        if !is_bindgen_build {
            // Get the current directory
            let current_dir = env::current_dir().unwrap();

            // Tell cargo where to find our libc.a file
            println!("cargo:rustc-link-search=native={}", current_dir.display());

            // Link with libc.a using whole-archive to ensure all symbols are included
            println!("cargo:rustc-link-arg=--whole-archive");
            println!("cargo:rustc-link-arg={}/libc.a", current_dir.display());
            println!("cargo:rustc-link-arg=--no-whole-archive");

            // Add wasm-ld specific flags
            println!("cargo:rustc-link-arg=--allow-undefined");
            println!("cargo:rustc-link-arg=--no-entry");

            println!("cargo:rerun-if-changed=libc.a");
        }
    }
}
