#![allow(unused_imports)]
#![allow(unused)]

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
pub use seq_macro::seq;

pub mod common_bench;

mod add_existing_bench;
mod add_remove_bench;
mod create_delete_entities_bench;
mod entity_bench;
mod event_bench;
mod get_bench;
mod has_bench;
mod hooks_bench;
mod observer_bench;
mod query_bench;
mod relationships_bench;
mod set_bench;

use add_existing_bench::*;
use add_remove_bench::*;
use create_delete_entities_bench::*;
use entity_bench::*;
use event_bench::*;
// use get_bench::*;
use has_bench::*;
use hooks_bench::*;
use observer_bench::*;
//use query_bench::*;
use relationships_bench::*;
use set_bench::*;

fn ecs_default_criterion() -> Criterion {
    let mut criterion_config = Criterion::default().configure_from_args();
    criterion_config = criterion_config
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_millis(300))
        //.measurement_time(Duration::from_secs(1))
        .sample_size(10)
        .noise_threshold(0.01)
        .confidence_level(0.95)
        .significance_level(0.05)
        .without_plots();
    criterion_config
}

criterion_main!(
    //has_bench,
    //get_bench,
    // set_bench,
    // g_add_remove_bench,
    g_create_delete_entities_bench,
    // hooks_bench,
    // observers_bench,
    // entity_bench,
    // event_bench
);

criterion_group!(
    name = has_bench;
    config = ecs_default_criterion();
    targets =
    has_component_not_found,
    has_components_found
);

// criterion_group!(
//     name = get_bench;
//     config = ecs_default_criterion();
//     targets =
//     get_component_not_found,
//     get_components_found,
//     get_components_not_found,
//     get_mut_components_found,
//     get_mut_components_not_found,
//     ensure_mut_components_found,
//     ensure_mut_components_found_cmd,
//     ensure_mut_not_found_and_remove,
//     ensure_mut_not_found_and_remove_cmd,
// );

criterion_group!(
    name = add_existing_bench;
    config = ecs_default_criterion();
    targets =
    add_existing_operations,
    add_existing_cmd,
);

criterion_group!(
    name = g_add_remove_bench;
    config = ecs_default_criterion();
    targets =
    add_remove,
    add_remove_cmd,
    add_remove_1_tag_to_entity_with_n_components
);

criterion_group!(
    name = set_bench;
    config = ecs_default_criterion();
    targets =
    set_remove,
    set_remove_cmd,
    set_found,
    set_found_cmd
);

criterion_group!(
    name = g_create_delete_entities_bench;
    config = ecs_default_criterion();
    targets =
    create_delete_entities,
    create_delete_entities_cmd,
    create_delete_entities_w_tree
);

criterion_group!(
    name = hooks_bench;
    config = ecs_default_criterion();
    targets =
    add_remove_hooks_components
);

criterion_group!(
    name = relationships_bench;
    config = ecs_default_criterion();
    targets =
    get_relationship_targets,
    override_components_add_remove,
    get_inherited_w_depth,
    change_parent,
    lookup_depth
);

criterion_group!(
    name = observers_bench;
    config = ecs_default_criterion();
    targets =
    observer_create_w_add
);

criterion_group!(
    name = entity_bench;
    config = ecs_default_criterion();
    targets =
    entity
);

criterion_group!(
    name = event_bench;
    config = ecs_default_criterion();
    targets =
    event_emit,
    event_emit_propagate,
    event_emit_forward,
    event_modified
);
