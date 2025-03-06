#[cfg(feature = "flecs_app")]
use crate::ecs_app_desc_t;
use crate::{
    EcsComponent, EcsOpaque, EcsPoly, ecs_alert_desc_t, ecs_alert_severity_filter_t,
    ecs_entity_desc_t, ecs_event_desc_t, ecs_header_t, ecs_metric_desc_t, ecs_observer_desc_t,
    ecs_query_desc_t, ecs_term_ref_t, ecs_term_t, ecs_type_hooks_t, ecs_type_t,
};

#[cfg(feature = "flecs_system")]
use crate::{EcsTickSource, ecs_system_desc_t};

#[cfg(feature = "flecs_pipeline")]
use crate::ecs_pipeline_desc_t;

impl Default for ecs_type_t {
    fn default() -> Self {
        Self {
            array: core::ptr::null_mut(),
            count: Default::default(),
        }
    }
}

impl Default for ecs_term_ref_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: core::ptr::null_mut(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
impl Default for ecs_term_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            src: Default::default(),
            first: Default::default(),
            second: Default::default(),
            inout: Default::default(),
            oper: Default::default(),
            field_index: Default::default(),
            trav: Default::default(),
            flags_: Default::default(),
        }
    }
}

impl Default for ecs_query_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            order_by: Default::default(),
            group_by: Default::default(),
            on_group_create: Default::default(),
            on_group_delete: Default::default(),
            group_by_ctx: core::ptr::null_mut(),
            group_by_ctx_free: Default::default(),
            ctx: core::ptr::null_mut(),
            binding_ctx: core::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
            expr: core::ptr::null(),
            cache_kind: Default::default(),
            flags: Default::default(),
            order_by_callback: Default::default(),
            order_by_table_callback: Default::default(),
            group_by_callback: Default::default(),
            entity: Default::default(),
            #[cfg(not(feature = "flecs_term_count_64"))]
            terms: Default::default(),
            #[cfg(feature = "flecs_term_count_64")]
            terms: [Default::default(); 64],
        }
    }
}

impl Default for ecs_observer_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            events: Default::default(),
            yield_existing: Default::default(),
            callback: Default::default(),
            run: Default::default(),
            ctx: core::ptr::null_mut(),
            callback_ctx: core::ptr::null_mut(),
            ctx_free: Default::default(),
            callback_ctx_free: Default::default(),
            observable: core::ptr::null_mut(),
            last_event_id: core::ptr::null_mut(),
            term_index_: Default::default(),
            query: Default::default(),
            run_ctx: core::ptr::null_mut(),
            run_ctx_free: Default::default(),
            flags_: Default::default(),
        }
    }
}

impl Default for ecs_header_t {
    fn default() -> Self {
        Self {
            magic: Default::default(),
            type_: Default::default(),
            mixins: core::ptr::null_mut(),
            refcount: Default::default(),
        }
    }
}

impl Default for ecs_entity_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            id: Default::default(),
            name: core::ptr::null(),
            sep: core::ptr::null(),
            root_sep: core::ptr::null(),
            symbol: core::ptr::null(),
            use_low_id: Default::default(),
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
            parent: Default::default(),
        }
    }
}

impl Default for ecs_event_desc_t {
    fn default() -> Self {
        Self {
            event: Default::default(),
            ids: core::ptr::null(),
            table: core::ptr::null_mut(),
            other_table: core::ptr::null_mut(),
            offset: Default::default(),
            count: Default::default(),
            entity: Default::default(),
            param: core::ptr::null_mut(),
            observable: core::ptr::null_mut(),
            flags: Default::default(),
            const_param: core::ptr::null(),
        }
    }
}

#[cfg(feature = "flecs_system")]
impl Default for ecs_system_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            query: Default::default(),
            run: Default::default(),
            callback: Default::default(),
            ctx: core::ptr::null_mut(),
            callback_ctx: core::ptr::null_mut(),
            ctx_free: Default::default(),
            interval: Default::default(),
            rate: Default::default(),
            tick_source: Default::default(),
            multi_threaded: Default::default(),
            immediate: Default::default(),
            callback_ctx_free: Default::default(),
            run_ctx: core::ptr::null_mut(),
            run_ctx_free: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
#[cfg(feature = "flecs_pipeline")]
impl Default for ecs_pipeline_desc_t {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            query: Default::default(),
        }
    }
}

#[cfg(feature = "flecs_app")]
impl Default for ecs_app_desc_t {
    fn default() -> Self {
        Self {
            target_fps: Default::default(),
            delta_time: Default::default(),
            threads: Default::default(),
            frames: Default::default(),
            enable_rest: Default::default(),
            enable_stats: Default::default(),
            port: Default::default(),
            init: Default::default(),
            ctx: core::ptr::null_mut(),
        }
    }
}

impl Default for ecs_alert_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            query: Default::default(),
            message: core::ptr::null(),
            doc_name: core::ptr::null(),
            brief: core::ptr::null(),
            severity: Default::default(),
            severity_filters: Default::default(),
            retain_period: Default::default(),
            member: Default::default(),
            id: Default::default(),
            var: core::ptr::null(),
        }
    }
}

impl Default for ecs_metric_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            member: Default::default(),
            dotmember: core::ptr::null(),
            id: Default::default(),
            targets: Default::default(),
            kind: Default::default(),
            brief: core::ptr::null(),
        }
    }
}

impl Default for ecs_alert_severity_filter_t {
    fn default() -> Self {
        Self {
            severity: Default::default(),
            with: Default::default(),
            var: core::ptr::null(),
            _var_index: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
impl Default for EcsOpaque {
    fn default() -> Self {
        Self {
            as_type: Default::default(),
            serialize: Default::default(),
            assign_bool: Default::default(),
            assign_char: Default::default(),
            assign_int: Default::default(),
            assign_uint: Default::default(),
            assign_float: Default::default(),
            assign_string: Default::default(),
            assign_entity: Default::default(),
            assign_null: Default::default(),
            clear: Default::default(),
            ensure_element: Default::default(),
            ensure_member: Default::default(),
            count: Default::default(),
            resize: Default::default(),
            assign_id: Default::default(),
        }
    }
}

#[cfg(feature = "flecs_system")]
impl Default for EcsTickSource {
    fn default() -> Self {
        Self {
            tick: false,
            time_elapsed: 0.0,
        }
    }
}

impl Default for ecs_type_hooks_t {
    fn default() -> Self {
        ecs_type_hooks_t {
            ctor: None,
            dtor: None,
            copy: None,
            move_: None,
            copy_ctor: None,
            move_ctor: None,
            ctor_move_dtor: None,
            move_dtor: None,
            on_add: None,
            on_set: None,
            on_remove: None,
            ctx: core::ptr::null_mut(),
            binding_ctx: core::ptr::null_mut(),
            ctx_free: None,
            binding_ctx_free: None,
            lifecycle_ctx: core::ptr::null_mut(),
            lifecycle_ctx_free: None,
            flags: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for EcsComponent {
    fn default() -> Self {
        Self {
            size: Default::default(),
            alignment: Default::default(),
        }
    }
}

impl Default for EcsPoly {
    fn default() -> Self {
        Self {
            poly: core::ptr::null_mut(),
        }
    }
}

impl Default for crate::ecs_iter_to_json_desc_t {
    fn default() -> Self {
        Self {
            serialize_builtin: Default::default(),
            serialize_entity_ids: Default::default(),
            serialize_values: Default::default(),
            serialize_doc: Default::default(),
            serialize_full_paths: Default::default(),
            serialize_fields: Default::default(),
            serialize_inherited: Default::default(),
            serialize_table: Default::default(),
            serialize_type_info: Default::default(),
            serialize_field_info: Default::default(),
            serialize_query_info: Default::default(),
            serialize_query_plan: Default::default(),
            serialize_query_profile: Default::default(),
            dont_serialize_results: Default::default(),
            serialize_alerts: Default::default(),
            serialize_refs: Default::default(),
            serialize_matches: Default::default(),
            query: core::ptr::null_mut(),
        }
    }
}

unsafe impl Sync for crate::EcsIdentifier {}
unsafe impl Send for crate::EcsIdentifier {}
unsafe impl Send for crate::EcsPoly {}
unsafe impl Sync for crate::EcsPoly {}
