use crate::prelude::*;
use crate::sys;

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
