use super::Count;

/// Meant to be used with `.member` method of (untyped) components.
/// This is to allow an unified function API to allow arbitrary number of arguments.
/// valid options are (name: &'a str,), (name: &'a str, count: i32), (name: &'a str, count: i32, offset: i32)
pub trait MetaMember<'a> {
    /// Whether to use explicit offset or not
    /// when false, flecs calculates the offset for us, this is useful for simple structs.
    /// but might fail for more complex structs.
    const USE_OFFSET: bool;

    fn name(&self) -> &'a str;
    fn count(&self) -> i32;
    fn offset(&self) -> i32;
}

impl<'a> MetaMember<'a> for &'a str {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &'a str {
        self
    }

    #[inline(always)]
    fn count(&self) -> i32 {
        0
    }

    #[inline(always)]
    fn offset(&self) -> i32 {
        0
    }
}

impl<'a> MetaMember<'a> for (&'a str,) {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &'a str {
        self.0
    }

    #[inline(always)]
    fn count(&self) -> i32 {
        0
    }

    #[inline(always)]
    fn offset(&self) -> i32 {
        0
    }
}

impl<'a> MetaMember<'a> for (&'a str, Count) {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &'a str {
        self.0
    }

    #[inline(always)]
    fn count(&self) -> i32 {
        self.1.0
    }

    #[inline(always)]
    fn offset(&self) -> i32 {
        0
    }
}

impl<'a> MetaMember<'a> for (&'a str, Count, usize) {
    const USE_OFFSET: bool = true;

    #[inline(always)]
    fn name(&self) -> &'a str {
        self.0
    }

    #[inline(always)]
    fn count(&self) -> i32 {
        self.1.0
    }

    #[inline(always)]
    fn offset(&self) -> i32 {
        self.2 as i32
    }
}
