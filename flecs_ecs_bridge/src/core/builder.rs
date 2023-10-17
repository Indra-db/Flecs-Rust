use super::{filter::Filter, iterable::Iterable, term::TermBuilder};

pub trait Builder: TermBuilder {
    type BuiltType;

    fn build(self) -> Self::BuiltType;
}
