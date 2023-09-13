use std::sync::OnceLock;

use flecs_ecs_bridge::core::component::{test, CachedComponentData, ComponentData};
use flecs_ecs_bridge::core::world::World;
#[macro_use]
extern crate debug_here;

#[derive(Clone, Default)]
struct Test {
    x: i32,
    v: Vec<i32>,
}

impl Drop for Test {
    fn drop(&mut self) {
        println!("dropped");
    }
}
#[derive(Clone, Default)]
struct Test1 {
    x: i32,
    v: Test,
}

#[derive(Clone, Default)]
struct Test2 {}

impl CachedComponentData for Test1 {
    fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }
    fn get_symbol_name() -> &'static str {
        use std::any::type_name;
        static SYMBOL_NAME: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME.get_or_init(|| type_name::<Self>().replace("::", "."))
    }
}
impl CachedComponentData for Test2 {
    fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }
    fn get_symbol_name() -> &'static str {
        use std::any::type_name;
        static SYMBOL_NAME: OnceLock<String> = OnceLock::new();
        SYMBOL_NAME.get_or_init(|| type_name::<Self>().replace("::", "."))
    }
}

fn main() {
    println!("Hello, world!");
    //debug_here!();
    let world = World::new();
    let world2 = World::new();

    //print id of Test1 and Test2
    println!("Test1 id: {}", Test1::get_id(world.world));
    println!("Test2 id: {}", Test2::get_id(world.world));

    println!("Test2 id: {}", Test2::get_id(world2.world));
}
