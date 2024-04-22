//commented modules are either test cases that take too long or are not relevant to the test suite
mod a_hello_world;

mod entity_basics;
mod entity_hierarchy;
mod entity_hooks;
mod entity_iterate_components;
mod entity_prefab;

mod observer_basics;
mod observer_custom_event;
mod observer_entity_event;
mod observer_monitor;
mod observer_propagate;
mod observer_two_components;
mod observer_yield_existing;

mod prefab_basics;
mod prefab_hierarchy;
mod prefab_nested;
mod prefab_override;
mod prefab_slots;
mod prefab_typed;
mod prefab_variant;

mod query_basics;
mod query_chaining_queries;
mod query_change_tracking;
mod query_find_entity;
mod query_group_by;
mod query_group_by_callbacks;
mod query_group_by_custom;
mod query_group_iter;
mod query_hierarchy;
mod query_instancing;
mod query_iter;
mod query_singleton;
mod query_sorting;
mod query_wildcard;
mod query_with;
mod query_without;
mod query_world_query;

mod relationships_basics;
mod relationships_component_data;
mod relationships_enum;
mod relationships_exclusive;
mod relationships_symmetric;
mod relationships_union;

mod rules_basics;
mod rules_component_inheritance;
mod rules_cyclic_variables;
mod rules_facts;
mod rules_setting_variables;
mod rules_transitive_queries;

mod system_basics;
mod system_ctx;
mod system_custom_phases;
mod system_custom_phases_no_builtin;
mod system_custom_pipeline;
mod system_custom_runner;
//mod system_delta_time;
mod system_mutate_entity;
mod system_mutate_entity_handle;
mod system_no_readonly;
mod system_pipeline;
mod system_startup_system;
//mod system_sync_point;
//mod system_sync_point_delete;
//mod system_target_fps;
//mod system_time_interval;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

fn main() {}

#[cfg(test)]
mod examples {
    use super::*;

    macro_rules! test {
        ($name:ident) => {
            #[test]
            fn $name() {
                $name::main().unwrap().test();
            }
        };
    }

    test!(a_hello_world);

    test!(entity_basics);
    test!(entity_hierarchy);
    test!(entity_hooks);
    test!(entity_iterate_components);
    test!(entity_prefab);

    test!(observer_basics);
    test!(observer_custom_event);
    test!(observer_entity_event);
    test!(observer_monitor);
    test!(observer_propagate);
    test!(observer_two_components);
    test!(observer_yield_existing);

    test!(prefab_basics);
    test!(prefab_hierarchy);
    test!(prefab_nested);
    test!(prefab_override);
    test!(prefab_slots);
    test!(prefab_typed);
    test!(prefab_variant);

    test!(query_basics);
    test!(query_chaining_queries);
    test!(query_change_tracking);
    test!(query_find_entity);
    test!(query_group_by);
    test!(query_group_by_callbacks);
    test!(query_group_by_custom);
    test!(query_group_iter);
    test!(query_hierarchy);
    test!(query_instancing);
    test!(query_iter);
    test!(query_singleton);
    test!(query_sorting);
    test!(query_wildcard);
    test!(query_with);
    test!(query_without);
    test!(query_world_query);

    test!(relationships_basics);
    test!(relationships_component_data);
    test!(relationships_enum);
    test!(relationships_exclusive);
    test!(relationships_symmetric);
    test!(relationships_union);

    test!(rules_basics);
    test!(rules_component_inheritance);
    test!(rules_cyclic_variables);
    test!(rules_facts);
    test!(rules_setting_variables);
    test!(rules_transitive_queries);

    test!(system_basics);
    #[test]
    fn system_ctx() {
        let snap = system_ctx::main().unwrap();
        assert!(snap.count() > 2);
    }
    test!(system_custom_phases);
    test!(system_custom_phases_no_builtin);
    test!(system_custom_pipeline);
    test!(system_custom_runner);
    test!(system_mutate_entity);
    test!(system_mutate_entity_handle);
    test!(system_no_readonly);
    test!(system_pipeline);
    test!(system_startup_system);
}
