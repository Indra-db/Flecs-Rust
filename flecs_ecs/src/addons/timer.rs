//! When running a pipeline, systems are ran each time [`World::progress()`](crate::core::World::progress) is called.
//! The `flecs_timer` feature addon makes it possible to run systems at a specific time interval or rate.
use core::ops::{Deref, DerefMut};

use flecs_ecs_sys::{self as sys};

use crate::core::{ComponentId, Entity, EntityView, IntoEntity, QueryTuple, World, WorldProvider};

use super::system::{System, SystemBuilder};

pub trait TimerAPI: Sized {
    fn world_ptr(&self) -> *const sys::ecs_world_t;
    fn world_ptr_mut(&self) -> *mut sys::ecs_world_t;
    fn id(&self) -> Entity;

    /// Set timer interval.
    /// This operation will continuously invoke systems associated with the timer after the interval period expires.
    /// If the entity contains an existing timer, the interval value will be reset.
    ///
    /// The timer is synchronous, and is incremented each frame by `delta_time`.
    ///
    /// The `tick_source` entity will be a tick source after this operation.
    /// Tick sources can be read by getting the [`flecs::TickSource`](crate::core::flecs::system::TickSource) component.
    /// If the tick source ticked this frame, the 'tick' member will be true.
    /// When the tick source is a system, the system will tick when the timer ticks.
    fn set_interval(self, interval: f32) -> Self {
        unsafe { sys::ecs_set_interval(self.world_ptr_mut(), *self.id(), interval) };
        self
    }

    /// Get current interval value for the specified timer.
    ///
    /// This operation returns the value set by [`set_interval()`](crate::addons::timer::TimerAPI::set_interval).
    ///
    /// # Returns
    ///
    /// The interval. If the entity is not a timer, the operation will return 0.
    fn interval(&self) -> f32 {
        unsafe { sys::ecs_get_interval(self.world_ptr(), *self.id()) }
    }

    /// Set timer timeout.
    /// This operation executes any systems associated with the timer after the specified timeout value.
    /// If the entity contains an existing timer, the timeout value will be reset.
    /// The timer can be started and stopped with [`start()`](crate::addons::timer::TimerAPI::start) and [`stop()`](crate::addons::timer::TimerAPI::stop).
    ///     
    /// The timer is synchronous, and is incremented each frame by `delta_time`.
    ///
    /// The `tick_source` entity will be a tick source after this operation.
    /// Tick sources can be read by getting the [`flecs::TickSource`](crate::core::flecs::system::TickSource) component.
    /// If the tick source ticked this frame, the 'tick' member will be true.
    /// When the tick source is a system, the system will tick when the timer ticks.
    fn set_timeout(self, timeout: f32) -> Self {
        unsafe { sys::ecs_set_timeout(self.world_ptr_mut(), *self.id(), timeout) };
        self
    }

    /// Get current timeout value for the specified timer.
    /// This operation returns the value set by [`set_timeout()`](crate::addons::timer::TimerAPI::set_timeout).
    ///
    /// After the timeout expires the [`flecs::timer::Timer`](crate::core::flecs::timer::Timer) component is removed from the entity.
    /// This means that if [`TimerAPI::timeout`] is invoked after the timer is expired, the operation will return 0.
    ///
    /// The timer is synchronous, and is incremented each frame by `delta_time`.
    ///
    /// The `tick_source` entity will be a tick source after this operation.
    /// Tick sources can be read by getting the [`flecs::TickSource`](crate::core::flecs::system::TickSource) component.
    /// If the tick source ticked this frame, the 'tick' member will be true.
    /// When the tick source is a system, the system will tick when the timer ticks.
    ///
    /// # Returns
    ///
    /// The timeout. If no timer is active for this entity, the operation returns 0.
    fn timeout(&self) -> f32 {
        unsafe { sys::ecs_get_timeout(self.world_ptr(), *self.id()) }
    }

    /// Set rate filter. Will use the frame tick as tick source,
    /// which corresponds with the number of times [`World::progress()`](crate::core::World::progress) is called.
    /// This operation initializes a rate filter.
    /// Rate filters sample tick sources and tick at a configurable multiple.
    /// A rate filter is a tick source itself, which means that rate filters can be chained.
    ///
    /// Rate filters enable deterministic system execution which cannot be achieved with interval timers alone.
    /// For example, if timer A has interval 2.0 and timer B has interval 4.0,
    /// it is not guaranteed that B will tick at exactly twice the multiple of A.
    /// This is partly due to the indeterministic nature of timers, and partly due to floating point rounding errors.
    ///
    /// Rate filters can be combined with timers (or other rate filters)
    /// to ensure that a system ticks at an exact multiple of a tick source (which can be another system).
    /// If a rate filter is created with a rate of 1 it will tick at the exact same time as its source.
    ///
    /// The `tick_source` entity will be a tick source after this operation.
    /// Tick sources can be read by getting the [`flecs::TickSource`](crate::core::flecs::system::TickSource) component.
    /// If the tick source ticked this frame, the 'tick' member will be true.
    /// When the tick source is a system, the system will tick when the timer ticks.
    ///
    /// # See also
    ///
    /// * [`TimerAPI::set_rate_w_tick_source()`]
    fn set_rate(self, rate: i32) -> Self {
        unsafe { sys::ecs_set_rate(self.world_ptr_mut(), *self.id(), rate, 0) };
        self
    }

    /// Set rate filter.
    /// This operation initializes a rate filter.
    /// Rate filters sample tick sources and tick at a configurable multiple.
    /// A rate filter is a tick source itself, which means that rate filters can be chained.
    ///
    /// Rate filters enable deterministic system execution which cannot be achieved with interval timers alone.
    /// For example, if timer A has interval 2.0 and timer B has interval 4.0,
    /// it is not guaranteed that B will tick at exactly twice the multiple of A.
    /// This is partly due to the indeterministic nature of timers, and partly due to floating point rounding errors.
    ///
    /// Rate filters can be combined with timers (or other rate filters)
    /// to ensure that a system ticks at an exact multiple of a tick source (which can be another system).
    /// If a rate filter is created with a rate of 1 it will tick at the exact same time as its source.
    ///
    /// If no tick source is provided (Entity(0)), the rate filter will use the frame tick as source,
    /// which corresponds with the number of times [`World::progress()`](crate::core::World::progress) is called.
    ///
    /// The `tick_source` entity will be a tick source after this operation.
    /// Tick sources can be read by getting the [`flecs::TickSource`](crate::core::flecs::system::TickSource) component.
    /// If the tick source ticked this frame, the 'tick' member will be true.
    /// When the tick source is a system, the system will tick when the timer ticks.
    /// # See also
    ///
    /// * [`TimerAPI::set_rate()`]
    fn set_rate_w_tick_source(self, rate: i32, tick_source: impl Into<Entity>) -> Self {
        unsafe { sys::ecs_set_rate(self.world_ptr_mut(), *self.id(), rate, *tick_source.into()) };
        self
    }

    /// Start timer.
    /// This operation resets the timer and starts it with the specified timeout.
    fn start(&self) {
        unsafe { sys::ecs_start_timer(self.world_ptr_mut(), *self.id()) };
    }

    /// Stop timer.
    /// This operation stops a timer from triggering.
    fn stop(&self) {
        unsafe { sys::ecs_stop_timer(self.world_ptr_mut(), *self.id()) };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Timer<'a> {
    entity: EntityView<'a>,
}

impl<'a> Deref for Timer<'a> {
    type Target = EntityView<'a>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl DerefMut for Timer<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl From<Timer<'_>> for Entity {
    #[inline]
    fn from(timer: Timer) -> Self {
        timer.id
    }
}

impl<'a> Timer<'a> {
    pub(crate) fn new(world: impl WorldProvider<'a>) -> Self {
        Timer {
            entity: EntityView::new(world),
        }
    }

    pub(crate) fn new_from<T: ComponentId>(world: impl WorldProvider<'a>) -> Self {
        Timer {
            entity: EntityView::new_from(world.world(), T::entity_id(world)),
        }
    }
}

impl TimerAPI for Timer<'_> {
    fn world_ptr(&self) -> *const flecs_ecs_sys::ecs_world_t {
        self.entity.world_ptr()
    }

    fn world_ptr_mut(&self) -> *mut flecs_ecs_sys::ecs_world_t {
        self.entity.world_ptr_mut()
    }

    fn id(&self) -> Entity {
        self.id
    }
}

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

impl TimerAPI for System<'_> {
    fn world_ptr(&self) -> *const flecs_ecs_sys::ecs_world_t {
        self.entity.world_ptr()
    }

    fn world_ptr_mut(&self) -> *mut flecs_ecs_sys::ecs_world_t {
        self.entity.world_ptr_mut()
    }

    fn id(&self) -> Entity {
        self.id
    }
}

impl System<'_> {
    /// Assign tick source to system based on an id.
    /// Systems can be their own tick source, which can be any of the tick sources (one shot timers, interval times and rate filters).
    /// However, in some cases it is must be guaranteed that different systems tick on the exact same frame.
    ///
    /// This cannot be guaranteed by giving two systems the same interval/rate filter as it is possible
    /// that one system is (for example) disabled, which would cause the systems to go out of sync.
    /// To provide these guarantees, systems must use the same tick source, which is what this operation enables.
    ///
    /// When two systems share the same tick source, it is guaranteed that they tick in the same frame.
    /// The provided tick source can be any entity that is a tick source, including another system.
    /// If the provided entity is not a tick source the system will not be ran.
    ///
    /// To disassociate a tick source from a system, use [`System::reset_tick_source()`](crate::addons::system::System::reset_tick_source).
    pub fn set_tick_source(&self, id: impl IntoEntity) {
        unsafe {
            sys::ecs_set_tick_source(
                self.entity.world_ptr_mut(),
                *self.id,
                *id.into_entity(self.world),
            );
        }
    }

    /// Reset, disassociate a tick source from a system
    pub fn reset_tick_source(&self) {
        unsafe { sys::ecs_set_tick_source(self.entity.world_ptr_mut(), *self.id, 0) }
    }
}

impl<T: QueryTuple> SystemBuilder<'_, T> {
    /// Set system interval.
    ///
    /// This operation will cause the system to be ran at the specified interval.
    ///
    /// The timer is synchronous, and is incremented each frame by `delta_time`.
    pub fn set_interval(&mut self, interval: f32) -> &mut Self {
        self.desc.interval = interval;
        self
    }

    /// Sets a rate filter on the system, causing it to run once every `rate`
    /// ticks. The tick source may be any entity, including another system.
    pub fn set_tick_source_rate(&mut self, tick_source: impl Into<Entity>, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self.desc.tick_source = *tick_source.into();
        self
    }

    /// Sets a rate filter on the system, causing it to run once every `rate`
    /// ticks. If a tick source was provided, this just updates the rate of the
    /// system.
    pub fn set_rate(&mut self, rate: i32) -> &mut Self {
        self.desc.rate = rate;
        self
    }

    /// Set tick source.
    /// This operation sets a shared tick source for the system.
    pub fn set_tick_source(&mut self, tick_source: impl IntoEntity) -> &mut Self {
        self.desc.tick_source = *tick_source.into_entity(self.world());
        self
    }
}
