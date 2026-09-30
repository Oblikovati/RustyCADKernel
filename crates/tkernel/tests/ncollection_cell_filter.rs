//! Complete legacy CellFilter declarations with the original inspector actions.
// Preserve the upstream construct-then-configure inspector sequence.
#![allow(non_snake_case, clippy::field_reassign_with_default)]
use rustycad_tkernel::ncollection::{CellAction, CellFilter, CellInspector};
struct TestInspector<const N: usize> {
    found: Vec<i32>,
    purge: i32,
}
impl<const N: usize> Default for TestInspector<N> {
    fn default() -> Self {
        Self {
            found: Vec::new(),
            purge: -1,
        }
    }
}
impl<const N: usize> CellInspector for TestInspector<N> {
    type Target = i32;
    type Point = [f64; N];
    const DIMENSION: usize = N;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(left: &i32, right: &i32) -> bool {
        left == right
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        if N == 2 && *target == self.purge {
            return CellAction::Purge;
        }
        self.found.push(*target);
        CellAction::Keep
    }
}

#[test]
fn NCollection_CellFilterTest__AddInspect2D_SinglePoint() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    // Add targets at different points
    aFilter.add(&10, &[0.5, 0.5]);
    aFilter.add(&20, &[1.5, 1.5]);
    aFilter.add(&30, &[0.3, 0.3]);

    // Inspect cell containing (0.5, 0.5) -- should find 10 and 30 (same cell)
    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);

    // Both 10 and 30 are in cell (0,0)
    assert_eq!(2, anInspector.found.len());
}

#[test]
fn NCollection_CellFilterTest__AddInspect2D_Range() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&1, &[0.5, 0.5]);
    aFilter.add(&2, &[1.5, 0.5]);
    aFilter.add(&3, &[2.5, 0.5]);

    // Inspect range covering cells (0,0) and (1,0)
    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect_range(&[0.0, 0.0], &[1.5, 0.5], &mut anInspector);

    // Should find targets 1 and 2
    assert_eq!(2, anInspector.found.len());
}

#[test]
fn NCollection_CellFilterTest__ReversedRangesAreIgnored() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);
    aFilter.add(&1, &[1.5, 1.5]);

    let aMin = &[1.5, 1.5];
    let aMax = &[0.5, 0.5];

    aFilter.add_range(&2, aMin, aMax);

    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect_range(aMin, aMax, &mut anInspector);
    assert!(anInspector.found.is_empty());

    aFilter.remove_range(&1, aMin, aMax);
    let mut aPointInspector = TestInspector::<2>::default();
    aFilter.inspect(&[1.5, 1.5], &mut aPointInspector);
    assert_eq!(1, aPointInspector.found.len());
    assert_eq!(1, aPointInspector.found[0]);
}

#[test]
fn NCollection_CellFilterTest__Remove2D() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&10, &[0.5, 0.5]);
    aFilter.add(&20, &[0.5, 0.5]);

    // Remove target 10
    aFilter.remove(&10, &[0.5, 0.5]);

    // Inspect -- should only find 20
    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);

    assert_eq!(1, anInspector.found.len());
    assert_eq!(20, anInspector.found[0]);
}

#[test]
fn NCollection_CellFilterTest__EmptyCellCleanupAfterRemove() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&10, &[0.5, 0.5]);

    // Remove the only target -- cell should be cleaned up
    aFilter.remove(&10, &[0.5, 0.5]);

    // Inspect -- should find nothing
    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);
    assert!(anInspector.found.is_empty());
}

#[test]
fn NCollection_CellFilterTest__EmptyCellCleanupAfterPurge() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&10, &[0.5, 0.5]);

    // Inspect with purge action for target 10
    let mut anInspector = TestInspector::<2>::default();
    anInspector.purge = 10;
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);

    // Found should be empty (purged, not kept)
    assert!(anInspector.found.is_empty());

    // Second inspect -- cell should have been cleaned up, nothing found
    let mut anInspector2 = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector2);
    assert!(anInspector2.found.is_empty());
}

#[test]
fn NCollection_CellFilterTest__AddInspect3D() {
    let mut aFilter = CellFilter::<TestInspector<3>>::new(1.0);

    aFilter.add(&1, &[0.5, 0.5, 0.5]);
    aFilter.add(&2, &[1.5, 1.5, 1.5]);
    aFilter.add(&3, &[0.3, 0.3, 0.3]);

    // Inspect cell containing (0.5, 0.5, 0.5)
    let mut anInspector = TestInspector::<3>::default();
    aFilter.inspect(&[0.5, 0.5, 0.5], &mut anInspector);

    // Both 1 and 3 are in cell (0,0,0)
    assert_eq!(2, anInspector.found.len());
}

#[test]
fn NCollection_CellFilterTest__Reset() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&10, &[0.5, 0.5]);

    // Reset with new cell size
    aFilter.reset(2.0, None).unwrap();

    // After reset, all data should be cleared
    let mut anInspector = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);
    assert!(anInspector.found.is_empty());
}

#[test]
fn NCollection_CellFilterTest__InspectWithPurge() {
    let mut aFilter = CellFilter::<TestInspector<2>>::new(1.0);

    aFilter.add(&10, &[0.5, 0.5]);
    aFilter.add(&20, &[0.5, 0.5]);
    aFilter.add(&30, &[0.5, 0.5]);

    // Inspect with purge for target 20
    let mut anInspector = TestInspector::<2>::default();
    anInspector.purge = 20;
    aFilter.inspect(&[0.5, 0.5], &mut anInspector);

    // Should find 10 and 30 (20 was purged)
    assert_eq!(2, anInspector.found.len());

    // Re-inspect -- 20 should be gone
    let mut anInspector2 = TestInspector::<2>::default();
    aFilter.inspect(&[0.5, 0.5], &mut anInspector2);
    assert_eq!(2, anInspector2.found.len());
}
