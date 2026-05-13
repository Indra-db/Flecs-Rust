#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unexpected_cfgs)]

use core::ops::{Deref, DerefMut};
use std::collections::HashMap;

pub use flecs_ecs::prelude::*;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }

    // Use the crash handler for integration tests
    #[cfg(feature = "test-with-crash-handler")]
    test_crash_handler::register();
}

/// RAII guard that temporarily replaces the Flecs `abort_` OS API callback
/// with a Rust panic shim for the duration of its lifetime.
///
/// This lets tests use `catch_unwind` or `#[should_panic]` for code paths
/// that would otherwise call `ecs_abort()` → SIGABRT.
///
/// The guard holds a mutex to prevent concurrent tests from racing on the
/// global OS API state. Drop restores the original handler.
///
/// # Example
/// ```ignore
/// let _guard = FlecsPanicAbortGuard::install();
/// let result = std::panic::catch_unwind(|| { /* code that calls ecs_abort */ });
/// // guard dropped here, original abort_ restored
/// assert!(result.is_err());
/// ```
pub struct FlecsPanicAbortGuard {
    original: flecs_ecs::sys::ecs_os_api_abort_t,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl FlecsPanicAbortGuard {
    pub fn install() -> Self {
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let mutex = LOCK.get_or_init(|| Mutex::new(()));
        let lock = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

        unsafe extern "C-unwind" fn panic_shim() {
            panic!("flecs abort (ecs_assert / ecs_abort fired)");
        }

        let original = unsafe {
            let mut api = flecs_ecs::sys::ecs_os_get_api();
            let orig = api.abort_;
            api.abort_ = Some(panic_shim);
            flecs_ecs::sys::ecs_os_set_api(&mut api);
            orig
        };

        FlecsPanicAbortGuard { original, _lock: lock }
    }
}

impl Drop for FlecsPanicAbortGuard {
    fn drop(&mut self) {
        unsafe {
            let mut api = flecs_ecs::sys::ecs_os_get_api();
            api.abort_ = self.original;
            flecs_ecs::sys::ecs_os_set_api(&mut api);
        }
    }
}

/// When the guard is active this is set to true so the abort() override knows
/// to suppress the call (the panic from ecs_os_abort() is already in flight).
pub(crate) static ABORT_GUARD_ACTIVE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

// Override libc abort() for the test binary only. When FlecsPanicAbortGuard is
// active, ecs_abort expands to: ecs_os_abort() [→ panic_shim → panic!()]
// followed immediately by abort() as a "satisfy compiler" no-op hint.
// Without this override, abort() kills the process before Rust unwind completes.
// This override makes abort() a no-op when the guard is active so the panic
// started by panic_shim can propagate normally.
#[unsafe(no_mangle)]
#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "C" fn abort() {
    if ABORT_GUARD_ACTIVE.load(core::sync::atomic::Ordering::SeqCst) {
        // Panic already in flight from panic_shim; suppress the redundant libc abort().
        return;
    }
    // No guard active — real abort.
    libc::raise(libc::SIGABRT);
}
#[derive(Debug, Component, Clone, Copy)]
pub struct Count(pub i32);

impl Deref for Count {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<i32> for &mut Count {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

#[derive(Component)]
pub struct QueryWrapper {
    pub query_entity: Entity,
}

#[derive(Component)]
pub struct Likes;

#[derive(Component)]
pub struct Apples;

#[derive(Component)]
pub struct Pears;

#[derive(Component)]
pub struct Eats;

#[derive(Component)]
pub struct SelfRef {
    pub value: Entity,
}

#[derive(Component)]
pub struct EntityRef {
    pub value: Entity,
}

#[derive(Component)]
pub struct SelfRef2 {
    pub value: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct Value {
    pub value: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Value2 {
    pub value: i32,
}

#[derive(Component, Clone)]
pub struct Value3 {
    pub value: i32,
}

#[derive(Component, Clone)]
pub struct Value4 {
    pub value: i32,
}
#[derive(Debug, Component, Default, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component, Default)]
#[flecs(meta)]
pub struct Point {
    x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

#[derive(Debug, Component, Default, Clone)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct MyStruct {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Other {
    pub value: i32,
}

#[derive(Component, Default)]
pub struct Mass {
    pub value: i32,
}

#[derive(Component)]
pub struct TypeA {
    pub value: i32,
}

#[derive(Component)]
pub struct Prefab {}

#[derive(Component)]
pub struct Obj {}

#[derive(Component)]
pub struct Obj2 {}

#[derive(Component)]
pub struct Rel {}

#[derive(Component, Default)]
pub struct RelFoo {
    pub foo: u32,
}

#[derive(Component)]
pub struct Alice {}

#[derive(Component)]
pub struct Bob {}

#[derive(Component, Debug)]
pub struct Tag;

#[derive(Component)]
pub struct TagA {}

#[derive(Component)]
pub struct TagB {}

#[derive(Component)]
pub struct TagC {}

#[derive(Component)]
pub struct TagD {}

#[derive(Component)]
pub struct TagE {}

#[derive(Component)]
pub struct TagF {}

#[derive(Component)]
pub struct TagG {}

#[derive(Component)]
pub struct TagH {}

#[derive(Component)]
pub struct TagI {}

#[derive(Component)]
pub struct TagJ {}

#[derive(Component)]
pub struct TagK {}

#[derive(Component)]
pub struct TagL {}

#[derive(Component)]
pub struct TagM {}

#[derive(Component)]
pub struct TagN {}

#[derive(Component)]
pub struct TagO {}

#[derive(Component)]
pub struct TagP {}

#[derive(Component)]
pub struct TagQ {}

#[derive(Component)]
pub struct TagR {}

#[derive(Component)]
pub struct TagS {}

#[derive(Component)]
pub struct TagT {}

#[derive(Component)]
pub struct TagV {}

#[derive(Component)]
pub struct TagX {}

#[derive(Component)]
pub struct Parent;

#[derive(Component)]
pub struct EntityType;

#[derive(Component)]
pub struct Base;
#[derive(Component)]
pub struct Head;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct Beam;
#[derive(Component)]
pub struct Railgun;

#[derive(Component)]
pub struct Foo;

#[derive(Component)]
pub struct Bar;

#[derive(Component)]
pub struct First;

#[derive(Component, Clone, Copy)]
pub struct Count2 {
    pub a: i32,
    pub b: i32,
}

#[derive(Component)]
pub struct Pod {
    pub value: i32,
    pub clone_count: u32,
    pub drop_count: u32,
    pub ctor_count: u32,
}

impl Default for Pod {
    fn default() -> Self {
        Pod {
            value: 0,
            clone_count: 0,
            drop_count: 0,
            ctor_count: 1,
        }
    }
}

impl Pod {
    #[allow(dead_code)]
    pub fn new(value: i32) -> Self {
        Pod {
            value,
            clone_count: 0,
            drop_count: 0,
            ctor_count: 1,
        }
    }
}

impl Clone for Pod {
    fn clone(&self) -> Self {
        Pod {
            value: self.value,
            clone_count: self.clone_count + 1,
            drop_count: 0,
            ctor_count: 0,
        }
    }
}

impl Drop for Pod {
    fn drop(&mut self) {
        self.drop_count += 1;
    }
}

#[derive(Component)]
pub struct Template<T: Send + Sync + 'static> {
    pub value: T,
}

#[derive(Component, Default)]
pub struct Templatex {
    pub value: String,
}

pub fn create_world_with_flags<T: ComponentId + Default + DataComponent + ComponentType<Struct>>()
-> World {
    let world = World::new();

    internal_register_component::<false, false, T>(&world, core::ptr::null());
    world.set(T::default());

    world
}
