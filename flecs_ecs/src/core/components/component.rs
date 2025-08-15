//! Registering and working with components

use core::{ffi::c_void, fmt::Debug, fmt::Display, marker::PhantomData, ops::Deref, ptr};

use crate::core::*;
#[cfg(feature = "flecs_meta")]
use crate::prelude::FetchedId;
use crate::sys;

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::boxed::Box;

/// Component class.
/// Class used to register components and component metadata.
pub struct Component<'a, T> {
    pub base: UntypedComponent<'a>,
    _marker: PhantomData<T>,
}

impl<T> Display for Component<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.base.entity)
    }
}

impl<T> Debug for Component<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.base.entity)
    }
}

impl<T> Clone for Component<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Component<'_, T> {}

impl<'a, T> Deref for Component<'a, T> {
    type Target = UntypedComponent<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, T: ComponentId> Component<'a, T> {
    /// Create a new component that is marked within Rust.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let world = world.world();
        let id = T::__register_or_get_id::<false>(world);

        let world = world.world();
        Self {
            base: UntypedComponent::new_from(world, id),
            _marker: PhantomData,
        }
    }

    /// Create a new component with a name.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `name`: the name of the component.
    pub(crate) fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        let id = T::__register_or_get_id_named::<false>(world.world(), name);

        let world = world.world();
        Self {
            base: UntypedComponent::new_from(world, id),
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn new_w_id(world: impl WorldProvider<'a>, id: impl IntoEntity) -> Self {
        Self {
            base: UntypedComponent::new_from(world, id),
            _marker: PhantomData,
        }
    }
}
impl<'a, T> Component<'a, T> {
    /// Create a new component that is not marked within Rust.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    #[cfg(feature = "flecs_meta")]
    pub(crate) fn new_id(world: impl WorldProvider<'a>, id: FetchedId<T>) -> Self {
        let world = world.world();

        Self {
            base: UntypedComponent::new_from(world, id),
            _marker: PhantomData,
        }
    }

    /// Create a new component with a name.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `name`: the name of the component.
    ///   Return the component as an entity
    #[cfg(feature = "flecs_meta")]
    pub fn new_named_id(world: impl WorldProvider<'a>, id: FetchedId<T>, name: &str) -> Self {
        let _name = compact_str::format_compact!("{}\0", name);
        let world = world.world();
        let entity = world.entity_from_id(id.id());
        entity.get_name().map_or_else(
            || {
                entity.set_name(name);
            },
            |current_name| {
                if current_name != name {
                    entity.set_name(name);
                }
            },
        );

        Self {
            base: UntypedComponent::new_from(world, id),
            _marker: PhantomData,
        }
    }
    /// Return the component as an entity
    #[inline(always)]
    pub fn entity(self) -> EntityView<'a> {
        self.base.entity
    }

    /// Get the binding context for the component.
    ///
    /// # Arguments
    ///
    /// * `type_hooks`: the type hooks.
    fn get_binding_context(type_hooks: &mut sys::ecs_type_hooks_t) -> &mut ComponentBindingCtx {
        let mut binding_ctx: *mut ComponentBindingCtx = type_hooks.binding_ctx as *mut _;

        if binding_ctx.is_null() {
            let new_binding_ctx = Box::<ComponentBindingCtx>::default();
            let static_ref = Box::leak(new_binding_ctx);
            binding_ctx = static_ref;
            type_hooks.binding_ctx = binding_ctx as *mut c_void;
            type_hooks.binding_ctx_free = Some(Self::binding_ctx_drop);
        }
        unsafe { &mut *binding_ctx }
    }

    /// Get the type hooks for the component.
    pub fn get_hooks(&self) -> sys::ecs_type_hooks_t {
        let type_hooks: *const sys::ecs_type_hooks_t =
            unsafe { sys::ecs_get_hooks_id(self.world.world_ptr(), *self.id) };
        if type_hooks.is_null() {
            sys::ecs_type_hooks_t::default()
        } else {
            unsafe { *type_hooks }
        }
    }

    /// Function to free the binding context.
    unsafe extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ComponentBindingCtx = ptr as *mut ComponentBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }

    /// Register on add hook.
    pub fn on_add<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: sys::ecs_type_hooks_t = self.get_hooks();

        ecs_assert!(
            type_hooks.on_add.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_add hook already set for component {}",
            core::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_add = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_add = Some(Self::on_add_drop::<Func>);
        type_hooks.on_add = Some(Self::run_add::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Register on remove hook.
    pub fn on_remove<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: sys::ecs_type_hooks_t = self.get_hooks();

        ecs_assert!(
            type_hooks.on_remove.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_remove hook already set for component {}",
            core::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_remove = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_remove = Some(Self::on_remove_drop::<Func>);
        type_hooks.on_remove = Some(Self::run_remove::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Register on set hook.
    pub fn on_set<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: sys::ecs_type_hooks_t = self.get_hooks();

        ecs_assert!(
            type_hooks.on_set.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_set hook already set for component {}",
            core::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_set = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_set = Some(Self::on_set_drop::<Func>);
        type_hooks.on_set = Some(Self::run_set::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Function to free the on add hook.
    unsafe extern "C" fn on_add_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on remove hook.
    unsafe extern "C" fn on_remove_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on set hook.
    unsafe extern "C" fn on_set_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to run the on add hook.
    unsafe extern "C" fn run_add<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        unsafe {
            let iter = &*iter;
            let ctx: *mut ComponentBindingCtx = iter.callback_ctx as *mut _;
            let on_add = (*ctx).on_add.unwrap();
            let on_add = on_add as *mut Func;
            let on_add = &mut *on_add;
            let world = WorldRef::from_ptr(iter.world);
            let entity = EntityView::new_from(world, *iter.entities);
            let component: *mut T = flecs_field::<T>(iter, 0);
            on_add(entity, &mut *component);
        }
    }

    /// Function to run the on set hook.
    unsafe extern "C" fn run_set<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let iter = unsafe { &*iter };
        let ctx: *mut ComponentBindingCtx = iter.callback_ctx as *mut _;
        let on_set = unsafe { (*ctx).on_set.unwrap() };
        let on_set = on_set as *mut Func;
        let on_set = unsafe { &mut *on_set };
        let world = unsafe { WorldRef::from_ptr(iter.world) };
        let entity = EntityView::new_from(world, unsafe { *iter.entities });
        let component: *mut T = flecs_field::<T>(iter, 0);
        on_set(entity, unsafe { &mut *component });
    }

    /// Function to run the on remove hook.
    unsafe extern "C" fn run_remove<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        unsafe {
            let iter = &*iter;
            let ctx: *mut ComponentBindingCtx = iter.callback_ctx as *mut _;
            let on_remove = (*ctx).on_remove.unwrap();
            let on_remove = on_remove as *mut Func;
            let on_remove = &mut *on_remove;
            let world = WorldRef::from_ptr(iter.world);
            let entity = EntityView::new_from(world, *iter.entities);
            let component: *mut T = flecs_field::<T>(iter, 0);
            on_remove(entity, &mut *component);
        }
    }
}

mod eq_operations {
    use super::*;

    impl<'a, T: ComponentId> PartialEq<Component<'a, T>> for u64 {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            *self == other.base.entity.id
        }
    }

    impl<T: ComponentId> PartialEq<u64> for Component<'_, T> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<T: ComponentId> PartialEq<Entity> for Component<'_, T> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<T: ComponentId> PartialEq<Id> for Component<'_, T> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<EntityView<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.base.entity == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<IdView<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.base.entity == other.id
        }
    }

    impl<'a, T: ComponentId> PartialEq<UntypedComponent<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            self.base.entity == other.entity
        }
    }

    impl<T: ComponentId> PartialEq for Component<'_, T> {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.base.entity == other.base.entity
        }
    }

    impl<T: ComponentId> Eq for Component<'_, T> {}

    impl<'a, T: ComponentId> PartialOrd<Component<'a, T>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &Component<'a, T>) -> Option<core::cmp::Ordering> {
            self.partial_cmp(&other.base.entity.id)
        }
    }
}

mod ord_operations {
    use super::*;
    impl<T: ComponentId> PartialOrd<u64> for Component<'_, T> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<T: ComponentId> PartialOrd<Entity> for Component<'_, T> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<core::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<T: ComponentId> PartialOrd<Id> for Component<'_, T> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<core::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<EntityView<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            self.base.entity.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<IdView<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<core::cmp::Ordering> {
            self.base.entity.partial_cmp(&other.id)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<UntypedComponent<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<core::cmp::Ordering> {
            self.base.entity.partial_cmp(&other.entity)
        }
    }

    impl<T: ComponentId> PartialOrd for Component<'_, T> {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<T: ComponentId> Ord for Component<'_, T> {
        #[inline]
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            self.base.entity.cmp(&other.base.entity)
        }
    }
}
