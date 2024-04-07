use flecs_ecs_derive::Component;

/// Component data that is cached by the `ComponentId` trait.
/// This data is used to register components with the world.
/// It is also used to ensure that components are registered consistently across different worlds.
#[derive(Clone, Debug, Default)]
pub struct IdComponent {
    pub id: u64,
}

pub struct Enum;
pub struct Struct;

#[derive(Component)]
pub enum NoneEnum {
    None = 1,
}

#[derive(Default, Clone)]
pub struct FlecsNoneDefaultDummy;

#[derive(Clone)]
pub struct FlecsNoneCloneDummy;

pub struct ConditionalTypeSelector<const B: bool, T> {
    phantom: std::marker::PhantomData<T>,
}
