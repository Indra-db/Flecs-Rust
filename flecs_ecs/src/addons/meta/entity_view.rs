use crate::prelude::*;
use crate::sys;

impl EntityView<'_> {
    /// Make entity a unit
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
                symbol: symbol.as_ptr() as *const _,
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
                symbol: core::ptr::null(),
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
    pub fn unit_prefix(&self, symbol: &str, factor: i32, power: i32) -> &Self {
        let symbol = compact_str::format_compact!("{}\0", symbol);
        let desc = sys::ecs_unit_prefix_desc_t {
            entity: *self.id,
            symbol: symbol.as_ptr() as *const _,
            translation: sys::ecs_unit_translation_t { factor, power },
        };

        unsafe { sys::ecs_unit_prefix_init(self.world_ptr_mut(), &desc) };

        self
    }

    /// Add quantity to unit
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
    pub fn quantity<T: ComponentId>(&self) -> &Self {
        self.quantity_id(T::get_id(self.world()))
    }

    /// Make entity a quantity
    pub fn quantity_self(&self) -> &Self {
        unsafe { sys::ecs_add_id(self.world_ptr_mut(), *self.id, flecs::meta::Quantity::ID) };
        self
    }
}
