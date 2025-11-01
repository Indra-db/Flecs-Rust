use core::{
    fmt::{Debug, Display},
    ops::Deref,
};
use std::ffi::c_void;
use std::ptr;

use flecs_ecs_derive::extern_abi;
use flecs_ecs_sys::ecs_field_size;

use crate::core::*;
use crate::sys;

/// Untyped component class.
#[derive(Clone, Copy)]
pub struct UntypedComponent<'a> {
    pub entity: EntityView<'a>,
}

impl Display for UntypedComponent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.entity)
    }
}

impl Debug for UntypedComponent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.entity)
    }
}

impl<'a> Deref for UntypedComponent<'a> {
    type Target = EntityView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> UntypedComponent<'a> {
    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        let desc = crate::sys::ecs_entity_desc_t {
            name: core::ptr::null(),
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: core::ptr::null(),
            use_low_id: true,
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
        };
        let id = unsafe { crate::sys::ecs_entity_init(world.world_ptr_mut(), &desc) };

        UntypedComponent {
            entity: EntityView::new_from(world, id),
        }
    }

    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    pub(crate) fn new_named(world: impl WorldProvider<'a>, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let desc = crate::sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            _canary: 0,
            id: 0,
            parent: 0,
            symbol: core::ptr::null(),
            use_low_id: true,
            add: core::ptr::null(),
            add_expr: core::ptr::null(),
            set: core::ptr::null(),
        };
        let id = unsafe { crate::sys::ecs_entity_init(world.world_ptr_mut(), &desc) };

        UntypedComponent {
            entity: EntityView::new_from(world, id),
        }
    }

    /// Wrap an existing component into untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    pub(crate) fn new_from(world: impl WorldProvider<'a>, id: impl IntoEntity) -> Self {
        UntypedComponent {
            entity: EntityView::new_from(world, id),
        }
    }

    /// Get the id of the component.
    pub fn as_entity(&self) -> EntityView<'a> {
        self.entity
    }

    /// Function to free the binding context.
    #[extern_abi]
    unsafe fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ComponentBindingCtx = ptr as *mut ComponentBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
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

    /// Function to run the on add hook.
    #[extern_abi]
    unsafe fn run_add<Func>(iter: *mut sys::ecs_iter_t)
    where
        Func: FnMut(EntityView, *mut c_void) + 'static,
    {
        unsafe {
            let iter = &*iter;
            let ctx: *mut ComponentBindingCtx = iter.callback_ctx as *mut _;
            let on_add = (*ctx).on_add.unwrap();
            let on_add = on_add as *mut Func;
            let on_add = &mut *on_add;
            let world = WorldRef::from_ptr(iter.world);
            let entity = EntityView::new_from(world, *iter.entities);
            let size = ecs_field_size(iter, 0);
            let component = flecs_field_w_size(iter, size, 0);
            on_add(entity, component);
        }
    }

    /// Function to free the on add hook.
    #[extern_abi]
    unsafe fn on_add_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, *mut c_void) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Register on add hook.
    pub fn on_add<Func>(self, func: Func) -> Self
    where
        Func: FnMut(EntityView, *mut c_void) + 'static,
    {
        let mut type_hooks: sys::ecs_type_hooks_t = self.get_hooks();

        ecs_assert!(
            type_hooks.on_add.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_add hook already set for component {:?}",
            unsafe { self.get_name_cstr().unwrap_or(c"") }
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
}

#[cfg(feature = "flecs_meta")]
impl UntypedComponent<'_> {}

#[cfg(feature = "flecs_metrics")]
impl UntypedComponent<'_> {}

mod eq_operations {
    use super::*;

    impl<'a> PartialEq<UntypedComponent<'a>> for u64 {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            *self == other.entity.id
        }
    }

    impl PartialEq<u64> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.entity.id == *other
        }
    }

    impl PartialEq<Entity> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.entity.id == *other
        }
    }

    impl PartialEq<Id> for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.entity.id == *other
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.entity == *other
        }
    }

    impl<'a> PartialEq<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.entity == other.id
        }
    }

    impl<'a, T> PartialEq<Component<'a, T>> for UntypedComponent<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            self.entity == other.base.entity
        }
    }

    impl PartialEq for UntypedComponent<'_> {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.entity == other.entity
        }
    }

    impl Eq for UntypedComponent<'_> {}
}

mod ord_operations {
    use super::*;

    impl<'a> PartialOrd<UntypedComponent<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<core::cmp::Ordering> {
            self.partial_cmp(&other.entity.id)
        }
    }

    impl PartialOrd<u64> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl PartialOrd<Entity> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl PartialOrd<Id> for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<core::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<core::cmp::Ordering> {
            self.entity.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<core::cmp::Ordering> {
            self.entity.partial_cmp(&other.id)
        }
    }

    impl PartialOrd for UntypedComponent<'_> {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for UntypedComponent<'_> {
        #[inline]
        fn cmp(&self, other: &Self) -> core::cmp::Ordering {
            self.entity.cmp(&other.entity)
        }
    }
}
