#include "flecs.h"

FLECS_API
void* ecs_rust_mut_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_id_t id);

FLECS_API
void* ecs_rust_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_id_t id);

FLECS_API
    int32_t ecs_rust_rel_count(
    const ecs_world_t *world,
    ecs_id_t id,
    ecs_table_t* table);

#ifdef FLECS_SAFETY_LOCKS

FLECS_API
bool ecs_rust_is_sparse_idr(
    const ecs_component_record_t* idr);

FLECS_API
ecs_entity_t ecs_rust_get_typeid(
    ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr);

FLECS_API
const ecs_type_info_t* ecs_rust_get_type_info_from_record(
    ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr);

FLECS_API
ecs_component_record_t* ecs_id_record_get(
    const ecs_world_t *world,
    ecs_id_t id);

FLECS_API
int32_t ecs_table_get_column_index_w_idr(
    const ecs_world_t *world,
    const ecs_table_t *table,
    ecs_id_t id,
    ecs_component_record_t* idr);

#endif