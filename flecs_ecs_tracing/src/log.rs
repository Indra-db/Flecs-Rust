use std::borrow::Cow;

use crate::{metadata, util::*};
use tracing_core::{Dispatch, Metadata};

fn ensure_event_meta(
    dispatch: &Dispatch,
    request: metadata::MetadataRequest<'_>,
) -> &'static Metadata<'static> {
    // Double-checked locking
    {
        let registry = metadata::REGISTRY.read().unwrap();
        if let Some(existing) = registry.metadata.get(&request) {
            return existing;
        }
    }

    let mut registry = metadata::REGISTRY.write().unwrap();

    if let Some(existing) = registry.metadata.get(&request) {
        return existing;
    }

    let (metadata, request) = request.register(dispatch);
    registry.metadata.insert(request, metadata);

    metadata
}

fn get_event_meta(
    dispatch: &Dispatch,
    filename: Option<Cow<'_, str>>,
    line: i32,
    level: tracing_core::metadata::Level,
) -> &'static Metadata<'static> {
    let request = metadata::MetadataRequest {
        kind: metadata::MetadataKind::Event,
        name: Cow::Borrowed("log"),
        filename,
        line: line.try_into().ok(),
        level,
    };

    if let Some(existing) = metadata::REGISTRY.read().unwrap().metadata.get(&request) {
        // existing metadata - fast path
        return existing;
    }

    // new metadata - slow path
    ensure_event_meta(dispatch, request)
}

pub(crate) unsafe extern "C-unwind" fn log_to_tracing(
    level: i32,
    c_file: *const i8,
    line: i32,
    c_msg: *const i8,
) {
    let level = match level {
        -4 | -3 => tracing_core::metadata::Level::ERROR,
        -2 => tracing_core::metadata::Level::WARN,
        0 => tracing_core::metadata::Level::INFO,
        1..=3 => tracing_core::metadata::Level::DEBUG,
        _ => tracing_core::metadata::Level::TRACE,
    };

    if level < tracing::level_filters::STATIC_MAX_LEVEL {
        // Early out due to static level constraint
        return;
    }

    tracing_core::dispatcher::get_default(|dispatch| {
        let file = flecs_str(c_file);
        let msg = flecs_str(c_msg).unwrap_or(Cow::Borrowed(""));

        let meta = get_event_meta(dispatch, file, line, level);
        if dispatch.enabled(meta) {
            let message_field = meta.fields().iter().next().expect("FieldSet corrupted");
            tracing_core::Event::dispatch(
                meta,
                &meta.fields().value_set(&[(
                    &message_field,
                    Some(&(format_args!("{msg}")) as &dyn tracing_core::field::Value),
                )]),
            );
        }
    });
}
