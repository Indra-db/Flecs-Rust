//! Integrates Flecs ECS logging and performance tracing into the `tracing` ecosystem.

use flecs_ecs::prelude::*;

mod log;
mod metadata;
#[cfg(feature = "perf_trace")]
mod perf_trace;
mod util;

/// Send Flecs internal logging to `tracing` subscribers instead.
///
/// Note that the application will need to set up those subscribers; by default, logs will go nowhere.
///
/// This must be called before the first [`World`] is created anywhere in the process;
/// see [`ecs_os_api::add_init_hook`] for details on those limitations.
///
/// # Example
/// ```standalone
/// use flecs_ecs::prelude::*;
/// use flecs_ecs_tracing::log_to_tracing;
/// use tracing_subscriber::prelude::*;
///
/// tracing_subscriber::registry()
///     // Send logs to stdout
///     .with(
///         tracing_subscriber::fmt::layer()
///             // By default, hide anything below WARN log level from stdout
///             .with_filter(
///                 tracing_subscriber::EnvFilter::builder()
///                     .with_default_directive(tracing::level_filters::LevelFilter::WARN.into())
///                     .from_env_lossy(),
///             ),
///     )
///     .init();
///
/// log_to_tracing();
/// let world = World::new();
/// ```
pub fn log_to_tracing() {
    // Ensure that the registry is initialized now
    metadata::init();

    ecs_os_api::add_init_hook(Box::new(|api| {
        api.log_ = Some(log::log_to_tracing);
    }));
}

#[cfg(feature = "perf_trace")]
/// Send Flecs performance traces to `tracing` subscribers.
///
/// This must be called before the first [`World`] is created anywhere in the process;
/// see [`ecs_os_api::add_init_hook`] for details on those limitations.
///
/// # Example
/// ```standalone
/// use flecs_ecs::prelude::*;
/// use flecs_ecs_tracing::{log_to_tracing, perf_trace_to_tracing};
/// use tracing_subscriber::prelude::*;
///
/// tracing_subscriber::registry()
///     // Send logs to stdout
///     .with(
///         tracing_subscriber::fmt::layer()
///             // By default, hide anything below WARN log level from stdout
///             .with_filter(
///                 tracing_subscriber::EnvFilter::builder()
///                     .with_default_directive(tracing::level_filters::LevelFilter::WARN.into())
///                     .from_env_lossy(),
///             ),
///     )
///     // Send logs and performance tracing data to the Tracy profiler
///     .with(
///         tracing_tracy::TracyLayer::default()
///             .with_filter(tracing::level_filters::LevelFilter::INFO),
///     )
///     .init();
///
/// log_to_tracing(); // optional, but recommended
/// perf_trace_to_tracing(tracing::Level::INFO);
///
/// let world = World::new();
/// ```
pub fn perf_trace_to_tracing(span_level: tracing_core::Level) {
    // Ensure that the registry is initialized now
    perf_trace::init();

    if span_level < tracing::level_filters::STATIC_MAX_LEVEL {
        // Early out due to static level constraint
        return;
    }

    // Set the log level of performance tracing spans
    perf_trace::SPAN_LEVEL.get_or_init(|| span_level);

    ecs_os_api::add_init_hook(Box::new(|api| {
        api.perf_trace_push_ = Some(perf_trace::perf_trace_push);
        api.perf_trace_pop_ = Some(perf_trace::perf_trace_pop);
    }));
}
