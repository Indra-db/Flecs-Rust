use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
// include!("common.rs") is used so this file can exist and we can manually control the order of the benchmarks

mod add_existing;
mod add_remove;
mod create_delete_entities;
mod entity;
mod event;
mod get;
mod has;
mod hooks;
mod observer;
mod query;
mod relationships;
mod set;

use add_existing::*;
use add_remove::*;
use create_delete_entities::*;
use entity::*;
use event::*;
use get::*;
use has::*;
use hooks::*;
use observer::*;
//use query::*;
use relationships::*;
use set::*;

fn ecs_default_criterion() -> Criterion {
    let mut criterion_config = Criterion::default().configure_from_args();
    criterion_config = criterion_config
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(3))
        .sample_size(50)
        .noise_threshold(0.01)
        .confidence_level(0.95)
        .significance_level(0.05)
        .without_plots();
    criterion_config
}

criterion_main!(
    has,
    get,
    set,
    add_remove,
    create_delete_entities,
    hooks,
    observers,
    entity,
    event
);

criterion_group!(
    name = has;
    config = ecs_default_criterion();
    targets =
    flecs_has_component_not_found,
    flecs_has_components_found
);

criterion_group!(
    name = get;
    config = ecs_default_criterion();
    targets =
    flecs_get_component_not_found,
    flecs_get_components_found,
    flecs_get_components_not_found,
    flecs_get_mut_components_found,
    flecs_get_mut_components_not_found,
    flecs_ensure_mut_components_found,
    flecs_ensure_mut_components_found_cmd,
    flecs_ensure_mut_not_found_and_remove,
    flecs_ensure_mut_not_found_and_remove_cmd,
);

criterion_group!(
    name = add_existing;
    config = ecs_default_criterion();
    targets =
    flecs_add_existing_operations,
    flecs_add_existing_cmd,
);

criterion_group!(
    name = add_remove;
    config = ecs_default_criterion();
    targets =
    flecs_add_remove,
    flecs_add_remove_cmd,
    flecs_add_remove_1_tag_to_entity_with_n_components
);

criterion_group!(
    name = set;
    config = ecs_default_criterion();
    targets =
    flecs_set_remove,
    flecs_set_remove_cmd,
    flecs_set_found,
    flecs_set_found_cmd
);

criterion_group!(
    name = create_delete_entities;
    config = ecs_default_criterion();
    targets =
    flecs_create_delete_entities,
    flecs_create_delete_entities_cmd,
    flecs_create_delete_entities_w_tree
);

criterion_group!(
    name = hooks;
    config = ecs_default_criterion();
    targets =
    add_remove_hooks_components
);

criterion_group!(
    name = relationships;
    config = ecs_default_criterion();
    targets =
    get_relationship_targets,
    override_components_add_remove,
    get_inherited_w_depth,
    change_parent
);

criterion_group!(
    name = observers;
    config = ecs_default_criterion();
    targets =
    observer_create_w_add
);

criterion_group!(
    name = entity;
    config = ecs_default_criterion();
    targets =
    flecs_entity_set_name
);

criterion_group!(
    name = event;
    config = ecs_default_criterion();
    targets =
    flecs_event_emit,
    flecs_event_emit_propagate,
    flecs_event_emit_forward,
    flecs_event_modified
);
