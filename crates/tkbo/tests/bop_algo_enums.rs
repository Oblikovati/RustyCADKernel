//! Rust checks for discriminants declared in upstream `BOPAlgo_*` enum headers.
use rustycad_tkbo::bop_algo::{Glue, Operation};

#[test]
fn glue_preserves_occt_discriminants_and_default() {
    assert_eq!(Glue::Off as i32, 0);
    assert_eq!(Glue::Shift as i32, 1);
    assert_eq!(Glue::Full as i32, 2);
    assert_eq!(Glue::default(), Glue::Off);
}

#[test]
fn boolean_operation_preserves_occt_discriminants_and_default() {
    assert_eq!(Operation::Common as i32, 0);
    assert_eq!(Operation::Fuse as i32, 1);
    assert_eq!(Operation::Cut as i32, 2);
    assert_eq!(Operation::Cut21 as i32, 3);
    assert_eq!(Operation::Section as i32, 4);
    assert_eq!(Operation::Unknown as i32, 5);
    assert_eq!(Operation::default(), Operation::Unknown);
}
