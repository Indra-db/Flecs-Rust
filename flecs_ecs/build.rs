#![allow(clippy::all)]
#![allow(warnings)]

#[cfg(feature = "flecs_generate_bindings")]
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

    let mut bindings = bindgen::Builder::default()
        .header("src/sys/flecs.h")
        // Only keep things that we've allowlisted rather than
        // recursively keeping nested uses around.
        .allowlist_file("src/sys/flecs.h")
        .allowlist_recursively(false)
        // Keep comments and keep all of them, not just doc comments.
        .generate_comments(true)
        .clang_arg("-fparse-all-comments")
        .parse_callbacks(Box::new(CommentsCallbacks))
        .blocklist_item("FLECS_HI_COMPONENT_ID")
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
        // These use va_list
        .blocklist_function("ecs_logv_")
        .blocklist_function("ecs_printv_")
        .blocklist_function("ecs_parser_errorv_")
        .blocklist_function("ecs_strbuf_vappend")
        .blocklist_function("ecs_vasprintf")
        .layout_tests(false)
        .raw_line("#![allow(clippy::all)]")
        .raw_line("#![allow(warnings)]")
        .raw_line("use super::*;")
        .raw_line("use libc::FILE;");

    let bindings = bindings.generate().expect("Unable to generate bindings");

    let crate_root: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();
    bindings
        .write_to_file(crate_root.join("src/sys/bindings.rs"))
        .unwrap();
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=src/sys/flecs.h");
    println!("cargo:rerun-if-changed=src/sys/flecs.c");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "flecs_generate_bindings")]
    generate_bindings();

    #[cfg(not(feature = "flecs_disable_build_c_library"))]
    // Compile flecs
    cc::Build::new()
        //.compiler("clang")
        //.opt_level(3)
        //.shared_flag(true)
        .warnings(true)
        .extra_warnings(true)
        //.define("NDEBUG", None)
        .file("src/sys/flecs.c")
        .compile("flecs");
}
