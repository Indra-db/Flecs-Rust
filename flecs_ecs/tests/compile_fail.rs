//! Compile-fail tests for thread-safety guarantees.
//!
//! Each file in `tests/compile_fail/` must fail to compile; the expected
//! compiler output lives in the matching `.stderr` file.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
