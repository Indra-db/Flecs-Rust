// On Windows, prevent winsock.h from being included by windows.h
// This must be defined before any Windows headers are included
#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOMINMAX
#define NOMINMAX
#endif
#endif

#include "flecs_rust.h"
/* This uses internals from flecs which aren't in the header. */
#include "flecs.c"

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


ecs_entity_t ecs_rust_get_typeid(
    const ecs_world_t *world,
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

const ecs_type_info_t* ecs_rust_get_type_info_from_record(
    const ecs_world_t *world,
    ecs_id_t id,
    const ecs_component_record_t* idr)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(id != 0, ECS_INVALID_PARAMETER, NULL);

    if (!idr && ECS_IS_PAIR(id)) {
        world = ecs_get_world(world);
        idr = flecs_components_get(world,
            ecs_pair(ECS_PAIR_FIRST(id), EcsWildcard));
        if (!idr || !idr->type_info) {
            idr = NULL;
        }
        if (!idr) {
            ecs_entity_t first = ecs_pair_first(world, id);
            if (!first || !ecs_has_id(world, first, EcsPairIsTag)) {
                idr = flecs_components_get(world,
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

/* Like flecs_ensure but adds new components WITHOUT constructor.
 * Mirrors flecs_ensure's fast paths (component_map for low IDs, sparse
 * handling) with the single difference: flecs_add_id_w_record(construct=false).
 * We don't modify upstream flecs_ensure because "ensure" semantically implies
 * construction. We can't use ecs_emplace_id because it debug-asserts no
 * on_replace (that assert protects the emplace public API where no new_ptr
 * is available, but we DO have new_ptr so it doesn't apply). */
static
flecs_component_ptr_t flecs_rust_ensure(
    ecs_world_t *world,
    ecs_entity_t entity,
    ecs_id_t component,
    ecs_record_t *r,
    ecs_size_t size)
{
    flecs_component_ptr_t dst = {0};

    ecs_assert(r != NULL, ECS_INTERNAL_ERROR, NULL);

    ecs_component_record_t *cr = NULL;
    ecs_table_t *table = r->table;
    ecs_assert(table != NULL, ECS_INTERNAL_ERROR, NULL);

    if (component < FLECS_HI_COMPONENT_ID) {
        int16_t column_index = table->component_map[component];
        if (column_index > 0) {
            ecs_column_t *column = &table->data.columns[column_index - 1];
            ecs_assert(column->ti->size == size, ECS_INTERNAL_ERROR, NULL);
            dst.ptr = ECS_ELEM(column->data, size, ECS_RECORD_TO_ROW(r->row));
            dst.ti = column->ti;
            return dst;
        } else if (column_index < 0) {
            column_index = flecs_ito(int16_t, -column_index - 1);
            const ecs_table_record_t *tr = &table->_->records[column_index];
            cr = tr->hdr.cr;
            if (cr->flags & EcsIdSparse) {
                dst.ptr = flecs_component_sparse_get(
                    world, cr, r->table, entity);
                dst.ti = cr->type_info;
                ecs_assert(dst.ti->size == size, ECS_INTERNAL_ERROR, NULL);
                return dst;
            }
        }
    } else {
        cr = flecs_components_get(world, component);
        dst = flecs_get_component_ptr(
            world, table, ECS_RECORD_TO_ROW(r->row), cr);
        if (dst.ptr) {
            ecs_assert(dst.ti->size == size, ECS_INTERNAL_ERROR, NULL);
            return dst;
        }
    }

    /* Entity doesn't have component — add WITHOUT constructor */
    flecs_add_id_w_record(world, entity, r, component, false);

    /* Flush so the pointer we're fetching is stable */
    flecs_defer_end(world, world->stages[0]);
    flecs_defer_begin(world, world->stages[0]);

    if (!cr) {
        cr = flecs_components_get(world, component);
        ecs_assert(cr != NULL, ECS_INTERNAL_ERROR, NULL);
    }

    ecs_assert(r->table != NULL, ECS_INTERNAL_ERROR, NULL);
    return flecs_get_component_ptr(
        world, r->table, ECS_RECORD_TO_ROW(r->row), cr);
}

/* Like flecs_defer_cpp_set but skips ctor for new components.
 * New components use EcsCmdEmplace (no ctor on flush). Existing components
 * use EcsCmdAddModified with on_replace handling (same as cpp). */
static
void* flecs_defer_rust_set(
    ecs_world_t *world,
    ecs_stage_t *stage,
    ecs_entity_t entity,
    ecs_id_t id,
    ecs_size_t size,
    const void *value,
    bool *is_new)
{
    ecs_assert(value != NULL, ECS_INTERNAL_ERROR, NULL);
    ecs_assert(size != 0, ECS_INTERNAL_ERROR, NULL);

    ecs_cmd_t *cmd = flecs_cmd_new_batched(stage, entity);
    ecs_assert(cmd != NULL, ECS_INTERNAL_ERROR, NULL);
    cmd->entity = entity;
    cmd->id = id;

    ecs_record_t *r = flecs_entities_get(world, entity);
    flecs_component_ptr_t ptr = flecs_defer_get_existing(
        world, entity, r, id, size);

    bool new_component = (ptr.ptr == NULL);
    if (is_new) {
        *is_new = new_component;
    }

    const ecs_type_info_t *ti = ptr.ti;
    ecs_check(ti != NULL, ECS_INVALID_PARAMETER,
        "provided component is not a type");
    ecs_assert(size == ti->size, ECS_INVALID_PARAMETER,
        "mismatching size specified for component in ensure/emplace/set");

    /* Handle trivial set command (no hooks, OnSet observers) */
    if (id < FLECS_HI_COMPONENT_ID) {
        if (!world->non_trivial_set[id]) {
            if (new_component) {
                ptr.ptr = flecs_stack_alloc(
                    &stage->cmd->stack, size, ti->alignment);

                /* No OnSet observers, so ensure is enough */
                cmd->kind = EcsCmdEnsure;
                cmd->is._1.size = size;
                cmd->is._1.value = ptr.ptr;
            } else {
                /* No OnSet observers, so only thing we need to do is make sure
                 * that a preceding remove command doesn't cause the entity to
                 * end up without the component. */
                cmd->kind = EcsCmdAdd;
            }

            ecs_os_memcpy(ptr.ptr, value, size);
            return ptr.ptr;
        }
    }

    if (new_component) {
        if (!ti->hooks.on_replace) {
            /* No on_replace: use EcsCmdEmplace (no ctor on flush). Rust writes
             * value directly into the cmd buffer. Then queue EcsCmdModified so
             * OnSet observers fire (EcsCmdEmplace alone skips OnSet). */
            cmd->kind = EcsCmdEmplace;
            cmd->is._1.size = size;
            ptr.ptr = cmd->is._1.value =
                flecs_stack_alloc(&stage->cmd->stack, size, ti->alignment);

            /* Queue Modified so OnSet fires at flush. Uses flecs_cmd_new
             * (not batched) so it's processed after the emplace in the main
             * flush loop. */
            ecs_cmd_t *mod_cmd = flecs_cmd_new(stage);
            if (mod_cmd) {
                mod_cmd->kind = EcsCmdModified;
                mod_cmd->id = id;
                mod_cmd->entity = entity;
            }
        } else {
            /* Has on_replace: can't use EcsCmdEmplace (ecs_emplace_id
             * debug-asserts no on_replace). Fall back to EcsCmdSet which
             * handles on_replace at flush. Trade-off: flush ctors dst then
             * move_dtor drops it (extra ctor+drop). Only hits types with
             * explicit .on_replace() registration. */
            cmd->kind = EcsCmdSet;
            cmd->is._1.size = size;
            ptr.ptr = cmd->is._1.value =
                flecs_stack_alloc(&stage->cmd->stack, size, ti->alignment);
        }
    } else {
        cmd->kind = EcsCmdAddModified;

        if (ti->hooks.on_replace) {
            flecs_invoke_replace_hook(
                world, r->table, entity, id, ptr.ptr, value, ti);
        }
    }

    return ptr.ptr;
error:
    return NULL;
}

/* Rust-optimized set: like ecs_cpp_set but skips ctor for new components.
 * In C++, ctor + assignment operator is the pattern. In Rust, there's no
 * assignment operator, so the ctor output would just be dropped — wasted work.
 * Both deferred and non-deferred paths skip construction for new components. */
ecs_rust_set_t ecs_rust_set(
    ecs_world_t *world,
    ecs_entity_t entity,
    ecs_id_t id,
    const void *new_ptr,
    size_t size)
{
    ecs_check(world != NULL, ECS_INVALID_PARAMETER, NULL);
    ecs_check(ecs_is_alive(world, entity), ECS_INVALID_PARAMETER, NULL);

    ecs_stage_t *stage = flecs_stage_from_world(&world);
    ecs_rust_set_t result;

    if (flecs_defer_cmd(stage)) {
        result.ptr = flecs_defer_rust_set(world, stage, entity, id,
            flecs_utosize(size), new_ptr, &result.is_new);
        result.call_modified = false;
        return result;
    }

    ecs_record_t *r = flecs_entities_get(world, entity);
    ecs_table_t *prev_table = r->table;
    flecs_component_ptr_t dst = flecs_rust_ensure(world, entity, id, r,
        flecs_uto(int32_t, size));

    result.ptr = dst.ptr;
    result.is_new = (r->table != prev_table);

    if (id < FLECS_HI_COMPONENT_ID) {
        if (!world->non_trivial_set[id]) {
            result.call_modified = false;
            goto done;
        }
    }

    /* Not deferring, so need to call modified after setting the component */
    result.call_modified = true;

    if (!result.is_new && dst.ti->hooks.on_replace) {
        flecs_invoke_replace_hook(
            world, r->table, entity, id, dst.ptr, new_ptr, dst.ti);
    }

done:
    flecs_defer_end(world, stage);

    return result;
error:
    return (ecs_rust_set_t){0};
}