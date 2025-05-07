use crate::core::ComponentId;

impl<T: ComponentId> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id::new()
    }
}

impl<T: ComponentId> Copy for Id<T> {}

#[derive(Debug)]
pub struct Id<T: ComponentId> {
    _marker: std::marker::PhantomData<T>,
}

pub fn id<T: ComponentId>() -> Id<T> {
    Id::new()
}

impl<T: ComponentId> Id<T> {
    pub(crate) fn new() -> Self {
        Id {
            _marker: std::marker::PhantomData,
        }
    }
}
