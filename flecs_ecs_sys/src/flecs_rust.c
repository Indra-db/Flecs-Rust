#include "flecs_rust.h"
/* This uses internals from flecs which aren't in the header. */
#include "flecs.c"

void* ecs_rust_mut_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_table_t* table,
    ecs_id_t id)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);

    ecs_assert(record != NULL, ECS_INVALID_PARAMETER, NULL);

    ecs_id_record_t *idr = flecs_id_record_get(world, id);
    if (!idr) {
        return NULL;
    }

    if (idr->flags & EcsIdIsSparse) {
        return flecs_sparse_get_any(idr->sparse, 0, entity);
    }

    const ecs_table_record_t *tr = flecs_id_record_get_table(idr, table);

    if (!tr || (tr->column == -1)) {
        return NULL;
    }

    int32_t column_index = tr->column;

    ecs_check(column_index < table->column_count, ECS_NOT_A_COMPONENT, NULL);

    ecs_column_t *column = &table->data.columns[column_index];

    return ecs_vec_get(column->data, column->ti->size, ECS_RECORD_TO_ROW(record->row));

error:
    return NULL;
}

void* ecs_rust_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* record,
    ecs_table_t* table,
    ecs_id_t id)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);

    ecs_assert(record != NULL, ECS_INVALID_PARAMETER, NULL);

    ecs_id_record_t *idr = flecs_id_record_get(world, id);
    if (!idr) {
        return NULL;
    }

    const ecs_table_record_t *tr = flecs_id_record_get_table(idr, table);
    if (!tr) {
        return flecs_get_base_component(world, table, id, idr, 0);
    } else {
        if (idr->flags & EcsIdIsSparse) {
            return flecs_sparse_get_any(idr->sparse, 0, entity);
        }
        ecs_check(tr->column != -1, ECS_NOT_A_COMPONENT, NULL);
    }

    int32_t column_index = tr->column;

    ecs_check(column_index < table->column_count, ECS_NOT_A_COMPONENT, NULL);

    ecs_column_t *column = &table->data.columns[column_index];

    return ecs_vec_get(column->data, column->ti->size, ECS_RECORD_TO_ROW(record->row));

error:
    return NULL;
}

int32_t ecs_rust_rel_count(
    const ecs_world_t *world,
    ecs_id_t id,
    ecs_table_t* table)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);

    if (!table) return -1;

    flecs_poly_assert(world, ecs_world_t);
    ecs_assert(id != 0, ECS_INVALID_PARAMETER, NULL);

    ecs_id_record_t *idr = flecs_id_record_get(world, id);
    if (!idr) {
        return -1;
    }
    ecs_table_record_t *tr = ecs_table_cache_get(&idr->cache, table);
    return tr->count;
error:
    return -1;
}

