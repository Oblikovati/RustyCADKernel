use super::{allocation_trace::*, observed_key::Key};
use rustycad_tkernel::ncollection::{
    BaseAllocator, BoundingTreeInsertion, EBTree, UBTree, UBTreeFiller,
};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
#[derive(Clone, Copy)]
pub struct Case {
    pub editable: bool,
    pub count: i32,
    pub full: bool,
    pub duplicate: bool,
    pub implicit: bool,
    pub rejection: &'static str,
}
impl Case {
    pub fn new(editable: bool, count: i32, full: bool) -> Self {
        Self {
            editable,
            count,
            full,
            duplicate: false,
            implicit: false,
            rejection: "0X0",
        }
    }
    fn prefix(self) -> String {
        format!(
            "dispatch {} {} {} {} {} {} ",
            u8::from(self.editable),
            self.count,
            u8::from(self.full),
            u8::from(self.duplicate),
            u8::from(self.implicit),
            self.rejection
        )
    }
}
struct Recording<Target> {
    target: Target,
    allocator: BaseAllocator,
    calls: String,
    next: u32,
    rejection: &'static str,
}
impl<Target: BoundingTreeInsertion<Key, Payload<'B'>>> Recording<Target> {
    fn rejected_phase(&mut self) -> (char, char, usize) {
        self.next += 1;
        let mut chars = self.rejection.chars();
        let call = chars.next().unwrap().to_digit(10).unwrap();
        let kind = chars.next().unwrap();
        let ordinal = chars.next().unwrap().to_digit(10).unwrap() as usize;
        (if self.next == call { 'C' } else { 'X' }, kind, ordinal)
    }
    fn record(&mut self, object: i32, result: i32) {
        let events = finish();
        let events = if events.is_empty() { "-" } else { &events };
        self.calls.push_str(&format!("{object}:{result}:{events}|"));
    }
}
impl<Target: BoundingTreeInsertion<Key, Payload<'B'>>> BoundingTreeInsertion<Key, Payload<'B'>>
    for Recording<Target>
{
    fn insert_bound(&mut self, object: Key, bounds: Payload<'B'>) -> bool {
        self.insert_bound_ref(&object, &bounds)
    }
    fn insert_bound_ref(&mut self, object: &Key, bounds: &Payload<'B'>) -> bool {
        let rejected = self.rejected_phase();
        start(&self.allocator, rejected);
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.target.insert_bound_ref(object, bounds)
        }));
        self.record(
            object.0.0,
            result.as_ref().map_or(-1, |value| i32::from(*value)),
        );
        result.unwrap_or_else(|error| resume_unwind(error))
    }
    fn tree_view(&self) -> &UBTree<Key, Payload<'B'>> {
        self.target.tree_view()
    }
}
fn queue<Target: BoundingTreeInsertion<Key, Payload<'B'>>>(
    filler: &mut UBTreeFiller<'_, Key, Payload<'B'>, Target>,
    case: Case,
) {
    for index in 1..=case.count {
        let object = if case.duplicate { 1 } else { index };
        filler.add_ref(&Key(Payload(object)), &Payload(object * 10));
    }
}
fn execute<Target: BoundingTreeInsertion<Key, Payload<'B'>>>(
    recording: &mut Recording<Target>,
    pending: &BaseAllocator,
    case: Case,
) -> (i32, bool) {
    let mut filler = UBTreeFiller::new(recording, Some(pending.clone()), case.full);
    queue(&mut filler, case);
    let mut result = -1;
    if case.implicit {
        return (result, false);
    }
    let failed = catch_unwind(AssertUnwindSafe(|| {
        result = filler.fill();
    }))
    .is_err();
    if failed {
        filler.reset();
    }
    (result, failed)
}
fn verify<Target: BoundingTreeInsertion<Key, Payload<'B'>>>(
    target: Target,
    allocator: BaseAllocator,
    case: Case,
) {
    let pending = BaseAllocator::accumulating(65536).unwrap();
    let mut recording = Recording {
        target,
        allocator,
        calls: String::new(),
        next: 0,
        rejection: case.rejection,
    };
    let (result, failed) = execute(&mut recording, &pending, case);
    let row = render_result(&recording, &pending, case, result, failed);
    assert_native_row(case, &row);
}
fn render_result<Target: BoundingTreeInsertion<Key, Payload<'B'>>>(
    recording: &Recording<Target>,
    pending: &BaseAllocator,
    case: Case,
    result: i32,
    failed: bool,
) -> String {
    let tree = recording.tree_view();
    let last = tree.last_node().map_or(-1, |node| node.object().0.0);
    let bounds = tree.root().map_or(-1, |node| node.bounds().0);
    format!(
        "{}{result} {} {} {last} {bounds} {} {}",
        case.prefix(),
        u8::from(failed),
        recording.calls,
        allocations(&recording.allocator),
        allocations(pending)
    )
}
fn assert_native_row(case: Case, row: &str) {
    let native =
        include_str!("../../../../tools/oracle/tkernel_ubtree_filler_dispatch_contracts.expected")
            .lines()
            .find(|line| line.starts_with(&case.prefix()))
            .unwrap();
    assert_eq!(row, native);
}
pub fn native_case(case: Case) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    if case.editable {
        let mut tree = EBTree::with_allocator(allocator.clone());
        verify(tree.native_insertion(), allocator, case);
    } else {
        let mut tree = UBTree::with_allocator(allocator.clone());
        verify(tree.native_insertion(), allocator, case);
    }
}
