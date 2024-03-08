use crate::{
    ecs_app_desc_t, ecs_entity_desc_t, ecs_event_desc_t, ecs_filter_desc_t, ecs_filter_t,
    ecs_filter_t_magic, ecs_header_t, ecs_iterable_t, ecs_observer_desc_t, ecs_pipeline_desc_t,
    ecs_query_desc_t, ecs_system_desc_t, ecs_term_id_t, ecs_term_t, ecs_type_hooks_t, ecs_type_t,
    EcsComponent, EcsOpaque, EcsPoly, EcsTickSource, ECS_FILTER_INIT,
};

impl Default for ecs_type_t {
    fn default() -> Self {
        Self {
            array: std::ptr::null_mut(),
            count: Default::default(),
        }
    }
}

impl Default for ecs_term_id_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: std::ptr::null_mut(),
            trav: Default::default(),
            flags: Default::default(),
        }
    }
}

impl Default for ecs_term_t {
    fn default() -> Self {
        Self {
            id: Default::default(),
            src: Default::default(),
            first: Default::default(),
            second: Default::default(),
            inout: Default::default(),
            oper: Default::default(),
            id_flags: Default::default(),
            name: std::ptr::null_mut(),
            field_index: Default::default(),
            idr: std::ptr::null_mut(),
            flags: Default::default(),
            move_: Default::default(),
        }
    }
}

impl Default for ecs_filter_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            terms: Default::default(),
            terms_buffer: std::ptr::null_mut(),
            terms_buffer_count: Default::default(),
            storage: std::ptr::null_mut(),
            instanced: Default::default(),
            flags: Default::default(),
            expr: std::ptr::null(),
            entity: Default::default(),
        }
    }
}

impl Default for ecs_query_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            filter: Default::default(),
            order_by_component: Default::default(),
            order_by: Default::default(),
            sort_table: Default::default(),
            group_by_id: Default::default(),
            group_by: Default::default(),
            on_group_create: Default::default(),
            on_group_delete: Default::default(),
            group_by_ctx: std::ptr::null_mut(),
            group_by_ctx_free: Default::default(),
            parent: std::ptr::null_mut(),
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
        }
    }
}

impl Default for ecs_observer_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            filter: Default::default(),
            events: Default::default(),
            yield_existing: Default::default(),
            callback: Default::default(),
            run: Default::default(),
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
            observable: std::ptr::null_mut(),
            last_event_id: std::ptr::null_mut(),
            term_index: Default::default(),
        }
    }
}

impl Default for ecs_header_t {
    fn default() -> Self {
        Self {
            magic: ecs_filter_t_magic as ::std::os::raw::c_int,
            type_: Default::default(),
            mixins: std::ptr::null_mut(),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ecs_iterable_t {
    fn default() -> Self {
        Self {
            init: Default::default(),
        }
    }
}

impl Default for ecs_filter_t {
    fn default() -> Self {
        unsafe { ECS_FILTER_INIT }
    }
}

impl Default for ecs_entity_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            id: Default::default(),
            name: std::ptr::null(),
            sep: std::ptr::null(),
            root_sep: std::ptr::null(),
            symbol: std::ptr::null(),
            use_low_id: Default::default(),
            add: Default::default(),
            add_expr: std::ptr::null(),
        }
    }
}

impl Default for ecs_event_desc_t {
    fn default() -> Self {
        Self {
            event: Default::default(),
            ids: std::ptr::null(),
            table: std::ptr::null_mut(),
            other_table: std::ptr::null_mut(),
            offset: Default::default(),
            count: Default::default(),
            entity: Default::default(),
            param: std::ptr::null_mut(),
            observable: std::ptr::null_mut(),
            flags: Default::default(),
            const_param: std::ptr::null(),
        }
    }
}

impl Default for ecs_system_desc_t {
    fn default() -> Self {
        Self {
            _canary: Default::default(),
            entity: Default::default(),
            query: Default::default(),
            run: Default::default(),
            callback: Default::default(),
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: Default::default(),
            binding_ctx_free: Default::default(),
            interval: Default::default(),
            rate: Default::default(),
            tick_source: Default::default(),
            multi_threaded: Default::default(),
            no_readonly: Default::default(),
        }
    }
}

#[allow(clippy::derivable_impls)] // this is generated by bindgen
impl Default for ecs_pipeline_desc_t {
    fn default() -> Self {
        Self {
            entity: Default::default(),
            query: Default::default(),
        }
    }
}

impl Default for ecs_app_desc_t {
    fn default() -> Self {
        Self {
            target_fps: Default::default(),
            delta_time: Default::default(),
            threads: Default::default(),
            frames: Default::default(),
            enable_rest: Default::default(),
            enable_monitor: Default::default(),
            port: Default::default(),
            init: Default::default(),
            ctx: std::ptr::null_mut(),
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
            ctx: std::ptr::null_mut(),
            binding_ctx: std::ptr::null_mut(),
            ctx_free: None,
            binding_ctx_free: None,
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
            poly: std::ptr::null_mut(),
        }
    }
}
