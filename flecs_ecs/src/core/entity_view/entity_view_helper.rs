use core::ffi::c_void;
use core::ptr;
use std::ffi::CStr;
use std::ptr::NonNull;

use crate::core::*;
use crate::sys;

pub(crate) unsafe fn add_id_unchecked(
    world: *mut sys::ecs_world_t,
    entity_id: Entity,
    id: impl IntoId,
) {
    let id = *id.into();

    unsafe { sys::ecs_add_id(world, *entity_id, id) }
}

/// Test if an entity has an id.
///
/// # Arguments
///
/// * `entity` - The entity to check.
///
/// # Returns
///
/// True if the entity has or inherits the provided id, false otherwise.
///
/// # See also
///
/// * [`EntityView::has()`]
/// * C++ API: `entity_view::has`
#[doc(alias = "entity_view::has")]
#[inline(always)]
pub(super) fn has_id(world: *const sys::ecs_world_t, own_id: Entity, id: impl IntoId) -> bool {
    unsafe { sys::ecs_has_id(world, *own_id, *id.into()) }
}

pub(super) fn check_add_id_validity(world: *const sys::ecs_world_t, id: u64) {
    let is_valid_id = unsafe { sys::ecs_id_is_valid(world, id) };

    if !is_valid_id {
        panic!("Id is not a valid component, pair or entity.");
    }

    let is_not_tag = unsafe { sys::ecs_get_typeid(world, id) != 0 };

    if is_not_tag {
        let hooks = unsafe { sys::ecs_get_hooks_id(world, id) };
        let is_default_hook = unsafe { (*hooks).ctor.is_some() };
        if !is_default_hook {
            panic!("Id is not a ZST type such as a Tag or Entity or does not implement the Default hook for a non ZST type.");
        }
    }
}

pub(super) fn path_from_id_default_sep(
    world: *const sys::ecs_world_t,
    id: Entity,
    parent: impl Into<Entity>,
) -> Option<String> {
    NonNull::new(unsafe {
        sys::ecs_get_path_w_sep(
            world,
            *parent.into(),
            *id,
            SEPARATOR.as_ptr(),
            SEPARATOR.as_ptr(),
        )
    })
    .map(|s| unsafe {
        let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
        // Convert the C string to a Rust String without any new heap allocation.
        // The String will de-allocate the C string when it goes out of scope.
        String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
    })
}

/// Lookup an entity by name.
///
/// Lookup an entity in the scope of this entity. The provided path may
/// contain double colons as scope separators, for example: "`Foo::Bar`".
///
/// # Arguments
///
/// * `path` - The name of the entity to lookup.
/// * `recursively` - Recursively traverse up the tree until entity is found.
///
/// # Returns
///
/// The entity if found, otherwise `None`.
///
/// # See also
///
/// * C++ API: `entity_view::lookup`
#[doc(alias = "entity_view::lookup")]
#[inline(always)]
pub(super) fn try_lookup_impl<'w>(
    world: WorldRef<'w>,
    id: Entity,
    name: &str,
    recursively: bool,
) -> Option<EntityView<'w>> {
    let name = compact_str::format_compact!("{}\0", name);

    ecs_assert!(
        id != 0,
        FlecsErrorCode::InvalidParameter,
        "invalid lookup from null handle"
    );
    let id = unsafe {
        sys::ecs_lookup_path_w_sep(
            world.world_ptr(),
            *id,
            name.as_ptr() as *const _,
            SEPARATOR.as_ptr(),
            SEPARATOR.as_ptr(),
            recursively,
        )
    };

    if id == 0 {
        None
    } else {
        Some(EntityView::new_from(world, id))
    }
}

pub(super) fn entity_observer_create(
    world: *mut sys::ecs_world_t,
    event: sys::ecs_entity_t,
    entity: sys::ecs_entity_t,
    binding_ctx: *mut ObserverEntityBindingCtx,
    callback: sys::ecs_iter_action_t,
) {
    let mut desc = sys::ecs_observer_desc_t::default();
    desc.events[0] = event;
    desc.query.terms[0].id = ECS_ANY;
    desc.query.terms[0].src.id = entity;
    desc.callback = callback;
    desc.callback_ctx = binding_ctx as *mut c_void;
    desc.callback_ctx_free = Some(binding_entity_ctx_drop);

    let observer = unsafe { sys::ecs_observer_init(world, &desc) };
    ecs_add_pair(world, observer, ECS_CHILD_OF, entity);
}

/// Callback of the observe functionality
///
/// # Arguments
///
/// * `iter` - The iterator which gets passed in from `C`
///
/// # See also
///
/// * C++ API: `entity_observer_delegate::invoke`
#[doc(alias = "entity_observer_delegate::invoke")]
pub(super) unsafe extern "C" fn run_empty<Func>(iter: *mut sys::ecs_iter_t)
where
    Func: FnMut(),
{
    let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
    let empty = (*ctx).empty.unwrap();
    let empty = &mut *(empty as *mut Func);
    let iter_count = (*iter).count as usize;

    sys::ecs_table_lock((*iter).world, (*iter).table);

    for _i in 0..iter_count {
        empty();
    }

    sys::ecs_table_unlock((*iter).world, (*iter).table);
}

/// Callback of the observe functionality
///
/// # Arguments
///
/// * `iter` - The iterator which gets passed in from `C`
///
/// # See also
///
/// * C++ API: `entity_observer_delegate::invoke`
#[doc(alias = "entity_observer_delegate::invoke")]
pub(super) unsafe extern "C" fn run_empty_entity<Func>(iter: *mut sys::ecs_iter_t)
where
    Func: FnMut(&mut EntityView),
{
    let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
    let empty = (*ctx).empty_entity.unwrap();
    let empty = &mut *(empty as *mut Func);
    let iter_count = (*iter).count as usize;

    sys::ecs_table_lock((*iter).world, (*iter).table);

    for _i in 0..iter_count {
        let world = WorldRef::from_ptr((*iter).world);
        empty(&mut EntityView::new_from(
            world,
            sys::ecs_field_src(iter, 0),
        ));
    }

    sys::ecs_table_unlock((*iter).world, (*iter).table);
}

/// Callback of the observe functionality
///
/// # Arguments
///
/// * `iter` - The iterator which gets passed in from `C`
///
/// # See also
///
/// * C++ API: `entity_payload_observer_delegate::invoke`
#[doc(alias = "entity_payload_observer_delegate::invoke")]
pub(super) unsafe extern "C" fn run_payload<C, Func>(iter: *mut sys::ecs_iter_t)
where
    Func: FnMut(&C),
{
    let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
    let empty = (*ctx).payload.unwrap();
    let empty = &mut *(empty as *mut Func);
    let iter_count = (*iter).count as usize;

    sys::ecs_table_lock((*iter).world, (*iter).table);

    for _i in 0..iter_count {
        let data = (*iter).param as *mut C;
        let data_ref = &mut *data;
        empty(data_ref);
    }

    sys::ecs_table_unlock((*iter).world, (*iter).table);
}

/// Callback of the observe functionality
///
/// # Arguments
///
/// * `iter` - The iterator which gets passed in from `C`
///
/// # See also
///
/// * C++ API: `entity_payload_observer_delegate::invoke`
#[doc(alias = "entity_payload_observer_delegate::invoke")]
pub(super) unsafe extern "C" fn run_payload_entity<C, Func>(iter: *mut sys::ecs_iter_t)
where
    Func: FnMut(&mut EntityView, &C),
{
    let ctx: *mut ObserverEntityBindingCtx = (*iter).callback_ctx as *mut _;
    let empty = (*ctx).payload_entity.unwrap();
    let empty = &mut *(empty as *mut Func);
    let iter_count = (*iter).count as usize;

    sys::ecs_table_lock((*iter).world, (*iter).table);

    for _i in 0..iter_count {
        let data = (*iter).param as *mut C;
        let data_ref = &mut *data;
        let world = WorldRef::from_ptr((*iter).world);
        empty(
            &mut EntityView::new_from(world, sys::ecs_field_src(iter, 0)),
            data_ref,
        );
    }

    sys::ecs_table_unlock((*iter).world, (*iter).table);
}

/// Callback to free the memory of the `empty` callback
pub(super) extern "C" fn on_free_empty(ptr: *mut c_void) {
    let ptr_func: *mut fn() = ptr as *mut fn();
    unsafe {
        ptr::drop_in_place(ptr_func);
    }
}

/// Callback to free the memory of the `empty_entity` callback
pub(super) extern "C" fn on_free_empty_entity(ptr: *mut c_void) {
    let ptr_func: *mut fn(&mut EntityView) = ptr as *mut fn(&mut EntityView);
    unsafe {
        ptr::drop_in_place(ptr_func);
    }
}

/// Callback to free the memory of the `payload` callback
pub(super) extern "C" fn on_free_payload<C>(ptr: *mut c_void) {
    let ptr_func: *mut fn(&mut C) = ptr as *mut fn(&mut C);
    unsafe {
        ptr::drop_in_place(ptr_func);
    }
}

/// Callback to free the memory of the `payload_entity` callback
pub(super) extern "C" fn on_free_payload_entity<C>(ptr: *mut c_void) {
    let ptr_func: *mut fn(&mut EntityView, &mut C) = ptr as *mut fn(&mut EntityView, &mut C);
    unsafe {
        ptr::drop_in_place(ptr_func);
    }
}

/// Executes the drop for the system binding context, meant to be used as a callback
pub(super) extern "C" fn binding_entity_ctx_drop(ptr: *mut c_void) {
    let ptr_struct: *mut ObserverEntityBindingCtx = ptr as *mut ObserverEntityBindingCtx;
    unsafe {
        ptr::drop_in_place(ptr_struct);
    }
}

pub(super) fn observe_impl<C, Func>(world: WorldRef<'_>, id: Entity, func: Func)
where
    Func: FnMut() + 'static,
    C: ComponentId,
{
    let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
    let binding_ctx = Box::leak(new_binding_ctx);

    let empty_func = Box::new(func);
    let empty_static_ref = Box::leak(empty_func);

    binding_ctx.empty = Some(empty_static_ref as *mut _ as *mut c_void);
    binding_ctx.free_empty = Some(on_free_empty);

    entity_observer_create(
        world.world_ptr_mut(),
        C::id(world),
        *id,
        binding_ctx,
        Some(run_empty::<Func> as unsafe extern "C" fn(_)),
    );
}

pub(super) fn observe_entity_impl<C, Func>(world: WorldRef<'_>, id: Entity, func: Func)
where
    Func: FnMut(&mut EntityView) + 'static,
    C: ComponentId,
{
    let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
    let binding_ctx = Box::leak(new_binding_ctx);

    let empty_func = Box::new(func);
    let empty_static_ref = Box::leak(empty_func);

    binding_ctx.empty_entity = Some(empty_static_ref as *mut _ as *mut c_void);
    binding_ctx.free_empty_entity = Some(on_free_empty_entity);

    entity_observer_create(
        world.world_ptr_mut(),
        C::id(world),
        *id,
        binding_ctx,
        Some(run_empty_entity::<Func> as unsafe extern "C" fn(_)),
    );
}

pub(super) fn observe_payload_impl<C, Func>(world: WorldRef<'_>, id: Entity, func: Func)
where
    Func: FnMut(&C) + 'static,
    C: ComponentId,
{
    let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
    let binding_ctx = Box::leak(new_binding_ctx);

    let empty_func = Box::new(func);
    let empty_static_ref = Box::leak(empty_func);

    binding_ctx.payload = Some(empty_static_ref as *mut _ as *mut c_void);
    binding_ctx.free_payload = Some(on_free_payload::<C>);

    entity_observer_create(
        world.world_ptr_mut(),
        C::id(world),
        *id,
        binding_ctx,
        Some(run_payload::<C, Func> as unsafe extern "C" fn(_)),
    );
}

pub(super) fn observe_payload_entity_impl<C, Func>(world: WorldRef<'_>, id: Entity, func: Func)
where
    Func: FnMut(&mut EntityView, &C) + 'static,
    C: ComponentId,
{
    let new_binding_ctx = Box::<ObserverEntityBindingCtx>::default();
    let binding_ctx = Box::leak(new_binding_ctx);

    let empty_func = Box::new(func);
    let empty_static_ref = Box::leak(empty_func);

    binding_ctx.payload_entity = Some(empty_static_ref as *mut _ as *mut c_void);
    binding_ctx.free_payload_entity = Some(on_free_payload_entity::<C>);

    entity_observer_create(
        world.world_ptr_mut(),
        C::id(world),
        *id,
        binding_ctx,
        Some(run_payload_entity::<C, Func> as unsafe extern "C" fn(_)),
    );
}
