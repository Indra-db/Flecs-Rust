pub trait MetaMember: 'static {
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

impl MetaMember for (&'static str, i32, i32) {
    const USE_OFFSET: bool = true;
    fn name(&self) -> &str {
        self.0
    }

    fn count(&self) -> i32 {
        self.1
    }

    fn offset(&self) -> i32 {
        self.2
    }
}
