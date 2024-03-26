//! manual bindings to prevent warnings / errors from bindgen in build or tests

#![allow(clippy::all)]
#![allow(warnings)]

use super::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[cfg(feature = "flecs_alerts")]
pub struct ecs_alert_desc_t {
    pub _canary: i32,
    #[doc = "Entity associated with alert"]
    pub entity: ecs_entity_t,
    #[doc = "Alert query. An alert will be created for each entity that matches the\n specified query. The query must have at least one term that uses the\n $this variable (default)."]
    pub filter: ecs_filter_desc_t,
    /// Template for alert message. This string is used to generate the alert
    /// message and may refer to variables in the query result. The format for
    /// the template expressions is as specified by ecs_interpolate_string().
    ///
    /// # Examples
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// "$this has Position but not Velocity"
    /// "$this has a parent entity $parent without Position"
    /// ```
    pub message: *const ::std::os::raw::c_char,
    #[doc = "User friendly name. Will only be set if FLECS_DOC addon is enabled."]
    pub doc_name: *const ::std::os::raw::c_char,
    #[doc = "Description of alert. Will only be set if FLECS_DOC addon is enabled"]
    pub brief: *const ::std::os::raw::c_char,
    #[doc = "Metric kind. Must be EcsAlertInfo, EcsAlertWarning, EcsAlertError or\n EcsAlertCritical. Defaults to EcsAlertError."]
    pub severity: ecs_entity_t,
    #[doc = "Severity filters can be used to assign different severities to the same\n alert. This prevents having to create multiple alerts, and allows\n entities to transition between severities without resetting the\n alert duration (optional)."]
    pub severity_filters: [ecs_alert_severity_filter_t; 4usize],
    #[doc = "The retain period specifies how long an alert must be inactive before it\n is cleared. This makes it easier to track noisy alerts. While an alert is\n inactive its duration won't increase.\n When the retain period is 0, the alert will clear immediately after it no\n longer matches the alert query."]
    pub retain_period: f32,
    #[doc = "Alert when member value is out of range. Uses the warning/error ranges\n assigned to the member in the MemberRanges component (optional)."]
    pub member: ecs_entity_t,
    #[doc = "(Component) id of member to monitor. If left to 0 this will be set to\n the parent entity of the member (optional)."]
    pub id: ecs_id_t,
    #[doc = "Variable from which to fetch the member (optional). When left to NULL\n 'id' will be obtained from $this."]
    pub var: *const ::std::os::raw::c_char,
}

extern "C" {
    /// Enable/disable logging time since last log.
    ///
    /// By default, deltatime is disabled. Note that enabling timestamps introduces
    /// overhead as the logging code will need to obtain the current time.
    ///
    /// When enabled, this logs the amount of time in seconds passed since the last
    /// log, when this amount is non-zero. The format is a '+' character followed by
    /// the number of seconds:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// +1 trace: log message
    /// ```
    ///
    /// # Parameters
    ///
    /// * `enabled` - Whether to enable tracing with timestamps.
    ///
    /// # Returns
    ///
    /// Previous timestamp setting.
    pub fn ecs_log_enable_timedelta(enabled: bool) -> bool;
}

extern "C" {
    /// Emulate a request.
    ///
    /// The request string must be a valid HTTP request. A minimal example:
    ///
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// GET /entity/flecs/core/World?label=true HTTP/1.1
    /// ```
    ///
    /// # Parameters
    ///
    /// * `srv` - The server.
    /// * `req` - The request.
    /// * `len` - The length of the request (optional).
    ///
    /// # Returns
    ///
    /// The reply.
    #[cfg(feature = "flecs_http")]
    pub fn ecs_http_server_http_request(
        srv: *mut ecs_http_server_t,
        req: *const ::std::os::raw::c_char,
        len: ecs_size_t,
        reply_out: *mut ecs_http_reply_t,
    ) -> ::std::os::raw::c_int;
}
