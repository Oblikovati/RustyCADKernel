//! Storage and shape contracts beyond the eleven original declarations.
use rustycad_tkmath::math::DoubleTab;

#[test]
fn inline_threshold_and_copy_ownership() {
    for (rows, cols, owned) in [
        (8, 8, false),
        (1, 64, false),
        (1, 65, true),
        (9, 8, true),
        (0, 2, false),
    ] {
        let table = DoubleTab::with_size(rows, cols);
        assert_eq!(table.is_deletable(), owned);
        assert_eq!((table.row_size(), table.col_size()), (rows, cols));
        assert_eq!(table.clone().is_deletable(), rows * cols != 0);
    }
}
#[test]
fn borrowed_cells_write_through_and_assignment_detaches_only_on_resize() {
    let mut cells = [0.0; 4];
    {
        let mut table = DoubleTab::from_buffer(&mut cells, 1, 2, 1, 2).unwrap();
        let mut source = DoubleTab::with_bounds(-1, 0, 3, 4).unwrap();
        source.init(4.0);
        table.assign(&source);
        assert!(!table.is_deletable());
        assert_eq!((table.lower_row(), table.lower_col()), (-1, 3));
        table.assign(&DoubleTab::with_size(1, 1));
        assert!(table.is_deletable());
    }
    assert_eq!(cells, [4.0; 4]);
}
#[test]
fn rebasing_preserves_values_and_zero_based_access() {
    let mut table = DoubleTab::with_size(2, 3);
    *table.change_at(1, 2).unwrap() = 9.0;
    table.set_lower_row(-7);
    table.set_lower_col(10);
    assert_eq!((table.upper_row(), table.upper_col()), (-6, 12));
    assert_eq!((table.nb_rows(), table.nb_columns()), (2, 3));
    assert_eq!(*table.value(-6, 12).unwrap(), 9.0);
    assert_eq!(*table.at(1, 2).unwrap(), 9.0);
    assert!(table.at(2, 0).is_err());
    assert!(table.value(-8, 10).is_err());
}
#[test]
fn construction_and_copy_reject_invalid_shapes() {
    assert!(DoubleTab::with_bounds(1, 0, 1, 1).is_err());
    assert!(DoubleTab::from_buffer(&mut [0.0], 1, 2, 1, 2).is_err());
    let source = DoubleTab::with_size(2, 3);
    assert!(source.copy_to(&mut DoubleTab::with_size(3, 2)).is_err());
    let mut destination = DoubleTab::with_bounds(-1, 0, 4, 6).unwrap();
    source.copy_to(&mut destination).unwrap();
    assert_eq!((destination.lower_row(), destination.lower_col()), (-1, 4));
}
#[test]
fn moving_inline_copies_while_moving_owned_empties_source() {
    for length in [4, 64, 65] {
        let mut source = DoubleTab::with_size(1, length);
        source.init(3.0);
        let moved = DoubleTab::move_construct(&mut source);
        assert_eq!(*moved.at(0, length - 1).unwrap(), 3.0);
        assert_eq!(source.row_size(), usize::from(length <= 64));
        assert_eq!(moved.is_deletable(), length > 64);
    }
}
#[test]
fn move_assignment_requires_owned_matching_bounds() {
    let mut source = DoubleTab::with_size(1, 65);
    source.init(7.0);
    let mut destination = DoubleTab::with_bounds(1, 1, 1, 65).unwrap();
    destination.move_assign(&mut source);
    assert_eq!(source.row_size(), 1);
    assert_eq!(destination.lower_row(), 0);
    destination.move_assign(&mut source);
    assert_eq!((source.row_size(), source.lower_row()), (0, 1));
    assert_eq!(*destination.at(0, 64).unwrap(), 7.0);
}
