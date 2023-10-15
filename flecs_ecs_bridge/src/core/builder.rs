use super::{filter::Filter, iterable::Iterable};

pub trait Builder {
    type BuiltType;
}

impl<'a, T> Builder for Filter<'a, T>
where
    T: Iterable<'a>,
{
    type BuiltType = Filter<'a, T>;
}
