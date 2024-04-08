use super::term::TermBuilder;

pub trait Builder<'a>: TermBuilder<'a> {
    type BuiltType;

    fn build(&'a mut self) -> Self::BuiltType;
}
