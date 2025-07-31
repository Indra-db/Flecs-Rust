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

#ifdef FLECS_SAFETY_LOCKS

const ecs_type_info_t* ecs_rust_get_type_info_from_record(
    ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(id != 0, ECS_INVALID_PARAMETER, NULL);

    if (!idr && ECS_IS_PAIR(id)) {
        world = ecs_get_world(world);
        idr = flecs_components_ensure(world, 
            ecs_pair(ECS_PAIR_FIRST(id), EcsWildcard));
        if (!idr || !idr->type_info) {
            idr = NULL;
        }
        if (!idr) {
            ecs_entity_t first = ecs_pair_first(world, id);
            if (!first || !ecs_has_id(world, first, EcsPairIsTag)) {
                idr = flecs_components_ensure(world, 
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
    ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr)     
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

uint64_t ecs_rust_table_id(
    const ecs_table_t* table)
{
    return table->id;
}

bool ecs_rust_is_sparse_idr(
    const ecs_component_record_t* idr)
{
    return idr->flags & EcsIdIsSparse;
}

ecs_component_record_t* ecs_id_record_get(
    const ecs_world_t *world,
    ecs_id_t id)
{
    flecs_poly_assert(world, ecs_world_t);
    if (id == ecs_pair(EcsIsA, EcsWildcard)) {
        return world->cr_isa_wildcard;
    } else if (id == ecs_pair(EcsChildOf, EcsWildcard)) {
        return world->cr_childof_wildcard;
    } else if (id == ecs_pair_t(EcsIdentifier, EcsName)) {
        return world->cr_identifier_name;
    }

    ecs_id_t hash = flecs_component_hash(id);
    ecs_component_record_t *idr = NULL;
    if (hash >= FLECS_HI_ID_RECORD_ID) {
        idr = ecs_map_get_deref(&world->id_index_hi, ecs_component_record_t, hash);
    } else {
        idr = world->id_index_lo[hash];
    }

    return idr;
}

int32_t ecs_table_get_column_index_w_idr(
    const ecs_world_t *world,
    const ecs_table_t *table,
    ecs_id_t id,
    ecs_component_record_t* idr)
{
    flecs_poly_assert(world, ecs_world_t);
    ecs_check(table != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_id_is_valid(world, id), ECS_INVALID_PARAMETER, NULL);

    if (id < FLECS_HI_COMPONENT_ID) {
        int16_t res = table->component_map[id];
        if (res > 0) {
            return res - 1;
        }
        return -1;
    }

    if (!idr) {
        return -1;
    }

    const ecs_table_record_t *tr = flecs_component_get_table(idr, table);
    if (!tr) {
        return -1;
    }

    return tr->column;
error:
    return -1;
}
#endif