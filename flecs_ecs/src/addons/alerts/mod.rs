mod alert_builder;
pub use alert_builder::*;
mod module;
pub use module::*;
mod types;
pub use types::*;

// TODO flecs core: add missing stuff in modules + alert_count should not default to 0 parameter, see ecs_check

use core::ops::Deref;
use core::ops::DerefMut;

use crate::core::*;
use crate::sys;

#[derive(Clone, Copy)]
pub struct Alert<'a> {
    pub(crate) entity: EntityView<'a>,
}

impl<'a> Deref for Alert<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl DerefMut for Alert<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a> WorldProvider<'a> for Alert<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> Alert<'a> {
    //todo!() in query etc desc is a pointer, does it need to be?
    /// Create a new alert
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the alert in.
    /// * `desc` - The alert description.
    pub fn new(world: impl WorldProvider<'a>, desc: sys::ecs_alert_desc_t) -> Self {
        let id = unsafe { sys::ecs_alert_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        Self { entity }
    }

    /// Wrap an existing alert entity in a alert object
    ///
    /// # Arguments
    ///
    /// * `world` - The world the alert is in.
    /// * `alert_entity` - The entity of the alert.
    pub fn new_from_existing(alert_entity: EntityView<'a>) -> Self {
        Self {
            entity: alert_entity,
        }
    }
}

impl World {
    /// Constructs a `Alert` from an existing entity.
    ///
    /// This function upcasts the given `entity` to a `Alert`, assuming the entity represents a alert.
    /// The purpose is to facilitate the interaction with entities that are specifically alerts within the ECS.
    ///
    /// # Arguments
    /// * `entity` - An `EntityView` that represents a alert within the world.
    pub fn alert_from<'a>(&'a self, entity: EntityView<'a>) -> Alert<'a> {
        Alert::new_from_existing(entity)
    }

    /// Creates a new `AlertBuilder` instance for constructing alerts.
    ///
    /// This function initializes a `AlertBuilder` which is used to create alerts that match specific components.
    /// It is a generic method that works with any component types that implement the `QueryTuple` trait.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::alert_named()`]
    pub fn alert<Components>(&self) -> AlertBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        AlertBuilder::<Components>::new(self)
    }

    /// Creates a new named `AlertBuilder` instance.
    ///
    /// Similar to `alert_builder`, but allows naming the alert for easier identification and debugging.
    /// The name does not affect the alert's behavior.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the alert.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    ///
    /// # See also
    ///
    /// * [`World::alert()`]
    pub fn alert_named<'a, Components>(&'a self, name: &str) -> AlertBuilder<'a, Components>
    where
        Components: QueryTuple,
    {
        AlertBuilder::<Components>::new_named(self, name)
    }

    /// Creates a `AlertBuilder` from a alert description.
    ///
    /// This function allows creating a alert based on a predefined alert description,
    /// facilitating more dynamic or configuration-driven alert creation.
    ///
    /// # Arguments
    /// * `desc` - A alert description that outlines the parameters for the alert builder.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `QueryTuple` trait.
    pub fn alert_builder_from_desc<Components>(
        &self,
        desc: sys::ecs_alert_desc_t,
    ) -> AlertBuilder<'_, Components>
    where
        Components: QueryTuple,
    {
        AlertBuilder::<Components>::new_from_desc(self, desc)
    }
}

impl EntityView<'_> {
    /*
        /** Return number of alerts for entity.
     *
     * @memberof flecs::entity_view
     * @ingroup cpp_addons_alerts
     */
    int32_t alert_count(flecs::entity_t alert = 0) const {
        return ecs_get_alert_count(world_, id_, alert);
    }

         */

    /// Return number of alerts for entity.
    ///
    /// # Arguments
    ///
    /// * `alert` - The alert to count. If 0, counts all alerts.
    pub fn alert_count(&self, alert: impl Into<Entity>) -> i32 {
        unsafe { sys::ecs_get_alert_count(self.world_ptr(), *self.id(), *alert.into()) }
    }
}
