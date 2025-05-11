use crate::core::ComponentId;

impl<T: ComponentId> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ComponentId> Copy for Id<T> {}

#[derive(Debug)]
pub struct Id<T: ComponentId> {
    _marker: core::marker::PhantomData<T>,
}

#[inline(always)]
pub const fn id<T: ComponentId>() -> Id<T> {
    Id::new()
}

impl<T: ComponentId> Id<T> {
    #[inline(always)]
    pub(crate) const fn new() -> Self {
        Id {
            _marker: core::marker::PhantomData,
        }
    }
}
