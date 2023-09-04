#![allow(clippy::all)]
#![allow(warnings)]

#[cfg(feature = "generate_bindings")]
fn generate_bindings() {
    use std::env;
    use std::path::PathBuf;

    let mut bindings = bindgen::Builder::default()
        .header("src/core/c_binding/flecs.h")
        .generate_comments(false)
        .layout_tests(false)
        .raw_line("#![allow(clippy::all)]")
        .raw_line("#![allow(warnings)]")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    // export comments from flecs source
    let bindings = bindings
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        // this yields two small comments
        .clang_arg("-fretain-comments-from-system-headers")
        .parse_callbacks(Box::new(CommentsCallbacks));

    let bindings = bindings
        .allowlist_file("src/core/c_binding/flecs.c")
        .allowlist_file("src/core/c_binding/flecs.h")
        .generate()
        .expect("Unable to generate bindings");

    let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    bindings
        .write_to_file(crate_root.join("src/core/c_binding/bindings.rs"))
        .unwrap();
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=src/core/c_binding/flecs.h");
    println!("cargo:rerun-if-changed=src/core/c_binding/flecs.c");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "generate_bindings")]
    generate_bindings();

    // Compile flecs C right into our Rust crate
    cc::Build::new()
        .warnings(true)
        .extra_warnings(true)
        .define("NDEBUG", None)
        .file("src/core/c_binding/flecs.c")
        .compile("flecs");
}

#[cfg(feature = "generate_bindings")]
#[derive(Debug)]
struct CommentsCallbacks;

#[cfg(feature = "generate_bindings")]
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
