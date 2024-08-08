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

    fn name(&self) -> &str {
        self
    }

    fn count(&self) -> i32 {
        0
    }

    fn offset(&self) -> i32 {
        0
    }
}

impl MetaMember for (&'static str,) {
    const USE_OFFSET: bool = false;
    fn name(&self) -> &str {
        self.0
    }

    fn count(&self) -> i32 {
        0
    }

    fn offset(&self) -> i32 {
        0
    }
}

impl MetaMember for (&'static str, i32) {
    const USE_OFFSET: bool = false;
    fn name(&self) -> &str {
        self.0
    }

    fn count(&self) -> i32 {
        self.1
    }

    fn offset(&self) -> i32 {
        0
    }
}

impl MetaMember for (&'static str, i32, usize) {
    const USE_OFFSET: bool = true;
    fn name(&self) -> &str {
        self.0
    }

    fn count(&self) -> i32 {
        self.1
    }

    fn offset(&self) -> i32 {
        self.2 as i32
    }
}
