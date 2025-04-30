#include "flecs.h"

FLECS_API
void* ecs_rust_mut_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_table_t* table,
    ecs_id_t id);

FLECS_API
void* ecs_rust_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_table_t* table,
    ecs_id_t id);

FLECS_API
    int32_t ecs_rust_rel_count(
    const ecs_world_t *world,
    ecs_id_t id,
    ecs_table_t* table);

