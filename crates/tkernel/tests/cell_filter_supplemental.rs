//! Cell ordering and index arithmetic beyond the upstream nine assertions.
use rustycad_tkernel::ncollection::{BaseAllocator, CellAction, CellFilter, CellInspector, List};
#[derive(Default)]
struct RecordingInspector {
    visited: Vec<i32>,
}
impl CellInspector for RecordingInspector {
    type Target = i32;
    type Point = [f64; 2];
    const DIMENSION: usize = 2;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.visited.push(*target);
        CellAction::Keep
    }
}
#[test]
fn native_cell_index_and_traversal_order() {
    let mut filter: CellFilter<RecordingInspector> = CellFilter::new(1.0);
    filter.add(&10, &[-0.9, -0.9]);
    filter.add(&20, &[0.9, 0.9]);
    filter.add(&30, &[1.1, 0.1]);
    filter.add(&40, &[0.1, 1.1]);
    filter.add(&50, &[1.1, 1.1]);
    let mut inspector: RecordingInspector = RecordingInspector::default();
    filter.inspect_range(&[0.0, 0.0], &[1.9, 1.9], &mut inspector);
    assert_eq!(inspector.visited, [20, 10, 30, 40, 50]);
    filter.add(&60, &[f64::from(i32::MAX) + 1.0, 0.0]);
    filter.add(&70, &[f64::from(i32::MIN) - 1.0, 0.0]);
    inspector.visited.clear();
    filter.inspect(&[1.0, 0.0], &mut inspector);
    filter.inspect(&[-1.0, 0.0], &mut inspector);
    assert_eq!(inspector.visited, [60, 30, 70]);
}
#[test]
fn range_duplicates_anisotropic_sizes_and_removal() {
    let arena: BaseAllocator = BaseAllocator::incremental(1024);
    let mut filter: CellFilter<RecordingInspector> = CellFilter::with_sizes(vec![2.0, 4.0], arena);
    filter.add_range(&1, &[0.0, 0.0], &[3.0, 7.0]);
    filter.add_range(&1, &[0.0, 0.0], &[3.0, 7.0]);
    let mut inspector: RecordingInspector = RecordingInspector::default();
    filter.inspect_range(&[0.0, 0.0], &[3.0, 7.0], &mut inspector);
    assert_eq!(inspector.visited, [1; 8]);
    filter.remove_range(&1, &[0.0, 0.0], &[3.0, 7.0]);
    inspector.visited.clear();
    filter.inspect_range(&[0.0, 0.0], &[3.0, 7.0], &mut inspector);
    assert!(inspector.visited.is_empty());
}
#[test]
fn list_retention_preserves_survivor_storage() {
    let mut list: List<i32> = List::from_iter_in([1, 2, 3, 4], BaseAllocator::common());
    let address: *const i32 = list.iter().nth(1).unwrap();
    let mut inspected: Vec<i32> = Vec::new();
    list.retain(|value| {
        inspected.push(*value);
        value % 2 == 0
    });
    assert_eq!(inspected, [1, 2, 3, 4]);
    assert_eq!(list.iter().copied().collect::<Vec<i32>>(), [2, 4]);
    assert_eq!(list.first().unwrap() as *const i32, address);
    list.retain(|_| false);
    assert!(list.is_empty());
    list.retain(|_| panic!("empty lists have no callback"));
}
