use super::term::TermBuilder;

pub trait Builder<'a>: TermBuilder<'a> {
    type BuiltType;

    fn build(&mut self) -> Self::BuiltType;
}
