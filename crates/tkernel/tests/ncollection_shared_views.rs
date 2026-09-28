//! Read-only buffer views behind BSplCLib/BSplSLib::UnitWeights.
use rustycad_tkernel::ncollection::{Array1, Array2};

#[test]
fn shared_views_do_not_own_and_detach_on_write() {
    let buffer: [f64; 6] = [1.0; 6];
    let mut view: Array1<'_, f64> = Array1::from_shared(&buffer[..3], 1);
    assert_eq!((view.lower(), view.upper(), view.len()), (1, 3, 3));
    assert!(!view.is_deletable());
    assert_eq!(view[3], 1.0);
    view[2] = 5.0;
    assert!(view.is_deletable());
    assert_eq!((view[2], buffer[1]), (5.0, 1.0));
    let grid: Array2<'_, f64> = Array2::from_shared(&buffer, 1, 2, 1, 3).unwrap();
    assert!(!grid.is_deletable());
    assert_eq!(
        (grid.upper_row(), grid.upper_col(), grid[(2, 3)]),
        (2, 3, 1.0)
    );
    assert!(Array2::from_shared(&buffer, 1, 3, 1, 3).is_err());
}
