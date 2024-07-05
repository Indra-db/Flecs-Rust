//! manual bindings to prevent warnings / errors from bindgen in build or tests

#![allow(clippy::all)]
#![allow(warnings)]

use super::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
//#[cfg(feature = "flecs_alerts")] //TODO flecs ecs_alert_init not properly defined in flecs c api.
pub struct ecs_alert_desc_t {
    pub _canary: i32,
    #[doc = "Entity associated with alert"]
    pub entity: ecs_entity_t,
    #[doc = "Alert query. An alert will be created for each entity that matches the\n specified query. The query must have at least one term that uses the\n $this variable (default)."]
    pub filter: ecs_query_desc_t,
    /// Template for alert message. This string is used to generate the alert
    /// message and may refer to variables in the query result. The format for
    /// the template expressions is as specified by ecs_interpolate_string().
    ///
    /// # Examples
    ///
    /// ```text
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
    /// ```text
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
    /// ```text
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

/// Type that contains information about the world.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WorldInfo {
    /// Last issued component entity id.
    pub last_component_id: ecs_entity_t,
    /// First allowed entity id.
    pub min_id: ecs_entity_t,
    /// Last allowed entity id.
    pub max_id: ecs_entity_t,
    /// Raw delta time (no time scaling).
    pub delta_time_raw: f32,
    /// Time passed to or computed by `ecs_progress`.
    pub delta_time: f32,
    /// Time scale applied to `delta_time`.
    pub time_scale: f32,
    /// Target fps.
    pub target_fps: f32,
    /// Total time spent processing a frame.
    pub frame_time_total: f32,
    /// Total time spent in systems.
    pub system_time_total: f32,
    /// Total time spent notifying observers.
    pub emit_time_total: f32,
    /// Total time spent in merges.
    pub merge_time_total: f32,
    /// Time elapsed in simulation.
    pub world_time_total: f32,
    /// Time elapsed in simulation (no scaling).
    pub world_time_total_raw: f32,
    /// Time spent on query rematching.
    pub rematch_time_total: f32,
    /// Total number of frames.
    pub frame_count_total: i64,
    /// Total number of merges.
    pub merge_count_total: i64,
    /// Total number of rematches.
    pub rematch_count_total: i64,
    /// Total number of times a new id was created.
    pub id_create_total: i64,
    /// Total number of times an id was deleted.
    pub id_delete_total: i64,
    /// Total number of times a table was created.
    pub table_create_total: i64,
    /// Total number of times a table was deleted.
    pub table_delete_total: i64,
    /// Total number of pipeline builds.
    pub pipeline_build_count_total: i64,
    /// Total number of systems ran in last frame.
    pub systems_ran_frame: i64,
    /// Total number of times observer was invoked.
    pub observers_ran_frame: i64,
    /// Number of tag (no data) ids in the world.
    pub tag_id_count: i32,
    /// Number of component (data) ids in the world.
    pub component_id_count: i32,
    /// Number of pair ids in the world.
    pub pair_id_count: i32,
    /// Number of tables.
    pub table_count: i32,
    /// Number of tables without entities.
    pub empty_table_count: i32,
    pub cmd: WorldInfoCmd,
    /// Value set by `ecs_set_name_prefix()`. Used
    /// to remove library prefixes of symbol names (such as `Ecs`, `ecs_`) when
    /// registering them as names.
    pub name_prefix: *const std::os::raw::c_char,
}

/// Command counts.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WorldInfoCmd {
    /// Add commands processed.
    pub add_count: i64,
    /// Remove commands processed.
    pub remove_count: i64,
    /// Delete commands processed.
    pub delete_count: i64,
    /// Clear commands processed.
    pub clear_count: i64,
    /// Set commands processed.
    pub set_count: i64,
    /// Ensure/emplace commands processed.
    pub ensure_count: i64,
    /// Modified commands processed.
    pub modified_count: i64,
    /// Commands discarded, happens when entity is no longer alive when running the command.
    pub discard_count: i64,
    /// Enqueued custom events.
    pub event_count: i64,
    /// Other commands processed.
    pub other_count: i64,
    /// Entities for which commands were batched.
    pub batched_entity_count: i64,
    /// Commands batched.
    pub batched_command_count: i64,
}

extern "C" {
    #[doc = "Get world info.\n\n @param world The world.\n @return Pointer to the world info. Valid for as long as the world exists."]
    pub fn ecs_get_world_info(world: *const ecs_world_t) -> *const WorldInfo;
}

#[test]
fn compile_test_check_if_any_ecs_world_info_fields_changed() {
    let info = ecs_world_info_t {
        last_component_id: 0,
        min_id: 0,
        max_id: 0,
        delta_time_raw: 0.0,
        delta_time: 0.0,
        time_scale: 0.0,
        target_fps: 0.0,
        frame_time_total: 0.0,
        system_time_total: 0.0,
        emit_time_total: 0.0,
        merge_time_total: 0.0,
        world_time_total: 0.0,
        world_time_total_raw: 0.0,
        rematch_time_total: 0.0,
        frame_count_total: 0,
        merge_count_total: 0,
        rematch_count_total: 0,
        id_create_total: 0,
        id_delete_total: 0,
        table_create_total: 0,
        table_delete_total: 0,
        pipeline_build_count_total: 0,
        systems_ran_frame: 0,
        observers_ran_frame: 0,
        tag_id_count: 0,
        component_id_count: 0,
        pair_id_count: 0,
        table_count: 0,
        empty_table_count: 0,
        cmd: ecs_world_info_t__bindgen_ty_1 {
            add_count: 0,
            remove_count: 0,
            delete_count: 0,
            clear_count: 0,
            set_count: 0,
            ensure_count: 0,
            modified_count: 0,
            discard_count: 0,
            event_count: 0,
            other_count: 0,
            batched_entity_count: 0,
            batched_command_count: 0,
        },
        name_prefix: std::ptr::null(),
    };
}

unsafe impl Send for EcsWorldStats {}

unsafe impl Sync for EcsWorldStats {}

unsafe impl Send for EcsWorldSummary {}

unsafe impl Sync for EcsWorldSummary {}

unsafe impl Send for EcsPipelineStats {}

unsafe impl Sync for EcsPipelineStats {}

unsafe impl Send for EcsSystemStats {}

unsafe impl Sync for EcsSystemStats {}
