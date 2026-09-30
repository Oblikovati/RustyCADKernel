//! Bnd_BoundSortBox behavior beyond the legacy suite: raised errors, the errors that
//! replace native null dereferences, result order and the preserved open-box quirk. The
//! same paths are compared with OCCT by the `sortbox` probe.
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};

fn cube(low: f64, high: f64) -> BoundingBox3 {
    BoundingBox3::new(Point3::new(low, low, low), Point3::new(high, high, high))
}
fn boxes(values: Vec<BoundingBox3>) -> Array1<'static, BoundingBox3> {
    let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (index, value) in values.into_iter().enumerate() {
        array[index as i32 + 1] = value;
    }
    array
}
fn indices(list: &List<i32>) -> Vec<i32> {
    list.iter().copied().collect()
}

#[test]
fn count_initialization_and_add_raise_the_native_exceptions() {
    let mut sorter = BoundSortBox::default();
    let failure = sorter
        .initialize_with_count(&cube(0.0, 1.0), 0)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullValue");
    // Add before any initialization dereferences a null handle natively.
    let failure = sorter.add(&cube(0.0, 1.0), 1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    sorter.initialize_with_count(&cube(0.0, 10.0), 2).unwrap();
    sorter.add(&cube(0.0, 1.0), 1).unwrap();
    let failure = sorter.add(&cube(0.0, 1.0), 1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_MultiplyDefined");
    let failure = sorter.add(&cube(0.0, 1.0), 3).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    // A void box passes the checks and is ignored, leaving the slot free.
    sorter.add(&BoundingBox3::default(), 2).unwrap();
    sorter.add(&cube(2.0, 3.0), 2).unwrap();
    assert!(!sorter.boxes().unwrap()[2].is_void());
}

#[test]
fn uninitialized_comparisons_are_empty() {
    let mut sorter = BoundSortBox::default();
    assert!(sorter.boxes().is_none());
    assert!(sorter.compare_box(&cube(0.0, 1.0)).unwrap().is_empty());
    let plane = Plane3::new(Point3::new(0.0, 0.0, 0.0), Direction3::Z).unwrap();
    assert!(sorter.compare_plane(plane).unwrap().is_empty());
}

#[test]
fn large_boxes_come_first_then_grid_boxes_by_z_slice() {
    let mut sorter = BoundSortBox::default();
    let content = vec![cube(8.0, 9.0), cube(0.0, 10.0), cube(1.0, 2.0)];
    sorter.initialize(boxes(content)).unwrap();
    let result = sorter.compare_box(&cube(0.5, 9.5)).unwrap();
    assert_eq!(indices(result), vec![2, 3, 1]);
}

#[test]
fn open_boxes_under_a_closed_enclosing_box_are_never_found() {
    // The open side maps to INT_MIN (cvttsd2si), -1/+1 wraps and the clamped voxel range
    // is empty, so the box never enters a slice; native OCCT reports nothing either.
    let mut open = cube(5.0, 6.0);
    open.open_xmin();
    let mut sorter = BoundSortBox::default();
    sorter
        .initialize_with_enclosing(&cube(0.0, 10.0), boxes(vec![cube(0.0, 1.0), open]))
        .unwrap();
    assert!(sorter.compare_box(&cube(5.5, 6.0)).unwrap().is_empty());
    // Without an explicit enclosing box the open side opens the enclosing box instead.
    let mut open = cube(5.0, 6.0);
    open.open_xmin();
    let mut sorter = BoundSortBox::default();
    sorter
        .initialize(boxes(vec![cube(0.0, 1.0), open]))
        .unwrap();
    assert_eq!(
        indices(sorter.compare_box(&cube(5.5, 6.0)).unwrap()),
        vec![2]
    );
}

#[test]
fn void_enclosing_box_keeps_the_previous_grid() {
    let mut sorter = BoundSortBox::default();
    sorter.initialize(boxes(vec![cube(0.0, 1.0)])).unwrap();
    sorter
        .initialize_with_enclosing(&BoundingBox3::default(), boxes(vec![cube(0.0, 1.0)]))
        .unwrap();
    // A void enclosing box rejects every query before the grid is consulted.
    assert!(sorter.compare_box(&cube(0.0, 1.0)).unwrap().is_empty());
    // Adding through a count initialization with a void enclosing box reads its corner.
    sorter
        .initialize_with_count(&BoundingBox3::default(), 1)
        .unwrap();
    let failure = sorter.add(&cube(0.0, 1.0), 1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}
