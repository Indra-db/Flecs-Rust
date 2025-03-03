#include "flecs_rust.h"
/* This uses internals from flecs which aren't in the header. */
#include "flecs.c"

void* ecs_rust_mut_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* r,
    ecs_table_t* table,
    ecs_id_t id)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);

    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    if (id < FLECS_HI_COMPONENT_ID) {
        ecs_get_low_id(table, r, id);
        return NULL;
    }

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

    //ecs_check(column_index < table->column_count, ECS_NOT_A_COMPONENT, NULL);

    ecs_column_t *column = &table->data.columns[column_index];

    return ECS_ELEM(column->data, column->ti->size, ECS_RECORD_TO_ROW(r->row));
    //return ecs_vec_get(column->data, column->ti->size, ECS_RECORD_TO_ROW(record->row));

error:
    return NULL;
}

void* ecs_rust_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* r,
    ecs_table_t* table,
    ecs_id_t id)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);
    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    if (id < FLECS_HI_COMPONENT_ID) {
        ecs_get_low_id(table, r, id);
        if (!(table->flags & EcsTableHasIsA)) {
            return NULL;
        }
    }

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

    return ECS_ELEM(column->data, column->ti->size, ECS_RECORD_TO_ROW(r->row));

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
    if (!tr) {
        return -1;
    }
    return tr->count;
error:
    return -1;
}

const ecs_type_info_t* ecs_rust_get_type_info_from_record(
    const ecs_world_t *world,
    ecs_id_t id,
    const ecs_id_record_t* idr)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(id != 0, ECS_INVALID_PARAMETER, NULL);

    if (!idr && ECS_IS_PAIR(id)) {
        world = ecs_get_world(world);
        idr = flecs_id_record_get(world, 
            ecs_pair(ECS_PAIR_FIRST(id), EcsWildcard));
        if (!idr || !idr->type_info) {
            idr = NULL;
        }
        if (!idr) {
            ecs_entity_t first = ecs_pair_first(world, id);
            if (!first || !ecs_has_id(world, first, EcsPairIsTag)) {
                idr = flecs_id_record_get(world, 
                    ecs_pair(EcsWildcard, ECS_PAIR_SECOND(id)));
                if (!idr || !idr->type_info) {
                    idr = NULL;
                }
            }
        }
    }

    if (idr) {
        return idr->type_info;
    } else if (!(id & ECS_ID_FLAGS_MASK)) {
        world = ecs_get_world(world);
        return flecs_type_info_get(world, id);
    }
error:
    return NULL;
}

ecs_entity_t ecs_rust_get_typeid(
    const ecs_world_t *world,
    ecs_id_t id,
    const ecs_id_record_t* idr)     
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    const ecs_type_info_t *ti = ecs_rust_get_type_info_from_record(world, id, idr);
    if (ti) {
        ecs_assert(ti->component != 0, ECS_INTERNAL_ERROR, NULL);
        return ti->component;
    }
error:
    return 0;
}