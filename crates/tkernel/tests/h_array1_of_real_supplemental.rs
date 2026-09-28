//! `NCollection_HArray1<double>`: bounds, checked access, runtime type and sharing.
use rustycad_tkernel::{
    ncollection::{Array1, ArrayError, HArray1OfReal},
    standard::{Handle, RuntimeTyped},
};
use std::thread;

#[test]
fn with_bounds_allocates_zeros_and_rejects_inverted_bounds() {
    let array = HArray1OfReal::with_bounds(2, 4).unwrap();
    assert_eq!((array.lower(), array.upper(), array.len()), (2, 4, 3));
    assert_eq!(array.value(3).unwrap(), 0.0);
    assert!(HArray1OfReal::with_bounds(1, 0).unwrap().is_empty());
    let error = HArray1OfReal::with_bounds(3, 0).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_RangeError");
    assert!(error.message().contains("NCollection_HArray1"));
}

#[test]
fn checked_access_raises_out_of_range_with_the_index() {
    let array = HArray1OfReal::with_bounds(1, 2).unwrap();
    array.set_value(2, 1.5).unwrap();
    assert_eq!(array.value(2).unwrap(), 1.5);
    let read = array.value(3).unwrap_err();
    assert_eq!(read.exception_type(), "Standard_OutOfRange");
    assert!(
        read.message()
            .contains("NCollection_HArray1::Value: index 3")
    );
    let write = array.set_value(0, 1.0).unwrap_err();
    assert!(write.message().contains("NCollection_HArray1::SetValue"));
}

#[test]
fn array1_copies_and_change_array1_edits_in_place() {
    let array = HArray1OfReal::from_array(Array1::with_bounds(0, 1).unwrap());
    let before = array.array1();
    array.change_array1(|values| values.init(2.0));
    assert_eq!(*before.value(0).unwrap(), 0.0);
    assert_eq!(array.value(1).unwrap(), 2.0);
}

#[test]
fn runtime_type_is_the_native_template_name() {
    let array = HArray1OfReal::shared(Array1::with_bounds(1, 1).unwrap());
    assert_eq!(
        array.dynamic_type().map(|kind| kind.name()),
        Some("NCollection_HArray1<double>")
    );
    assert_eq!(
        HArray1OfReal::static_type().name(),
        "NCollection_HArray1<double>"
    );
}

#[test]
fn edits_through_a_handle_reach_holders_on_other_threads() {
    let shared: Handle<HArray1OfReal> = HArray1OfReal::shared(Array1::with_bounds(1, 3).unwrap());
    let writer = shared.clone();
    thread::spawn(move || writer.set_value(2, 7.0).unwrap())
        .join()
        .unwrap();
    assert_eq!(shared.value(2).unwrap(), 7.0);
}

#[test]
fn array_errors_map_to_native_exception_types() {
    let cases = [
        (
            ArrayError::Bounds { lower: 2, upper: 0 },
            "Standard_RangeError",
        ),
        (
            ArrayError::Index {
                index: 5,
                lower: 1,
                upper: 3,
            },
            "Standard_OutOfRange",
        ),
        (
            ArrayError::DimensionMismatch {
                expected: 2,
                actual: 3,
            },
            "Standard_DimensionMismatch",
        ),
        (
            ArrayError::ShapeMismatch {
                expected: (1, 2),
                actual: (2, 1),
            },
            "Standard_DimensionMismatch",
        ),
    ];
    for (error, kind) in cases {
        let failure = error.into_failure("Member");
        assert_eq!(failure.exception_type(), kind);
        assert_eq!(failure.message(), format!("Member: {error}"));
    }
}
