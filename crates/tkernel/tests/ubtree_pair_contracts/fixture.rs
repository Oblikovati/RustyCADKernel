use rustycad_tkernel::ncollection::{BaseAllocator, BoundingVolume, UBTree};
use std::cell::RefCell;
#[derive(Default)]
struct PairObservation {
    allocator: Option<BaseAllocator>,
    active: bool,
    events: String,
    reject: (char, char, usize),
    cleanup_reject: (char, char, usize),
}
thread_local! {
    static OBSERVATION: RefCell<PairObservation> = RefCell::new(PairObservation::default());
}
pub fn allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| usize::try_from(block.alloc_count).unwrap())
        .sum()
}
pub fn start(allocator: &BaseAllocator, reject: (char, char, usize)) {
    OBSERVATION.with(|slot| {
        *slot.borrow_mut() = PairObservation {
            allocator: Some(allocator.clone()),
            active: true,
            events: String::new(),
            reject,
            cleanup_reject: ('X', 'X', 0),
        }
    });
}
pub fn finish() -> String {
    OBSERVATION.with(|slot| {
        let mut observation = slot.borrow_mut();
        observation.active = false;
        std::mem::take(&mut observation.events)
    })
}
fn record(phase: char, kind: char, value: i32) {
    let rejected = OBSERVATION.with(|slot| observe(&mut slot.borrow_mut(), phase, kind, value));
    assert!(
        !rejected,
        "payload {kind}, phase {phase}; expected permitted callback"
    );
}
fn observe(trace: &mut PairObservation, phase: char, kind: char, value: i32) -> bool {
    if !trace.active {
        return false;
    }
    let count = allocations(trace.allocator.as_ref().unwrap());
    trace
        .events
        .push_str(&format!("{phase}{kind}{value}:{count},"));
    reject_match(&mut trace.reject, phase, kind)
        || reject_match(&mut trace.cleanup_reject, phase, kind)
}
fn reject_match(reject: &mut (char, char, usize), phase: char, kind: char) -> bool {
    if (phase, kind) != (reject.0, reject.1) {
        return false;
    }
    reject.2 -= 1;
    if reject.2 != 0 {
        return false;
    }
    *reject = ('X', 'X', 0);
    true
}
pub fn arm_cleanup_failure() {
    OBSERVATION.with(|slot| slot.borrow_mut().cleanup_reject = ('D', 'O', 1));
}

pub fn rejection_tuple(rejection: &str) -> (char, char, usize) {
    let chars: Vec<char> = rejection.chars().collect();
    (chars[0], chars[1], chars[2].to_digit(10).unwrap() as usize)
}
pub struct Payload<const KIND: char>(pub i32);
impl<const KIND: char> Default for Payload<KIND> {
    fn default() -> Self {
        record('N', KIND, 0);
        Self(0)
    }
}
impl<const KIND: char> Clone for Payload<KIND> {
    fn clone(&self) -> Self {
        record('C', KIND, self.0);
        Self(self.0)
    }
    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0;
        record('A', KIND, self.0);
    }
}
impl<const KIND: char> Drop for Payload<KIND> {
    fn drop(&mut self) {
        record('D', KIND, self.0);
    }
}
impl BoundingVolume for Payload<'B'> {
    fn add(&mut self, _: &Self) {}
    fn is_out(&self, _: &Self) -> bool {
        false
    }
    fn square_extent(&self) -> f64 {
        0.0
    }
}
pub type Tree = UBTree<Payload<'O'>, Payload<'B'>>;
pub fn populated(branch: bool) -> (Tree, BaseAllocator) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Tree::with_allocator(allocator.clone());
    tree.add(Payload(1), Payload(10));
    if branch {
        tree.gemmate_ref(0, &Payload(30), &Payload(2), &Payload(20));
    }
    (tree, allocator)
}
pub fn native_row(prefix: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ubtree_pair_contracts.expected")
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
}
pub fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
pub fn pair_is_contiguous(tree: &Tree) -> bool {
    let first = tree.child(tree.root().unwrap(), 0).unwrap();
    let second = tree.child(tree.root().unwrap(), 1).unwrap();
    std::ptr::from_ref(second).addr() - std::ptr::from_ref(first).addr()
        == std::mem::size_of_val(first)
}
