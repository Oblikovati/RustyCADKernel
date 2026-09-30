//! Regression coverage for the native duplicate-boundary and insertion semantics.
use rustycad_tkbo::int_tools::{MarkedRangeSet, Range};
#[test]
fn constructors_and_reinitialization_preserve_boundaries() {
    let mut ranges = MarkedRangeSet::default();
    assert_eq!(ranges.length(), 0);
    assert_eq!(
        ranges.get_index(0.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    ranges.set_boundaries(2.0, -1.0, 3);
    assert_eq!(ranges.range(1).unwrap().range(), (2.0, -1.0));
    assert_eq!(ranges.flag(1).unwrap(), 3);
    ranges.set_flag(1, 4).unwrap();
    assert_eq!(ranges.flag(1).unwrap(), 4);
    ranges.set_ranges(&[0.0, 0.5, 1.0], 7);
    assert_eq!(ranges.length(), 2);
    assert_eq!(ranges.range(2).unwrap().range(), (0.5, 1.0));
    assert!(ranges.flag(0).is_err());
    assert!(ranges.set_flag(3, 1).is_err());
    assert!(ranges.range(3).is_err());
}
#[test]
fn duplicate_bounds_report_all_matching_indices() {
    let mut ranges = MarkedRangeSet::from_ranges(&[0.0, 1.0, 1.0, 2.0], 0);
    assert_eq!(ranges.get_index(1.0).unwrap(), 1);
    assert_eq!(ranges.get_index_with_side(1.0, true).unwrap(), 3);
    assert_eq!(ranges.get_index_with_side(1.0, false).unwrap(), 1);
    assert_eq!(ranges.get_indices(1.0).unwrap(), &[1, 2, 3]);
    assert!(ranges.get_indices(-1.0).unwrap().is_empty());
    assert!(ranges.get_indices(3.0).unwrap().is_empty());
    assert_eq!(ranges.get_index_with_side(0.0, false).unwrap(), 0);
}
#[test]
fn generic_insertion_retains_existing_and_duplicate_bounds() {
    let mut ranges = MarkedRangeSet::from_ranges(&[0.0, 1.0, 2.0], 0);
    assert!(ranges.insert_range(0.5, 1.5, 7).unwrap());
    assert_eq!(ranges.length(), 4);
    assert_eq!((ranges.flag(2).unwrap(), ranges.flag(3).unwrap()), (7, 7));
    assert!(ranges.insert(&Range::new(1.0, 1.0), 9).unwrap());
    assert_eq!(ranges.length(), 6);
    assert!(!ranges.insert_range(2.0, 0.0, 3).unwrap());
    assert!(!ranges.insert_range(-1.0, 1.0, 3).unwrap());
    assert!(!ranges.insert_range(0.0, 3.0, 3).unwrap());
}
#[test]
fn indexed_insertion_uses_native_tolerance_and_fallback() {
    let mut ranges = MarkedRangeSet::new(0.0, 1.0, 0);
    assert!(!ranges.insert_range_at(0.1, 0.9, 1, 0).unwrap());
    assert!(ranges.insert_range_at(0.1, 0.9, 1, 1).unwrap());
    assert_eq!(ranges.length(), 3);
    assert_eq!(ranges.range(2).unwrap().range(), (0.1, 0.9));
    assert!(ranges.insert_at(&Range::new(0.1, 0.9), 2, 2).unwrap());
    assert_eq!(ranges.length(), 3);
    assert_eq!(ranges.flag(2).unwrap(), 2);
    assert!(ranges.insert_range_at(0.0, 1.0, 3, 2).unwrap());
    assert_eq!(ranges.length(), 5);
}
