use super::Count;

/// Meant to be used with `.member` method of (untyped) components.
/// This is to allow an unified function API to allow arbitrary number of arguments.
/// valid options are (name : &'static str,), (name: &'static str, count: i32), (name: &'static str, count: i32, offset: i32)
pub trait MetaMember: 'static {
    /// Whether to use explicit offset or not
    /// when false, flecs calculates the offset for us, this is useful for simple structs.
    /// but might fail for more complex structs.
    const USE_OFFSET: bool;

    fn name(&self) -> &str;
    fn count(&self) -> i32;
    fn offset(&self) -> i32;
}

impl MetaMember for &'static str {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &str {
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

impl MetaMember for (&'static str,) {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &str {
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

impl MetaMember for (&'static str, Count) {
    const USE_OFFSET: bool = false;

    #[inline(always)]
    fn name(&self) -> &str {
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

impl MetaMember for (&'static str, Count, usize) {
    const USE_OFFSET: bool = true;

    #[inline(always)]
    fn name(&self) -> &str {
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
