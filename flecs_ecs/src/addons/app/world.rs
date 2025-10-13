use super::App;
use crate::core::*;

impl<'a> WorldProvider<'a> for App<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

/// App mixin implementation
impl World {
    /// Create a new app.
    /// The app builder is a convenience wrapper around a loop that runs
    /// [`World::progress()`]. An app allows for writing platform agnostic code,
    /// as it provides hooks to modules for overtaking the main loop which is
    /// required for frameworks like emscripten.
    ///
    /// # See also
    ///
    /// * [`addons::app`](crate::addons::app)
    #[inline(always)]
    pub fn app(&self) -> App<'_> {
        App::new(self)
    }
}
