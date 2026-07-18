#include "flecs.h"

typedef struct ecs_rust_set_t {
    void *ptr;
    bool call_modified;
    bool is_new;
} ecs_rust_set_t;

FLECS_API
    int32_t ecs_rust_rel_count(
    const ecs_world_t *world,
    ecs_id_t id,
    ecs_table_t* table);

FLECS_API
ecs_entity_t ecs_rust_get_typeid(
    const ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr);

FLECS_API
const ecs_type_info_t* ecs_rust_get_type_info_from_record(
    const ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr);

FLECS_API
ecs_rust_set_t ecs_rust_set(
    ecs_world_t *world,
    ecs_entity_t entity,
    ecs_id_t id,
    const void *new_ptr,
    size_t size);

/* Fast path for compile-time-known sparse / dont_fragment components without
 * the (OnInstantiate, Inherit) trait. Mirrors ecs_get_sparse_id() but returns
 * an ecs_get_ptr_t so lock-target info is available under
 * FLECS_MUT_ALIAS_LOCKS. */
FLECS_API
ecs_get_ptr_t ecs_rust_get_sparse_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    ecs_id_t id,
    size_t size);

/* ABI guards: C-side sizeof for structs with FLECS_DEBUG-gated fields, so the
 * Rust side can assert its bindings match the compiled profile. */
FLECS_API size_t ecs_rust_sizeof_ecs_ref_t(void);
FLECS_API size_t ecs_rust_sizeof_ecs_map_t(void);
FLECS_API size_t ecs_rust_sizeof_ecs_map_iter_t(void);
FLECS_API size_t ecs_rust_sizeof_ecs_stack_t(void);
FLECS_API size_t ecs_rust_sizeof_ecs_stack_cursor_t(void);
