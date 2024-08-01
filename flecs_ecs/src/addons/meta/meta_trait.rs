pub trait Meta<Component> {
    fn meta(component: flecs_ecs::core::Component<Component>);
}

impl<'a, T: Meta<T>> crate::core::Component<'a, T> {
    pub fn meta(self) {
        T::meta(self);
    }
}
