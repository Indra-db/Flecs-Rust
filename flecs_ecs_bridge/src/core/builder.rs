use super::{filter::Filter, iterable::Iterable, term::TermBuilder};

pub trait Builder: TermBuilder {
    type BuiltType;

    fn build(&mut self) -> Self::BuiltType;
}
