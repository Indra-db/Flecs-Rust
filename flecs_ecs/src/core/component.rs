//! Registering and working with components

use crate::{
    addons::meta::Opaque,
    core::{c_binding::bindings::ecs_set_hooks_id, get_full_type_name, FlecsErrorCode},
    ecs_assert,
};

use super::{
    c_binding::bindings::{ecs_get_hooks_id, ecs_opaque_init},
    c_types::*,
    component_registration::*,
    ecs_field,
    entity::Entity,
    World,
};

use std::{ffi::CStr, os::raw::c_void, ptr};

use std::{marker::PhantomData, ops::Deref};

type EcsCtxFreeT = extern "C" fn(*mut c_void);

struct ComponentBindingCtx {
    on_add: Option<*mut c_void>,
    on_remove: Option<*mut c_void>,
    on_set: Option<*mut c_void>,
    free_on_add: Option<EcsCtxFreeT>,
    free_on_remove: Option<EcsCtxFreeT>,
    free_on_set: Option<EcsCtxFreeT>,
}

impl Drop for ComponentBindingCtx {
    fn drop(&mut self) {
        if let Some(on_add) = self.on_add {
            if let Some(free_on_add) = self.free_on_add {
                free_on_add(on_add);
            }
        }
        if let Some(on_remove) = self.on_remove {
            if let Some(free_on_remove) = self.free_on_remove {
                free_on_remove(on_remove);
            }
        }
        if let Some(on_set) = self.on_set {
            if let Some(free_on_set) = self.free_on_set {
                free_on_set(on_set);
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ComponentBindingCtx {
    fn default() -> Self {
        Self {
            on_add: None,
            on_remove: None,
            on_set: None,
            free_on_add: None,
            free_on_remove: None,
            free_on_set: None,
        }
    }
}
impl ComponentBindingCtx {
    pub fn new(
        on_add: Option<*mut c_void>,
        on_remove: Option<*mut c_void>,
        on_set: Option<*mut c_void>,
        free_on_add: Option<EcsCtxFreeT>,
        free_on_remove: Option<EcsCtxFreeT>,
        free_on_set: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            on_add,
            on_remove,
            on_set,
            free_on_add,
            free_on_remove,
            free_on_set,
        }
    }
}

/// Untyped component class.
pub struct UntypedComponent {
    pub entity: Entity,
}

impl Deref for UntypedComponent {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl UntypedComponent {
    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::untyped_component`
    #[doc(alias = "untyped_component::untyped_component")]
    pub fn new(world: &World, id: IdT) -> Self {
        UntypedComponent {
            entity: Entity::new_from_existing_raw(world.raw_world, id),
        }
    }
}

#[cfg(feature = "flecs_meta")]
impl UntypedComponent {}

#[cfg(feature = "flecs_metrics")]
impl UntypedComponent {}

/// Component class.
/// Class used to register components and component metadata.
pub struct Component<T: CachedComponentData + Default> {
    pub base: UntypedComponent,
    _marker: PhantomData<T>,
}

impl<T: CachedComponentData + Default> Deref for Component<T> {
    type Target = UntypedComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<T: CachedComponentData + Default> Component<T> {
    /// Create a new component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    ///
    /// # See also
    ///
    /// * C++ API: `component::component`
    #[doc(alias = "component::component")]
    pub fn new(world: &World) -> Self {
        if !T::is_registered_with_world(world.raw_world) {
            T::register_explicit(world.raw_world);
        }

        Self {
            base: UntypedComponent::new(world, unsafe { T::get_id_unchecked() }),
            _marker: PhantomData,
        }
    }

    /// Create a new component with a name.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `name`: the name of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `component::component`
    #[doc(alias = "component::component")]
    pub fn new_named(world: &World, name: &CStr) -> Self {
        if !T::is_registered_with_world(world.raw_world) {
            T::register_explicit_named(world.raw_world, name);
        }

        Self {
            base: UntypedComponent::new(world, unsafe { T::get_id_unchecked() }),
            _marker: PhantomData,
        }
    }

    /// Get the binding context for the component.
    ///
    /// # Arguments
    ///
    /// * `type_hooks`: the type hooks.
    ///
    /// # See also
    ///
    /// * C++ API: `component::get_binding_ctx`
    #[doc(alias = "component::get_binding_ctx")]
    fn get_binding_ctx(&mut self, type_hooks: &mut TypeHooksT) -> &mut ComponentBindingCtx {
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
    ///
    /// # See also
    ///
    /// * C++ API: `component::get_hooks`
    #[doc(alias = "component::get_hooks")]
    fn get_hooks(&self) -> TypeHooksT {
        let type_hooks: *const TypeHooksT = unsafe { ecs_get_hooks_id(self.world, self.raw_id) };
        if type_hooks.is_null() {
            TypeHooksT::default()
        } else {
            unsafe { *type_hooks }
        }
    }

    /// Function to free the binding context.
    ///
    /// # See also
    ///
    /// * C++ API: `component::binding_ctx_free`
    #[doc(alias = "component::binding_ctx_free")]
    extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ComponentBindingCtx = ptr as *mut ComponentBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }

    /// Register on add hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_add`
    #[doc(alias = "component::on_add")]
    pub fn on_add<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let mut type_hooks: TypeHooksT = self.get_hooks();

        ecs_assert!(
            type_hooks.on_add.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_add hook already set for component {}",
            get_full_type_name::<T>()
        );

        let binding_ctx = self.get_binding_ctx(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_add = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_add = Some(Self::on_add_drop::<Func>);
        type_hooks.on_add = Some(Self::run_add::<Func>);
        unsafe { ecs_set_hooks_id(self.world, self.raw_id, &type_hooks) };
        self
    }

    /// Register on remove hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_remove`
    #[doc(alias = "component::on_remove")]
    pub fn on_remove<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let mut type_hooks = self.get_hooks();

        ecs_assert!(
            type_hooks.on_remove.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_remove hook already set for component {}",
            get_full_type_name::<T>()
        );

        let binding_ctx = self.get_binding_ctx(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_remove = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_remove = Some(Self::on_remove_drop::<Func>);
        type_hooks.on_remove = Some(Self::run_remove::<Func>);
        unsafe { ecs_set_hooks_id(self.world, self.raw_id, &type_hooks) };
        self
    }

    /// Register on set hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_set`
    #[doc(alias = "component::on_set")]
    pub fn on_set<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let mut type_hooks = self.get_hooks();

        ecs_assert!(
            type_hooks.on_set.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_set hook already set for component {}",
            get_full_type_name::<T>()
        );

        let binding_ctx = self.get_binding_ctx(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_set = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_set = Some(Self::on_set_drop::<Func>);
        type_hooks.on_set = Some(Self::run_set::<Func>);
        unsafe { ecs_set_hooks_id(self.world, self.raw_id, &type_hooks) };
        self
    }

    /// Function to free the on add hook.
    extern "C" fn on_add_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on remove hook.
    extern "C" fn on_remove_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on set hook.
    extern "C" fn on_set_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to run the on add hook.
    extern "C" fn run_add<Func>(iter: *mut IterT)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = unsafe { (*iter).binding_ctx as *mut _ };
        let on_add = unsafe { (*ctx).on_add.unwrap() };
        let on_add = on_add as *mut Func;
        let on_add = unsafe { &mut *on_add };
        let entity = unsafe { Entity::new_from_existing_raw((*iter).world, *(*iter).entities) };
        let component: *mut T = unsafe { ecs_field::<T>(iter, 1) };
        on_add(entity, unsafe { &mut *component });
    }

    /// Function to run the on set hook.
    extern "C" fn run_set<Func>(iter: *mut IterT)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = unsafe { (*iter).binding_ctx as *mut _ };
        let on_set = unsafe { (*ctx).on_set.unwrap() };
        let on_set = on_set as *mut Func;
        let on_set = unsafe { &mut *on_set };
        let entity = unsafe { Entity::new_from_existing_raw((*iter).world, *(*iter).entities) };
        let component: *mut T = unsafe { ecs_field::<T>(iter, 1) };
        on_set(entity, unsafe { &mut *component });
    }

    /// Function to run the on remove hook.
    extern "C" fn run_remove<Func>(iter: *mut IterT)
    where
        Func: FnMut(Entity, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = unsafe { (*iter).binding_ctx as *mut _ };
        let on_remove = unsafe { (*ctx).on_remove.unwrap() };
        let on_remove = on_remove as *mut Func;
        let on_remove = unsafe { &mut *on_remove };
        let entity = unsafe { Entity::new_from_existing_raw((*iter).world, *(*iter).entities) };
        let component: *mut T = unsafe { ecs_field::<T>(iter, 1) };
        on_remove(entity, unsafe { &mut *component });
    }
}

#[cfg(feature = "flecs_meta")]
impl<T: CachedComponentData> Component<T> {
    // todo!("Check if this is correctly ported")
    /// # See also
    ///
    /// * C++ API: `component::opque`
    #[doc(alias = "component::opque")]
    pub fn opaque<OpaqueType>(&mut self) -> &mut Self
    where
        OpaqueType: CachedComponentData,
    {
        let mut ts = Opaque::<OpaqueType>::new(self.world);
        ts.desc.entity = T::get_id(self.world);
        unsafe { ecs_opaque_init(self.world, &ts.desc) };
        self
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_entity_id_type(&mut self, as_type: EntityT) -> Opaque<T> {
        let mut opaque = Opaque::<T>::new(self.world);
        opaque.as_type(as_type);
        opaque
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_entity_type(&mut self, as_type: Entity) -> Opaque<T> {
        self.opaque_entity_id_type(as_type.raw_id)
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_untyped_component(&mut self, as_type: UntypedComponent) -> Opaque<T> {
        self.opaque_entity_id_type(as_type.raw_id)
    }

    //todo!("untyped component constant function")
}
