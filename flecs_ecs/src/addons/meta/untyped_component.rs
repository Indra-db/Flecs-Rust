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
    /// (name: &'a str,),
    /// (name: &'a str, count: i32),
    /// (name: &'a str, count: i32, offset: i32)
    pub fn member_unit<'a, Meta: MetaMember<'a>>(
        self,
        type_id: impl IntoEntity,
        unit: impl IntoEntity,
        data: Meta,
    ) -> Self {
        let name = compact_str::format_compact!("{}\0", data.name());
        let world = self.world_ptr_mut();
        let id = *self.id;
        let type_id = *type_id.into_entity(self.world());
        let unit = *unit.into_entity(self.world());

        let member = sys::ecs_member_t {
            name: name.as_ptr() as *const _,
            type_: type_id,
            unit,
            count: data.count(),
            offset: data.offset(),
            use_offset: Meta::USE_OFFSET,
            ..Default::default()
        };

        let _result = unsafe { sys::ecs_struct_add_member(world, id, &member) };
        ecs_assert!(
            _result == 0,
            FlecsErrorCode::InternalError,
            "failed to add member to struct"
        );
        self
    }

    /// Add member.
    ///
    /// [`MetaMember`] is a trait that accepts the following options:
    /// (name: &'a str,),
    /// (name: &'a str, count: i32),
    /// (name: &'a str, count: i32, offset: i32)
    pub fn member<'a>(self, type_id: impl IntoEntity, data: impl MetaMember<'a>) -> Self {
        self.member_unit(type_id, 0, data)
    }

    /// Create child entities for the members of this struct type, equivalent
    /// to the C `ecs_struct_desc_t::create_member_entities` opt-in. Flecs no
    /// longer creates member entities by default.
    ///
    /// Member entities are required for member value queries, e.g.
    /// `Thrusters.left($this, $thruster)`.
    ///
    /// Flecs drops `use_offset` when it converts a member entity back to a
    /// struct member, so an explicit offset-0 member would trigger an
    /// auto-relayout that overwrites all explicit (`repr(Rust)`) offsets. To
    /// avoid this, offset-0 members are created with a temporary in-bounds
    /// offset; afterwards the original offsets are restored and the type
    /// serializer is rebuilt if they changed.
    pub fn create_member_entities(self) -> Self {
        let world = self.world_ptr_mut();
        let id = *self.id;

        struct MemberSnapshot {
            name: compact_str::CompactString,
            type_: sys::ecs_entity_t,
            unit: sys::ecs_entity_t,
            count: i32,
            offset: i32,
            range: sys::ecs_member_value_range_t,
            warning_range: sys::ecs_member_value_range_t,
            error_range: sys::ecs_member_value_range_t,
            size: i32,
            has_entity: bool,
        }

        let mut members: alloc::vec::Vec<MemberSnapshot> = alloc::vec::Vec::new();
        let mut i = 0;
        loop {
            let m = unsafe { sys::ecs_struct_get_nth_member(world, id, i) };
            if m.is_null() {
                break;
            }
            let m = unsafe { &*m };
            let name = unsafe { core::ffi::CStr::from_ptr(m.name) };
            members.push(MemberSnapshot {
                name: compact_str::format_compact!("{}\0", name.to_string_lossy()),
                type_: m.type_,
                unit: m.unit,
                count: m.count,
                offset: m.offset,
                range: m.range,
                warning_range: m.warning_range,
                error_range: m.error_range,
                size: m.size,
                has_entity: m.member != 0,
            });
            i += 1;
        }
        members.sort_by_key(|m| m.offset);

        let struct_size = unsafe {
            let ptr = sys::ecs_get_id(world, id, flecs::Component::ID) as *const sys::EcsComponent;
            if ptr.is_null() { 0 } else { (*ptr).size }
        };

        for member in members.iter().filter(|m| !m.has_entity) {
            let desc = sys::ecs_entity_desc_t {
                name: member.name.as_ptr() as *const _,
                parent: id,
                ..Default::default()
            };
            let member_entity = unsafe { sys::ecs_entity_init(world, &desc) };
            ecs_assert!(
                member_entity != 0,
                FlecsErrorCode::InternalError,
                "failed to create member entity"
            );

            let offset = if member.offset == 0 && struct_size > member.size {
                struct_size - member.size
            } else {
                member.offset
            };
            let data = sys::EcsMember {
                type_: member.type_,
                unit: member.unit,
                count: member.count,
                offset,
                use_offset: true,
            };
            unsafe {
                sys::ecs_set_id(
                    world,
                    member_entity,
                    flecs::meta::Member::ID,
                    core::mem::size_of::<sys::EcsMember>(),
                    &data as *const sys::EcsMember as *const c_void,
                );
            }

            let has_ranges = |r: &sys::ecs_member_value_range_t| r.min != 0.0 || r.max != 0.0;
            if has_ranges(&member.range)
                || has_ranges(&member.warning_range)
                || has_ranges(&member.error_range)
            {
                let ranges = sys::EcsMemberRanges {
                    value: member.range,
                    warning: member.warning_range,
                    error: member.error_range,
                };
                unsafe {
                    sys::ecs_set_id(
                        world,
                        member_entity,
                        flecs::meta::MemberRanges::ID,
                        core::mem::size_of::<sys::EcsMemberRanges>(),
                        &ranges as *const sys::EcsMemberRanges as *const c_void,
                    );
                }
            }
        }

        let mut offsets_changed = false;
        for member in &members {
            let m =
                unsafe { sys::ecs_struct_get_member(world, id, member.name.as_ptr() as *const _) };
            if m.is_null() || unsafe { (*m).offset } == member.offset {
                continue;
            }
            offsets_changed = true;
            unsafe {
                (*m).offset = member.offset;
                (*m).use_offset = true;
                let member_entity = (*m).member;
                if member_entity != 0 {
                    let ptr = sys::ecs_get_mut_id(world, member_entity, flecs::meta::Member::ID)
                        as *mut sys::EcsMember;
                    if !ptr.is_null() {
                        (*ptr).offset = member.offset;
                        (*ptr).use_offset = true;
                    }
                }
            }
        }
        if offsets_changed {
            unsafe { sys::ecs_modified_id(world, id, flecs::meta::Type::ID) };
        }
        self
    }

    /// Add member with unit typed.
    ///
    /// [`MetaMember`] is a trait that accepts the following options:
    /// (name: &'satatic str,),
    /// (name: &'a str, count: i32),
    /// (name: &'a str, count: i32, offset: i32)
    pub fn member_unit_type<'a, T: ComponentId, U: ComponentId>(
        self,
        data: impl MetaMember<'a>,
    ) -> Self {
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

        unsafe {
            (*m).range.min = min;
            (*m).range.max = max;
        }

        let member_entity = unsafe { (*m).member };
        if member_entity != 0 {
            let world_ptr = self.world_ptr_mut();
            let w = unsafe { WorldRef::from_ptr(world_ptr) };
            let me = w.entity_from_id(member_entity);

            let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
            let ptr = unsafe {
                sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
            };
            assert!(!ptr.is_null(), "failed to ensure MemberRanges component");
            let mr = unsafe { &mut *(ptr as *mut flecs::meta::MemberRanges) };

            mr.value.min = min;
            mr.value.max = max;
            me.modified(flecs::meta::MemberRanges::ID);
        }
        self
    }

    /// add member warning range
    pub fn warning_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        unsafe {
            (*m).warning_range.min = min;
            (*m).warning_range.max = max;
        }

        let member_entity = unsafe { (*m).member };
        if member_entity != 0 {
            let world_ptr = self.world_ptr_mut();
            let w = unsafe { WorldRef::from_ptr(world_ptr) };
            let me = w.entity_from_id(member_entity);

            let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
            let ptr = unsafe {
                sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
            };
            assert!(!ptr.is_null(), "failed to ensure MemberRanges component");
            let mr = unsafe { &mut *(ptr as *mut flecs::meta::MemberRanges) };

            mr.warning.min = min;
            mr.warning.max = max;
            me.modified(flecs::meta::MemberRanges::ID);
        }
        self
    }

    /// add member error range
    pub fn error_range(self, min: f64, max: f64) -> Self {
        let m = unsafe { sys::ecs_cpp_last_member(self.world_ptr(), *self.id) };
        if m.is_null() {
            return self;
        }

        unsafe {
            (*m).error_range.min = min;
            (*m).error_range.max = max;
        }

        let member_entity = unsafe { (*m).member };
        if member_entity != 0 {
            let world_ptr = self.world_ptr_mut();
            let w = unsafe { WorldRef::from_ptr(world_ptr) };
            let me = w.entity_from_id(member_entity);

            let size = const { core::mem::size_of::<flecs::meta::MemberRanges>() };
            let ptr = unsafe {
                sys::ecs_ensure_id(world_ptr, *me.id, flecs::meta::MemberRanges::ID, size)
            };
            assert!(!ptr.is_null(), "failed to ensure MemberRanges component");
            let mr = unsafe { &mut *(ptr as *mut flecs::meta::MemberRanges) };

            mr.error.min = min;
            mr.error.max = max;
            me.modified(flecs::meta::MemberRanges::ID);
        }
        self
    }
}
