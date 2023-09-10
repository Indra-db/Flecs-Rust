// Considerations on Type Trait Checks in Rust:
//
// 1. Unlike C++, Rust doesn't support compile-time checks for trivial types.
// 2. Current implementation prioritizes simplicity over performance optimizations.
//    - If trivial type registration incurs a significant performance penalty, reconsider this approach.
//
// Challenges:
// - Rust lacks several features for this scenario:
//   a) Trait specialization.
//   b) Compile-time trivial type checks.
//   c) A direct equivalent of `placement_new` from C++.
//      ptr::write still constructs the object on the stack and then moves it, barring optimizations.
//
// Potential Solutions:
// - Bypass the need for `placement_new` with a `placement_ctor` function.
//   - Drawback: Each field needs manual setting, which impacts user experience.
//      - example code:
//      ```
//           struct MyType {
//               vec: Vec<i32>,
//           }
//
//           trait PlacementNew {
//               unsafe fn placement_new(ptr: *mut Self);
//           }
//
//           impl PlacementNew for MyType {
//               unsafe fn placement_new(ptr: *mut Self) {
//                   (*ptr).vec = Vec::<i32>::default();
//               }
//           }
//      ```
// - For potential type optimizations, consider:
//   a) Utilizing the `Zeroable` trait and rely on user's proper implementation.
//   b) Implement pseudo-trait specialization, as detailed in:
//      - http://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html
//      - https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1e548abff8e35b97b25adcacdddaacda
//
// possible helpful crates for trait specialization / type specialization:
// - For type casting: https://crates.io/crates/castaway
//
// Note: C does the same, where the user needs to opt in for non trivial types. We can do the same.
