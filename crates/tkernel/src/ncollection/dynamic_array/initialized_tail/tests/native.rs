use super::panics::PanicDropItem;
use super::*;
const EXPECTED: &str =
    include_str!("../../../../../../../tools/oracle/tkernel_dynamic_tail_fastpath.expected");
fn growth_row(name: &str, target: &DynamicArray<i32>) -> String {
    let values = target
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{name} {} {} {}\n",
        target.size(),
        target.blocks.len(),
        if values.is_empty() { "-" } else { &values }
    )
}
fn growth_rows() -> String {
    let mut target = DynamicArray::<i32>::with_increment(2);
    reserve_blocks(&mut target, 4);
    let mut output = String::new();
    for index in 0..6 {
        target.set_value(index, index + 10).unwrap();
        output += &growth_row(&format!("grow{index}"), &target);
        assert_eq!(assert_tail_state(&target), 0);
    }
    target.clear(false);
    output += &growth_row("grow_clear", &target);
    target.set_value(7, 99).unwrap();
    assert_eq!(assert_tail_state(&target), 0);
    output + &growth_row("grow_reuse", &target)
}
struct MoveFixture {
    target: DynamicArray<PanicDropItem>,
    source: DynamicArray<PanicDropItem>,
    allocator: BaseAllocator,
    trace: Rc<RefCell<Vec<i32>>>,
    fail: Rc<Cell<i32>>,
}
impl MoveFixture {
    fn new() -> Self {
        let allocator = BaseAllocator::accumulating(4096).unwrap();
        Self {
            target: DynamicArray::with_allocator(2, Some(allocator.clone())),
            source: DynamicArray::with_allocator(2, Some(allocator.clone())),
            allocator,
            trace: Rc::new(RefCell::new(Vec::new())),
            fail: Rc::new(Cell::new(1)),
        }
    }
    fn item(&self, value: i32) -> PanicDropItem {
        PanicDropItem {
            value,
            fail_value: Rc::clone(&self.fail),
            trace: Rc::clone(&self.trace),
        }
    }
    fn row(&self, name: &str) -> String {
        let donor = self.source.value(0).map_or(-1, |item| item.value);
        let trace = self
            .trace
            .borrow()
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{name} {} {} {} {donor} {trace}\n",
            self.target.size(),
            self.source.size(),
            live_allocations(&self.allocator)
        )
    }
    fn failed_rows(&mut self) -> String {
        self.target.append(self.item(1));
        self.source.append(self.item(7));
        let failure = catch_unwind(AssertUnwindSafe(|| self.target.move_from(&mut self.source)));
        assert!(failure.is_err());
        assert!(self.target.needs_tail_scan);
        assert!(!self.source.needs_tail_scan);
        let mut output = self.row("move_receiver_failed");
        unsafe { self.target.storage_extension().set_used_size(0).unwrap() };
        self.target.clear(true);
        output += &self.row("move_receiver_repaired");
        output
    }
    fn success_rows(&mut self) -> String {
        self.target.append(self.item(2));
        self.target.move_from(&mut self.source);
        assert_eq!(self.target.value(0).unwrap().value, 7);
        assert_tail_state(&self.target);
        assert_tail_state(&self.source);
        let mut output = self.row("move_receiver_succeeded");
        self.target.clear(true);
        output += &self.row("move_receiver_cleared");
        output
    }
}
#[test]
fn native_growth_and_assign_receiver_failure_rows_match() {
    let mut fixture = MoveFixture::new();
    let actual = growth_rows() + &fixture.failed_rows() + &fixture.success_rows();
    assert_eq!(actual, EXPECTED);
}
