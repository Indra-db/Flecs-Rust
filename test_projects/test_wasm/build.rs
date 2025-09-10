fn main() {
    // Link configuration for WASM target
    // if std::env::var("TARGET").unwrap_or_default() == "wasm32-unknown-unknown" {
    //     // Add wasm-ld specific flags
    //     println!("cargo:rustc-link-arg=--allow-undefined");
    //     println!("cargo:rustc-link-arg=--no-entry");

    //     // The wasm32_musl_libc dependency will handle libc linking automatically
    //     println!("cargo:rerun-if-changed=build.rs");
    // }
}
