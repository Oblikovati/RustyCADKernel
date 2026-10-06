use super::{CellAction, CellFilter, CellInspector};
#[derive(Default)]
pub struct Inspector {
    pub seen: Vec<i32>,
    pub fail: usize,
    pub purge: bool,
}
impl CellInspector for Inspector {
    type Target = i32;
    type Point = [f64; 3];
    const DIMENSION: usize = 3;
    fn coord(axis: usize, point: &Self::Point) -> f64 {
        point[axis]
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.seen.push(*target);
        assert!(
            self.fail == 0 || self.seen.len() != self.fail,
            "native inspection failure"
        );
        if self.purge {
            CellAction::Purge
        } else {
            CellAction::Keep
        }
    }
}
pub fn assert_row(row: usize, label: &str, cursor: &[i32], inspector: &Inspector) {
    let mut actual = format!("{label} cursor");
    for coordinate in cursor {
        actual.push_str(&format!(" {coordinate}"));
    }
    actual.push_str(" seen");
    for value in &inspector.seen {
        actual.push_str(&format!(" {value}"));
    }
    let expected =
        include_str!("../../../../tools/oracle/tkernel_cell_filter_protected_helpers.expected");
    assert_eq!(
        actual,
        expected.lines().nth(row).unwrap(),
        "native row {row}"
    );
}
pub fn collect_range(
    row: usize,
    label: &str,
    axis: i32,
    cursor: &mut [i32],
    min: &[i32],
    max: &[i32],
) {
    let mut probe = RangeProbe::new(axis, min, max);
    let initial = cursor.to_vec();
    probe.compare_operation(row, label, cursor, SelfOperation::Add);
    cursor.copy_from_slice(&initial);
    probe.compare_operation(row + 1, "range-empty", cursor, SelfOperation::Remove);
}
#[derive(Clone, Copy)]
enum SelfOperation {
    Add,
    Remove,
}
struct RangeProbe<'a> {
    filter: CellFilter<Inspector>,
    axis: i32,
    min: &'a [i32],
    max: &'a [i32],
}
impl<'a> RangeProbe<'a> {
    fn new(axis: i32, min: &'a [i32], max: &'a [i32]) -> Self {
        Self {
            filter: CellFilter::new(1.0),
            axis,
            min,
            max,
        }
    }
    fn compare_operation(
        &mut self,
        row: usize,
        label: &str,
        cursor: &mut [i32],
        op: SelfOperation,
    ) {
        let initial = cursor.to_vec();
        match op {
            SelfOperation::Add => self.add(cursor),
            SelfOperation::Remove => self.remove(cursor),
        }
        let completed = cursor.to_vec();
        cursor.copy_from_slice(&initial);
        let inspected = self.inspect(cursor);
        assert_row(row, label, cursor, &inspected);
        assert_eq!(completed, cursor);
    }
    fn add(&mut self, cursor: &mut [i32]) {
        self.filter
            .iterate_add_indices(self.axis, cursor, self.min, self.max, &5)
            .unwrap();
    }
    fn remove(&mut self, cursor: &mut [i32]) {
        self.filter
            .iterate_remove_indices(self.axis, cursor, self.min, self.max, &5)
            .unwrap();
    }
    fn inspect(&mut self, cursor: &mut [i32]) -> Inspector {
        let mut inspector = Inspector::default();
        self.filter
            .iterate_inspect_indices(self.axis, cursor, self.min, self.max, &mut inspector)
            .unwrap();
        inspector
    }
}
pub fn inspect_failure(
    filter: &mut CellFilter<Inspector>,
    cursor: &mut [i32],
    min: &[i32],
    max: &[i32],
) -> Inspector {
    let mut inspector = Inspector {
        fail: 3,
        purge: true,
        ..Inspector::default()
    };
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        filter
            .iterate_inspect_indices(1, cursor, min, max, &mut inspector)
            .unwrap();
    }));
    assert!(failed.is_err());
    inspector
}
mod failures;
