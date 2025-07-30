#include "flecs_rust.h"
/* This uses internals from flecs which aren't in the header. */
#include "flecs.c"

void* ecs_rust_mut_get_id(
    const ecs_world_t *world,
    ecs_entity_t _entity,
    const ecs_record_t* r,
    ecs_id_t id)
{
    (void)_entity;
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, _entity), ECS_INVALID_PARAMETER, NULL);
    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    world = ecs_get_world(world);

    ecs_table_t *table = r->table;
    ecs_assert(table != NULL, ECS_INTERNAL_ERROR, NULL);

    flecs_check_exclusive_world_access_write(world);

    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    if (id < FLECS_HI_COMPONENT_ID) {
        if (!world->non_trivial[id]) {
            ecs_get_low_id(table, r, id);
            return NULL;
        }
    }
    ecs_component_record_t *cr = flecs_components_get(world, id);
    int32_t row = ECS_RECORD_TO_ROW(r->row);
    return flecs_get_component_ptr(table, row, cr).ptr;
error:
    return NULL;
}

void* ecs_rust_get_id(
    const ecs_world_t *world,
    ecs_entity_t entity,
    const ecs_record_t* r,
    ecs_id_t id)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);
    ecs_assert(r != NULL, ECS_INVALID_PARAMETER, NULL);

    world = ecs_get_world(world);

    ecs_table_t *table = r->table;
    ecs_assert(table != NULL, ECS_INTERNAL_ERROR, NULL);

    if (id < FLECS_HI_COMPONENT_ID) {
        ecs_get_low_id(table, r, id);
        if (!world->non_trivial[id]) {
            if (!(table->flags & EcsTableHasIsA)) {
                return NULL;
            }
        }
    }

    ecs_component_record_t *cr = flecs_components_get(world, id);
    if (!cr) {
        return NULL;
    }

    if (cr->flags & EcsIdDontFragment) {
        void *ptr = flecs_component_sparse_get(cr, entity);
        if (ptr) {
            return ptr;
        }
    }

    const ecs_table_record_t *tr = flecs_component_get_table(cr, table);
    if (!tr) {
        return flecs_get_base_component(world, table, id, cr, 0);
    } else {
        if (cr->flags & EcsIdIsSparse) {
            return flecs_component_sparse_get(cr, entity);
        }
        ecs_check(tr->column != -1, ECS_NOT_A_COMPONENT, NULL);
    }

    int32_t row = ECS_RECORD_TO_ROW(r->row);
    return flecs_table_get_component(table, tr->column, row).ptr;

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

    ecs_component_record_t *cr = flecs_components_get(world, id);
    if (!cr) {
        return -1;
    }
    ecs_table_record_t *tr = ecs_table_cache_get(&cr->cache, table);
    if (!tr) {
        return -1;
    }
    return tr->count;
error:
    return -1;
}

