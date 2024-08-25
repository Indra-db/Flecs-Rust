use super::{ecs_pair, flecs, ComponentId, Entity, EntityId, FlecsConstantId, WorldProvider};
use crate::sys;

pub trait EntityViewMeta<'w>: EntityId + WorldProvider<'w> + Sized {
    /// Make entity a unit
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::unit`
    #[doc(alias = "entity_builder::unit")]
    fn unit(
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
                entity: *self.entity_id(),
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
                entity: *self.entity_id(),
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
    fn unit_prefix(&self, symbol: &str, factor: i32, power: i32) -> &Self {
        let symbol = compact_str::format_compact!("{}\0", symbol);
        let desc = sys::ecs_unit_prefix_desc_t {
            entity: *self.entity_id(),
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
    fn quantity_id(&self, quantity: impl Into<Entity>) -> &Self {
        unsafe {
            sys::ecs_add_id(
                self.world_ptr_mut(),
                *self.entity_id(),
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
    fn quantity<T: ComponentId>(&self) -> &Self {
        self.quantity_id(T::id(self.world()))
    }

    /// Make entity a quantity
    ///
    /// # See also
    ///
    /// * C++ API: `entity_builder::quantity`
    #[doc(alias = "entity_builder::quantity")]
    fn quantity_self(&self) -> &Self {
        unsafe {
            sys::ecs_add_id(
                self.world_ptr_mut(),
                *self.entity_id(),
                flecs::meta::Quantity::ID,
            )
        };
        self
    }
}
