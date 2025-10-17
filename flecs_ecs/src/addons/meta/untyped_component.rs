use crate::prelude::*;
use crate::sys;
use core::ffi::c_void;

impl UntypedComponent<'_> {
    /// Add constant.
    pub fn constant<T: ComponentId>(&self, name: &str, value: T) -> &Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world = self.world_ptr_mut();
        let id = *self.id;

        unsafe { sys::ecs_add_id(world, id, flecs::meta::EcsEnum::ID) };

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
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
                ecs_pair(flecs::Constant::ID, *self.world.component_id::<T>()),
                core::mem::size_of::<T>(),
                &value as *const T as *const c_void,
            );
        };
        self
    }

    /// Add member with unit.
    ///
    /// [`MetaMember`] is a trait that accepts the following options:
    /// (name : &'static str,),
    /// (name: &'static str, count: i32),
    /// (name: &'static str, count: i32, offset: i32)
    pub fn member_unit<Meta: MetaMember>(
        self,
        type_id: impl IntoEntity,
        unit: impl IntoEntity,
        data: Meta,
    ) -> Self {
        let name = compact_str::format_compact!("{}\0", data.name());
        let world = self.world_ptr_mut();
        let id = *self.id;
        let type_id = *type_id.into_entity(world);
        let unit = *unit.into_entity(world);

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
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
            count: data.count(),
            offset: data.offset(),
            use_offset: Meta::USE_OFFSET,
        };

        entity.set(member);
        self
    }

    /// Add member.
    ///
    /// [`MetaMember`] is a trait that accepts the following options:
    /// (name : &'static str,),
    /// (name: &'static str, count: i32),
    /// (name: &'static str, count: i32, offset: i32)
    pub fn member(self, type_id: impl IntoEntity, data: impl MetaMember) -> Self {
        self.member_unit(type_id, 0, data)
    }

    /// Add member with unit typed.
    ///
    /// [`MetaMember`] is a trait that accepts the following options:
    /// (name : &'static str,),
    /// (name: &'static str, count: i32),
    /// (name: &'static str, count: i32, offset: i32)
    pub fn member_unit_type<T: ComponentId, U: ComponentId>(self, data: impl MetaMember) -> Self {
        self.member_unit(T::get_id(self.world()), U::get_id(self.world()), data)
    }

    //TODO

    /*
    /** Add member using pointer-to-member. */
    template <typename MemberType, typename ComponentType, typename RealType = typename core::remove_extent<MemberType>::type>
    untyped_component& member(const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, name, core::extent<MemberType>::value, offset);
    }

    /** Add member with unit using pointer-to-member. */
    template <typename MemberType, typename ComponentType, typename RealType = typename core::remove_extent<MemberType>::type>
    untyped_component& member(flecs::entity_t unit, const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, unit, name, core::extent<MemberType>::value, offset);
    }

    /** Add member with unit using pointer-to-member. */
    template <typename UnitType, typename MemberType, typename ComponentType, typename RealType = typename core::remove_extent<MemberType>::type>
    untyped_component& member(const char* name, const MemberType ComponentType::* ptr) {
        flecs::entity_t type_id = _::type<RealType>::id(world_);
        flecs::entity_t unit_id = _::type<UnitType>::id(world_);
        size_t offset = reinterpret_cast<size_t>(&(static_cast<ComponentType*>(nullptr)->*ptr));
        return member(type_id, unit_id, name, core::extent<MemberType>::value, offset);
             */

    /// Add bitmask constant
    pub fn bit<T: ComponentId>(self, name: &str, value: T) -> Self {
        let name = compact_str::format_compact!("{}\0", name);
        let world = self.world_ptr_mut();
        let id = *self.id;

        unsafe { sys::ecs_add_id(world, id, flecs::meta::Bitmask::ID) };

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
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
                ecs_pair(flecs::Constant::ID, *self.world.component_id::<T>()),
                core::mem::size_of::<T>(),
                &value as *const T as *const c_void,
            );
        };
        self
    }

    /// register array metadata for component
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
    pub fn range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
                as *mut flecs::meta::MemberRanges)
        };

        mr.value.min = min;
        mr.value.max = max;
        me.modified(flecs::meta::MemberRanges::ID);
        self
    }

    /// add member warning range
    pub fn warning_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
                as *mut flecs::meta::MemberRanges)
        };

        mr.warning.min = min;
        mr.warning.max = max;
        me.modified(flecs::meta::MemberRanges::ID);
        self
    }

    /// add member error range
    pub fn error_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        let world_ptr = self.world_ptr_mut();
        let w = unsafe { WorldRef::from_ptr(world_ptr) };
        let me = w.entity_from_id(unsafe { (*m).member });

        let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
        let mr = unsafe {
            &mut *(sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
                as *mut flecs::meta::MemberRanges)
        };

        mr.error.min = min;
        mr.error.max = max;
        me.modified(flecs::meta::MemberRanges::ID);
        self
    }
}
