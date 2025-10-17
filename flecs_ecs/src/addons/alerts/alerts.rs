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
