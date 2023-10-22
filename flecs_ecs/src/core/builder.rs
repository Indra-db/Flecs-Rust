use super::term::TermBuilder;

pub trait Builder: TermBuilder {
    type BuiltType;

    fn build(&mut self) -> Self::BuiltType;
}
