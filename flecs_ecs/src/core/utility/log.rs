use crate::core::c_binding::bindings::{
    ecs_log_enable_colors, ecs_log_enable_timedelta, ecs_log_enable_timestamp, ecs_log_get_level,
    ecs_log_set_level,
};

/// Sets the logging level to the specified value.
///
/// ### Arguments
///
/// * `level` - An integer representing the logging level.
pub fn set_log_level(level: i32) {
    unsafe {
        ecs_log_set_level(level);
    }
}

/// Returns the current logging level.
///
/// ### Returns
///
/// An integer representing the current logging level.
pub fn get_log_level() -> i32 {
    unsafe { ecs_log_get_level() }
}

/// Enables or disables colors in logging.
///
/// ### Arguments
///
/// * `enabled` - A boolean value indicating whether to enable or disable colors.
pub fn enable_color_logging(enabled: bool) {
    unsafe {
        ecs_log_enable_colors(enabled);
    }
}

/// Enables or disables timestamps in logging.
///
/// ### Arguments
///
/// * `enabled` - A boolean value indicating whether to enable or disable timestamps.
pub fn enable_timestamp_logging(enabled: bool) {
    unsafe {
        ecs_log_enable_timestamp(enabled);
    }
}

/// Enables or disables time delta in logging.
///
/// ### Arguments
///
/// * `enabled` - A boolean value indicating whether to enable or disable time delta.
pub fn enable_timedelta_logging(enabled: bool) {
    unsafe {
        ecs_log_enable_timedelta(enabled);
    }
}
