use crate::prelude::*;
use crate::sys;

/// Register opaque type interface
impl<'a, T: 'static> Component<'a, T> {
    pub fn opaque_func<Func>(&self, func: Func) -> &Self
    where
        Func: FnOnce(WorldRef<'a>) -> Opaque<'a, T>,
    {
        let mut opaque = func(self.world());
        opaque.desc.entity = self.world().component_id_map::<T>();
        unsafe { sys::ecs_opaque_init(self.world_ptr_mut(), &opaque.desc) };
        self
    }

    pub fn opaque_func_id<Func, Elem>(&self, id: impl Into<Entity>, func: Func) -> &Self
    where
        Func: FnOnce(WorldRef<'a>) -> Opaque<'a, T, Elem>,
    {
        let mut opaque = func(self.world());
        opaque.desc.entity = *id.into();
        unsafe { sys::ecs_opaque_init(self.world_ptr_mut(), &opaque.desc) };
        self
    }

    pub fn opaque<Type: 'static>(&self) -> Opaque<'a, T> {
        let id = self.world().component_id_map::<Type>();
        let mut opaque = Opaque::<T>::new(self.world());
        opaque.as_type(id);
        opaque
    }

    pub fn opaque_id(&self, id: impl IntoEntity) -> Opaque<'a, T> {
        let id = id.into_entity(self.world());
        let mut opaque = Opaque::<T>::new(self.world());
        opaque.as_type(id);
        opaque
    }

    pub fn opaque_dyn_id<E>(&self, id_type: E, id_field: E) -> Opaque<'a, T>
    where
        E: Into<Entity> + Copy,
    {
        let mut opaque = Opaque::<T>::new_id(self.world(), id_type);
        opaque.as_type(id_field);
        opaque
    }

    /// Return opaque type builder for collection type
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// #[derive(Component)]
    /// struct SerVec {
    ///     pub value: Vec<i32>,
    /// }
    ///
    /// world
    ///     .component::<SerVec>()
    ///     .opaque_collection_vector::<i32>();
    /// ```
    pub fn opaque_collection_vector<ElemType: 'static>(&self) -> Opaque<'a, T, ElemType> {
        let world = self.world();
        let mut opaque = Opaque::<T, ElemType>::new(self.world());
        let id = world.vector::<ElemType>();
        opaque.as_type(id);
        opaque
    }

    /// Return opaque type builder for collection type
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    /// let world = World::new();
    ///
    /// #[derive(Component)]
    /// struct SerVec {
    ///     pub value: Vec<i32>,
    /// }
    ///
    /// world
    ///     .component::<SerVec>()
    ///     .opaque_collection_dyn::<i32>(world.vector::<i32>());
    /// ```
    pub fn opaque_collection_dyn<ElemType>(
        &self,
        id: impl Into<Entity>,
    ) -> Opaque<'a, T, ElemType> {
        let id: Entity = id.into();
        let copy_id = id;
        let mut opaque = Opaque::<T, ElemType>::new_id(self.world(), self.id);
        opaque.as_type(copy_id);
        opaque
    }
}

impl<T: EnumComponentInfo + 'static> Component<'_, T> {
    /// Add constant.
    pub fn constant(&self, name: &str, value: T) -> &Self {
        unsafe { sys::ecs_add_id(self.world_ptr_mut(), *self.id, flecs::meta::EcsEnum::ID) };

        let name = compact_str::format_compact!("{}\0", name);

        let desc = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            parent: *self.id,
            ..Default::default()
        };

        let eid = unsafe { sys::ecs_entity_init(self.world_ptr_mut(), &desc) };

        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InternalError,
            "failed to create entity"
        );

        let id = self.world().component_id::<T::UnderlyingTypeOfEnum>();

        let pair = ecs_pair(flecs::Constant::ID, *id);

        unsafe {
            let size = const { core::mem::size_of::<T::UnderlyingTypeOfEnum>() };
            let ptr = sys::ecs_ensure_id(self.world_ptr_mut(), eid, pair, size)
                as *mut T::UnderlyingTypeOfEnum;
            *ptr = *(&value as *const T as *const <T as ComponentId>::UnderlyingTypeOfEnum);
            sys::ecs_modified_id(self.world_ptr_mut(), eid, pair);
        }
        self
    }
}
