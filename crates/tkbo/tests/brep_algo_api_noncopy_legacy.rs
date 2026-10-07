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

/// The Boolean APIs share the copy contract of their General Fuse base: no `Clone`/`Copy`,
/// so a C++ copy construction has no Rust spelling.
mod boolean_apis {
    use rustycad_tkbo::brep_algo_api::{
        BRepAlgoAPIBooleanOperation, BRepAlgoAPICommon, BRepAlgoAPICut, BRepAlgoAPIFuse,
        BRepAlgoAPISection, BRepAlgoAPISplitter,
    };
    use static_assertions::assert_not_impl_any;

    assert_not_impl_any!(BRepAlgoAPIBooleanOperation: Clone, Copy);
    assert_not_impl_any!(BRepAlgoAPICut: Clone, Copy);
    assert_not_impl_any!(BRepAlgoAPIFuse: Clone, Copy);
    assert_not_impl_any!(BRepAlgoAPICommon: Clone, Copy);
    assert_not_impl_any!(BRepAlgoAPISection: Clone, Copy);
    assert_not_impl_any!(BRepAlgoAPISplitter: Clone, Copy);

    #[test]
    fn boolean_operation_not_copy_constructible() {
        assert!(BRepAlgoAPIBooleanOperation::new().tools().is_empty());
    }

    #[test]
    fn cut_not_copy_constructible() {
        assert!(BRepAlgoAPICut::new().tools().is_empty());
    }

    #[test]
    fn fuse_not_copy_constructible() {
        assert!(BRepAlgoAPIFuse::new().tools().is_empty());
    }

    #[test]
    fn common_not_copy_constructible() {
        assert!(BRepAlgoAPICommon::new().tools().is_empty());
    }

    #[test]
    fn section_not_copy_constructible() {
        assert!(BRepAlgoAPISection::new().tools().is_empty());
    }

    #[test]
    fn splitter_not_copy_constructible() {
        assert!(BRepAlgoAPISplitter::new().tools().is_empty());
    }
}
