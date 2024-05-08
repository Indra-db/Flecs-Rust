#![allow(dead_code)]
use flecs_ecs::core::ComponentId;
use flecs_ecs_derive::Component;

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

fn is_ref<T: ComponentId>(is_ref: bool) {
    assert_eq!(is_ref, T::IS_REF);
}

fn is_mut<T: ComponentId>(is_mut: bool) {
    assert_eq!(is_mut, T::IS_MUT);
}

#[test]
fn test_ref_mut_ref() {
    is_ref::<Position>(false);
    is_mut::<Position>(false);

    is_ref::<&Position>(true);
    is_mut::<&Position>(false);

    is_ref::<&mut Position>(false);
    is_mut::<&mut Position>(true);
}
