#![doc(hidden)]
mod component_id_fetcher;
mod cursor;
mod declarations;
mod impl_bindings;
mod impl_primitives;
mod meta_functions;
mod opaque;

use std::ffi::c_void;

pub use component_id_fetcher::*;
pub use cursor::*;
pub use declarations::*;
pub use opaque::*;

use crate::core::*;

use crate::core::ecs_assert;
use crate::sys;

impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_ext<T>(&self, id: FetchedId<T>) -> Component<T> {
        Component::<T>::new_id(self, id)
    }

    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_named_ext<'a, T>(&'a self, id: FetchedId<T>, name: &str) -> Component<'a, T> {
        Component::<T>::new_named_id(self, id, name)
    }

    /// Return meta cursor to value
    ///
    /// # See also
    ///
    /// * C++ API: `world::cursor`
    pub fn cursor_id(&self, type_id: impl Into<Entity>, ptr: *mut c_void) -> Cursor {
        Cursor::new(self, type_id, ptr)
    }

    /// Return meta cursor to value
    ///
    /// # See also
    ///
    /// * C++ API: `world::cursor`
    pub fn cursor<T: ComponentId>(&self, data: &mut T) -> Cursor {
        let type_id = T::get_id(self.world());
        Cursor::new(self, type_id, data as *mut T as *mut c_void)
    }

    /// Create primitive type
    ///
    /// # See also
    ///
    /// * C++ API: `world::primitive`
    pub fn primitive(&self, kind: EcsPrimitiveKind) -> EntityView {
        let desc = sys::ecs_primitive_desc_t {
            kind: kind as u32,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_primitive_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create primitive type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create array type
    ///
    /// # See also
    ///
    /// * C++ API: `world::array`
    pub fn array_id(&self, elem_id: impl Into<Entity>, array_count: i32) -> EntityView {
        let desc = sys::ecs_array_desc_t {
            type_: *elem_id.into(),
            count: array_count,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_array_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create array type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create array type
    ///
    /// # See also
    ///
    /// * C++ API: `world::array`
    pub fn array<T: ComponentId>(&self, array_count: i32) -> EntityView {
        self.array_id(T::get_id(self.world()), array_count)
    }

    /// Create vector type
    ///
    /// # See also
    ///
    /// * C++ API: `world::vector`
    pub fn vector_id(&self, elem_id: impl Into<Entity>) -> EntityView {
        let desc = sys::ecs_vector_desc_t {
            entity: 0u64,
            type_: *elem_id.into(),
        };

        let eid = unsafe { sys::ecs_vector_init(self.ptr_mut(), &desc) };

        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create vector type"
        );

        EntityView::new_from(self, eid)
    }

    /// Create vector type
    ///
    /// # See also
    ///
    /// * C++ API: `world::vector`
    pub fn vector<T>(&self, id: FetchedId<T>) -> EntityView {
        self.vector_id(id)
    }
}

pub trait EcsSerializer {
    fn value_id(&self, type_id: impl Into<Entity>, value: *const c_void) -> i32;
    fn value<T: ComponentId>(&self, value: &T) -> i32;
    fn member(&self, name: &str) -> i32;
}

impl EcsSerializer for sys::ecs_serializer_t {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn value_id(&self, type_id: impl Into<Entity>, value: *const c_void) -> i32 {
        if let Some(value_func) = self.value {
            unsafe { value_func(self, *type_id.into(), value) }
        } else {
            0
        }
    }

    fn value<T: ComponentId>(&self, value: &T) -> i32 {
        self.value_id(
            T::get_id(unsafe { WorldRef::from_ptr(self.world as *mut _) }),
            value as *const T as *const c_void,
        )
    }

    fn member(&self, name: &str) -> i32 {
        let name = compact_str::format_compact!("{}\0", name);
        if let Some(member_func) = self.member {
            unsafe { member_func(self, name.as_ptr() as *const i8) }
        } else {
            0
        }
    }
}

/// Register opaque type interface
impl<'a, T: 'static> Component<'a, T> {
    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_func_id<Func, Elem>(&self, id: FetchedId<T>, func: Func) -> &Self
    where
        Func: FnOnce(WorldRef<'a>) -> Opaque<'a, T, Elem>,
    {
        let mut opaque = func(self.world());
        opaque.desc.entity = id.id();
        unsafe { sys::ecs_opaque_init(self.world_ptr_mut(), &opaque.desc) };
        self
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_id(&self, id: FetchedId<T>) -> Opaque<'a, T> {
        let mut opaque = Opaque::<T>::new_id(self.world(), id);
        opaque.as_type(id.id());
        opaque
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_dyn_id<E>(&self, id_type: E, id_field: E) -> Opaque<'a, T>
    where
        E: Into<Entity> + Copy,
    {
        let mut opaque = Opaque::<T>::new_id(self.world(), FetchedId::<T>::new(*id_type.into()));
        opaque.as_type(id_field);
        opaque
    }

    /// Return opaque type builder for collection type
    /// # See also
    ///
    /// * C++ API: `component::opaque`
    pub fn opaque_collection<ElemType>(&self, id: FetchedId<T>) -> Opaque<'a, T, ElemType> {
        let mut opaque = Opaque::<T, ElemType>::new_id(self.world(), id);
        opaque.as_type(id.id());
        opaque
    }

    /// Return opaque type builder for collection type
    /// # See also
    ///
    /// * C++ API: `component::opaque`
    pub fn opaque_collection_dyn_id<ElemType>(
        &self,
        id: impl Into<Entity>,
    ) -> Opaque<'a, T, ElemType> {
        let id: Entity = id.into();
        let copy_id = id;
        let mut opaque = Opaque::<T, ElemType>::new_id(self.world(), FetchedId::<T>::new(*self.id));
        opaque.as_type(copy_id);
        opaque
    }

    /// Add constant.
    ///
    /// # See also
    ///
    /// * C++ API: `component::constant`
    pub fn constant(&self, name: &str, value: impl Into<i32>) -> &Self {
        UntypedComponent::constant(self, name, value);
        self
    }
}

/// Register opaque type interface
impl<'a, T: ComponentId> Component<'a, T> {
    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_func<Func>(&self, func: Func) -> &Self
    where
        Func: FnOnce(WorldRef<'a>) -> Opaque<'a, T>,
    {
        let mut opaque = func(self.world());
        opaque.desc.entity = T::get_id(self.world());
        unsafe { sys::ecs_opaque_init(self.world_ptr_mut(), &opaque.desc) };
        self
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    pub fn opaque<FieldId: ComponentId>(&self) -> Opaque<'a, T> {
        self.opaque_dyn_id(T::get_id(self.world()), FieldId::get_id(self.world()))
    }
}

impl<'a> UntypedComponent<'a> {
    /// Add constant.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::constant`
    pub fn constant(&self, name: &str, value: impl Into<i32>) -> &Self {
        let name = compact_str::format_compact!("{}\0", name);
        let value: i32 = value.into();
        let world = self.world_ptr_mut();
        let id = *self.id;

        unsafe { sys::ecs_add_id(world, id, flecs::meta::EcsEnum::ID) };

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const i8,
            parent: id,
            ..Default::default()
        };
        let eid = unsafe { sys::ecs_entity_init(world, &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InternalError,
            "failed to create entity"
        );

        unsafe {
            sys::ecs_set_id(
                world,
                eid,
                ecs_pair(flecs::meta::Constant::ID, flecs::meta::I32::ID),
                std::mem::size_of::<i32>(),
                &value as *const i32 as *const c_void,
            );
        };
        self
    }

    /// Add member with unit.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::member`
    pub fn member_id_unit(
        self,
        type_id: impl Into<Entity>,
        unit: impl Into<Entity>,
        name: &str,
        count: i32,
        offset: i32,
    ) -> Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world = self.world_ptr_mut();
        let id = *self.id;
        let type_id = *type_id.into();
        let unit = *unit.into();

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const i8,
            parent: id,
            ..Default::default()
        };
        let eid = unsafe { sys::ecs_entity_init(world, &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InternalError,
            "failed to create entity"
        );

        let entity = EntityView::new_from(self.world(), eid);

        let member: sys::EcsMember = sys::EcsMember {
            type_: type_id,
            unit,
            count,
            offset,
        };

        entity.set(member);
        self
    }

    /// Add member.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::member`
    pub fn member_id(
        self,
        type_id: impl Into<Entity>,
        name: &str,
        count: i32,
        offset: i32,
    ) -> Self {
        self.member_id_unit(type_id, 0, name, count, offset)
    }

    /// Add member.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::member`
    pub fn member<T: ComponentId>(self, name: &str, count: i32, offset: i32) -> Self {
        self.member_id(T::get_id(self.world()), name, count, offset)
    }

    /// Add member with unit.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::member`
    pub fn member_unit<T: ComponentId>(
        self,
        unit: impl Into<Entity>,
        name: &str,
        count: i32,
        offset: i32,
    ) -> Self {
        self.member_id_unit(T::get_id(self.world()), unit, name, count, offset)
    }

    /// Add member with unit typed.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::member`
    pub fn member_unit_type<T: ComponentId, U: ComponentId>(
        self,
        name: &str,
        count: i32,
        offset: i32,
    ) -> Self {
        self.member_id_unit(
            T::get_id(self.world()),
            U::get_id(self.world()),
            name,
            count,
            offset,
        )
    }

    //TODO

    /*
    /** Add member using pointer-to-member. */
    template <typename MemberType, typename ComponentType, typename RealType = typename std::remove_extent<MemberType>::type>
    untyped_component& member(const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, name, std::extent<MemberType>::value, offset);
    }

    /** Add member with unit using pointer-to-member. */
    template <typename MemberType, typename ComponentType, typename RealType = typename std::remove_extent<MemberType>::type>
    untyped_component& member(flecs::entity_t unit, const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, unit, name, std::extent<MemberType>::value, offset);
    }

    /** Add member with unit using pointer-to-member. */
    template <typename UnitType, typename MemberType, typename ComponentType, typename RealType = typename std::remove_extent<MemberType>::type>
    untyped_component& member(const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        flecs::entity_t unit_id = _::type<UnitType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, unit_id, name, std::extent<MemberType>::value, offset);
             */

    /// Add bitmask constant
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::bit`
    pub fn bit(self, name: &str, value: u32) -> Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world = self.world_ptr_mut();
        let id = *self.id;

        unsafe { sys::ecs_add_id(world, id, flecs::meta::Bitmask::ID) };

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const i8,
            parent: id,
            ..Default::default()
        };

        let eid = unsafe { sys::ecs_entity_init(world, &desc) };

        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InternalError,
            "failed to create entity"
        );

        unsafe {
            sys::ecs_set_id(
                world,
                eid,
                ecs_pair(flecs::meta::Constant::ID, flecs::meta::U32::ID),
                std::mem::size_of::<u32>(),
                &value as *const u32 as *const c_void,
            );
        };
        self
    }

    /// register array metadata for component
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::array`
    pub fn array<ElemType: ComponentId>(self, elem_count: i32) -> Self {
        let desc = sys::ecs_array_desc_t {
            entity: *self.id,
            type_: ElemType::get_id(self.world()),
            count: elem_count,
        };

        unsafe { sys::ecs_array_init(self.world_ptr_mut(), &desc) };
        self
    }

    /// add member value range
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::range`
    pub fn range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID)
                as *mut flecs::meta::MemberRanges)
        };

        mr.value.min = min;
        mr.value.max = max;
        me.modified::<flecs::meta::MemberRanges>();
        self
    }

    /// add member warning range
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::warning_range`
    pub fn warning_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID)
                as *mut flecs::meta::MemberRanges)
        };

        mr.warning.min = min;
        mr.warning.max = max;
        me.modified::<flecs::meta::MemberRanges>();
        self
    }

    /// add member error range
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::error_range`
    pub fn error_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID)
                as *mut flecs::meta::MemberRanges)
        };

        mr.error.min = min;
        mr.error.max = max;
        me.modified::<flecs::meta::MemberRanges>();
        self
    }
}

pub fn flecs_entity_support<'a>(world: impl WorldProvider<'a>) -> Opaque<'a, Entity> {
    let mut opaque = Opaque::<Entity>::new(world);
    opaque.as_type(flecs::meta::Entity::ID);
    opaque.serialize(|ser: &Serializer, data: &Entity| {
        let id: Id = <Entity as Into<Id>>::into(*data);
        let id: u64 = *id;
        ser.value_id(flecs::meta::Entity::ID, &id as *const u64 as *const c_void)
    });
    opaque.assign_entity(|dst: &mut Entity, _world: WorldRef<'a>, e: Entity| {
        *dst = e;
    });
    opaque
}

impl<'a> EntityView<'a> {
    /// Make entity a unit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::unit`
    #[doc(alias = "entity_builder::unit")]
    pub fn unit(
        &self,
        symbol: Option<&str>,
        prefix: impl Into<Entity>,
        base: impl Into<Entity>,
        over: impl Into<Entity>,
        factor: i32,
        power: i32,
    ) -> &Self {
        if let Some(symbol) = symbol {
            let symbol = compact_str::format_compact!("{}\0", symbol);
            let desc = sys::ecs_unit_desc_t {
                entity: *self.id,
                symbol: symbol.as_ptr() as *const i8,
                base: *base.into(),
                over: *over.into(),
                prefix: *prefix.into(),
                translation: sys::ecs_unit_translation_t { factor, power },
                quantity: 0,
            };

            unsafe { sys::ecs_unit_init(self.world_ptr_mut(), &desc) };
        } else {
            let desc = sys::ecs_unit_desc_t {
                entity: *self.id,
                symbol: std::ptr::null(),
                base: *base.into(),
                over: *over.into(),
                prefix: *prefix.into(),
                translation: sys::ecs_unit_translation_t { factor, power },
                quantity: 0,
            };

            unsafe { sys::ecs_unit_init(self.world_ptr_mut(), &desc) };
        }

        self
    }

    /// Make entity an unit prefix
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::unit_prefix`
    #[doc(alias = "entity_builder::unit_prefix")]
    pub fn unit_prefix(&self, symbol: &str, factor: i32, power: i32) -> &Self {
        let symbol = compact_str::format_compact!("{}\0", symbol);
        let desc = sys::ecs_unit_prefix_desc_t {
            entity: *self.id,
            symbol: symbol.as_ptr() as *const i8,
            translation: sys::ecs_unit_translation_t { factor, power },
        };

        unsafe { sys::ecs_unit_prefix_init(self.world_ptr_mut(), &desc) };

        self
    }

    /// Add quantity to unit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::quantity`
    pub fn quantity_id(&self, quantity: impl Into<Entity>) -> &Self {
        unsafe {
            sys::ecs_add_id(
                self.world_ptr_mut(),
                *self.id,
                ecs_pair(flecs::meta::Quantity::ID, *quantity.into()),
            );
        };
        self
    }

    /// Add quantity to unit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::quantity`
    #[doc(alias = "entity_builder::quantity")]
    pub fn quantity<T: ComponentId>(&self) -> &Self {
        self.quantity_id(T::get_id(self.world()))
    }

    /// Make entity a quantity
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::quantity`
    #[doc(alias = "entity_builder::quantity")]
    pub fn quantity_self(&self) -> &Self {
        unsafe { sys::ecs_add_id(self.world_ptr_mut(), *self.id, flecs::meta::Quantity::ID) };
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // pub type SerializeFn<T> = extern "C" fn(*const Serializer, *const T) -> i32;

    #[derive(Debug, Clone, Component)]
    struct Int {
        value: i32,
    }

    // //#[test]
    // fn test_opaque() {
    //     let world = World::new();
    //     world
    //         .component::<Int>()
    //         .opaque::<flecs::meta::I32>()
    //         .serialize(|s: &meta::Serializer, i: &Int| s.value::<i32>(&i.value));

    //     let int_type = Int { value: 10 };

    //     let json = world.to_json::<Int>(&int_type);

    //     println!("{}", json);
    //     assert_eq!("10", json);
    // }

    // #[derive(Component, Default)]
    // struct Position {
    //     x: f32,
    //     y: f32,
    // }

    // //#[test]
    // fn test_expr() {
    //     let world = World::new();

    //     world
    //         .component::<Position>()
    //         .member::<f32>("x", 1, std::mem::offset_of!(Position, x) as i32)
    //         .member::<f32>("y", 1, std::mem::offset_of!(Position, y) as i32);

    //     let e = world.entity().set(Position { x: 10.0, y: 20.0 });

    //     let pos_id = <Position as ComponentId>::id(&world);

    //     // e.get::<&Position>(|pos| {
    //     //     let expr = world.to_expr(pos);
    //     //     println!("{}", expr);
    //     // });
    // }
}
