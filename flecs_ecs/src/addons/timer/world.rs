use super::*;
use crate::core::*;
use crate::sys;

impl World {
    /// Find or register a singleton Timer
    pub fn timer(&self) -> Timer<'_> {
        Timer::new(self)
    }

    /// Find or register a Timer
    pub fn timer_from<T: ComponentId>(&self) -> Timer<'_> {
        Timer::new_from::<T>(self)
    }

    /// Enable randomizing initial time value of timers.
    /// Initializes timers with a random time value, which can improve scheduling as systems/timers
    /// for the same interval don't all happen on the same tick.
    pub fn randomize_timers(&self) {
        unsafe { sys::ecs_randomize_timers(self.ptr_mut()) }
    }
}
