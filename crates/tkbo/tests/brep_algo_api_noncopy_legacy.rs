//! Rust copy-contract port of BRepAlgoAPI_BuilderAlgo's native type-trait test.
//! C++ copy construction maps to the Rust `Clone`/`Copy` contract here; C++ move
//! traits have no equivalent because Rust values are movable by default.
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBuilderAlgo;
use static_assertions::assert_not_impl_any;

assert_not_impl_any!(BRepAlgoAPIBuilderAlgo: Clone, Copy);

#[test]
fn not_copy_constructible() {
    let builder = BRepAlgoAPIBuilderAlgo::new();
    assert!(builder.arguments().is_empty());
}

/// C++ copy assignment maps to `Clone::clone_from`, which the `Clone` assertion above rules
/// out; only an owned value can replace a builder.
#[test]
fn not_copy_assignable() {
    let mut builder = BRepAlgoAPIBuilderAlgo::new();
    assert!(builder.arguments().is_empty());
    builder = BRepAlgoAPIBuilderAlgo::new();
    assert!(builder.arguments().is_empty());
}
