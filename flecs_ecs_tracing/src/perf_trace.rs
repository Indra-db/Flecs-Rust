use crate::{metadata, util::*};
use core::ffi;
use hashbrown::HashMap;
use std::{
    borrow::Cow,
    sync::{LazyLock, OnceLock, RwLock},
};
use tracing_core::{span, Dispatch, Level, Metadata};

/// Log level for performance tracing spans
pub(crate) static SPAN_LEVEL: OnceLock<Level> = OnceLock::new();

/// Registry of dynamic tracing spans generated from Flecs
static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(Default::default);

/// Registry of dynamic tracing spans generated from Flecs
struct Registry {
    spans: HashMap<String, DynamicSpan>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            spans: HashMap::with_capacity(256),
        }
    }
}

#[derive(Clone)]
struct DynamicSpan {
    metadata: &'static Metadata<'static>,
    span: tracing_core::span::Id,
}

pub(crate) fn init() {
    LazyLock::force(&REGISTRY);
}

fn ensure_span(
    dispatch: &Dispatch,
    filename: Option<Cow<'_, str>>,
    line: usize,
    name: Cow<'_, str>,
) -> (DynamicSpan, bool) {
    let mut registry = REGISTRY.write().unwrap();

    // Double-checked locking (maybe another thread added this span between read unlock and write lock)
    if let Some(existing) = registry.spans.get(name.as_ref()) {
        return (existing.clone(), false);
    }

    // "C++-style" names allow Tracy UI to auto-collapse names in short spans
    // This conversion is done late so that the fast path can avoid allocating;
    // the unmodified name is used as the hashmap key in the registry.
    let meta_name = name.replace(".", "::");

    // Don't bother caching metadata for these; there's (currently) a 1-to-1 correspondence between spans and metadata.
    let (metadata, _) = metadata::MetadataRequest {
        kind: metadata::MetadataKind::Span,
        name: Cow::Owned(meta_name),
        filename,
        line: line.try_into().ok(),
        level: *SPAN_LEVEL.get().unwrap_or(&Level::INFO),
    }
    .register(dispatch);

    // Flecs doesn't send any parameters (yet)
    let values = metadata.fields().value_set(&[]);
    let attributes = span::Attributes::new(metadata, &values);

    // Tell `tracing` subscribers about the new span
    let span = dispatch.new_span(&attributes);

    let ret = DynamicSpan { metadata, span };
    registry.spans.insert(name.into_owned(), ret.clone());

    (ret, true)
}

fn get_span(
    dispatch: &Dispatch,
    filename: Option<Cow<'_, str>>,
    line: usize,
    name: Cow<'_, str>,
) -> (DynamicSpan, bool) {
    if let Some(existing) = REGISTRY.read().unwrap().spans.get(name.as_ref()) {
        // existing span - fast path
        return (existing.clone(), false);
    }

    // new span - slow path
    ensure_span(dispatch, filename, line, name)
}

pub(crate) unsafe extern "C-unwind" fn perf_trace_push(
    c_filename: *const ffi::c_char,
    line: usize,
    c_name: *const ffi::c_char,
) {
    tracing_core::dispatcher::get_default(|dispatch| {
        let filename = flecs_str(c_filename);
        let name = flecs_str(c_name).unwrap_or(Cow::Borrowed("<unknown>"));

        let (span, _new) = get_span(dispatch, filename, line, name);
        if dispatch.enabled(span.metadata) {
            dispatch.enter(&span.span);
        }
    });
}

pub(crate) unsafe extern "C-unwind" fn perf_trace_pop(
    c_filename: *const ffi::c_char,
    line: usize,
    c_name: *const ffi::c_char,
) {
    tracing_core::dispatcher::get_default(|dispatch| {
        let filename = flecs_str(c_filename);
        let name = flecs_str(c_name).unwrap_or(Cow::Borrowed("<unknown>"));

        let (span, new) = get_span(dispatch, filename, line, name);
        debug_assert!(
            !new,
            "span being popped must already exist name={:?} line={line} filename={:?}",
            flecs_str(c_filename),
            flecs_str(c_name)
        );
        if dispatch.enabled(span.metadata) {
            dispatch.exit(&span.span);
        }
    });
}
