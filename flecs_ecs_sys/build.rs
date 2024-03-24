#![allow(clippy::all)]
#![allow(warnings)]

#[cfg(feature = "regenerate_binding")]
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
        .header("src/flecs.h")
        // Only keep things that we've allowlisted rather than
        // recursively keeping nested uses around.
        .allowlist_file("src/flecs.h")
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
        .write_to_file(crate_root.join("src/bindings.rs"))
        .unwrap();
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the sources change
    println!("cargo:rerun-if-changed=src/flecs.h");
    println!("cargo:rerun-if-changed=src/flecs.c");
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "regenerate_binding")]
    generate_bindings();

    #[cfg(not(feature = "disable_build_c_library"))]
    {
        let mut build = cc::Build::new();

        build
            .compiler("clang")
            .file("src/flecs.c")
            .warnings(true)
            .extra_warnings(true);

        #[cfg(feature = "flecs_module")]
        build.define("FLECS_MODULE", None);

        #[cfg(feature = "flecs_parser")]
        build.define("FLECS_PARSER", None);

        #[cfg(feature = "flecs_plecs")]
        build.define("FLECS_PLECS", None);

        #[cfg(feature = "flecs_rules")]
        build.define("FLECS_RULES", None);

        #[cfg(feature = "flecs_snapshot")]
        build.define("FLECS_SNAPSHOT", None);

        #[cfg(feature = "flecs_stats")]
        build.define("FLECS_STATS", None);

        #[cfg(feature = "flecs_monitor")]
        build.define("FLECS_MONITOR", None);

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

        #[cfg(feature = "flecs_expr")]
        build.define("FLECS_EXPR", None);

        #[cfg(feature = "flecs_json")]
        build.define("FLECS_JSON", None);

        #[cfg(feature = "flecs_doc")]
        build.define("FLECS_DOC", None);

        #[cfg(feature = "flecs_coredoc")]
        build.define("FLECS_COREDOC", None);

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

        #[cfg(not(feature = "build_debug"))]
        {
            build.opt_level(3).define("NDEBUG", None).compile("flecs");
        }

        build.compile("flecs");
    }
}
