#![doc(hidden)]

pub trait Builder<'a> {
    type BuiltType;

    fn build(&mut self) -> Self::BuiltType;
}
