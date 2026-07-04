use super::App;
use crate::core::*;

impl<'a> WorldProvider<'a> for &'a App {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
            .as_ref()
            .expect("App::run consumed the world; the App cannot be reused")
            .world()
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
    /// The app holds its own claimed world handle, so the `World` this is
    /// called on stays valid after the app quits.
    ///
    /// # See also
    ///
    /// * [`addons::app`](crate::addons::app)
    #[inline(always)]
    pub fn app(&self) -> App {
        App::new(self.clone())
    }
}
