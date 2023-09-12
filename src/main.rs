use std::sync::OnceLock;

use flecs_ecs_bridge::core::component::{test, CachedComponentData, ComponentData};

#[derive(Clone, Default)]
struct Test1 {}

#[derive(Clone, Default)]
struct Test2 {}

impl CachedComponentData for Test1 {
    fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }
}
impl CachedComponentData for Test2 {
    fn get_once_lock_data() -> &'static OnceLock<ComponentData> {
        static ONCE_LOCK: OnceLock<ComponentData> = OnceLock::new();
        &ONCE_LOCK
    }
}
fn main() {
    println!("Hello, world!");

    //print id of Test1 and Test2
    println!("Test1 id: {}", Test1::get_id_checked());
    println!("Test2 id: {}", Test2::get_id_checked());
}
