//! Shared actual allocation counts and payload phases; defaults receive the installed thread fixture.
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
/// Counts actual outstanding allocations; `allocations(&allocator)`.
pub fn allocations(allocator: &BaseAllocator) -> usize {
    allocator
        .accumulating_chain()
        .iter()
        .map(|block| usize::try_from(block.alloc_count).unwrap())
        .sum()
}
/// Installs the recorder and one rejected callback; `start(&allocator,('C','B',1))`.
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
/// Stops recording and returns its trace; `let events=finish();`.
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
/// Rejects the first subsequent object destructor; `arm_cleanup_failure()`.
pub fn arm_cleanup_failure() {
    OBSERVATION.with(|slot| slot.borrow_mut().cleanup_reject = ('D', 'O', 1));
}

/// Decodes the native three-character callback selector; `rejection_tuple("CB1")`.
pub fn rejection_tuple(rejection: &str) -> (char, char, usize) {
    let chars: Vec<char> = rejection.chars().collect();
    (chars[0], chars[1], chars[2].to_digit(10).unwrap() as usize)
}
/// Value-only observable payload; `Payload::<'O'>(1)`.
#[derive(Debug, PartialEq)]
pub struct Payload<const KIND: char>(
    /// Integer identity observed by native callbacks.
    pub i32,
);
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
    fn add(&mut self, other: &Self) {
        self.0 += other.0;
        record('J', 'B', self.0);
    }
    fn is_out(&self, _: &Self) -> bool {
        record('R', 'B', self.0);
        false
    }
    fn square_extent(&self) -> f64 {
        record('E', 'B', self.0);
        f64::from(self.0)
    }
}
/// The allocation-observing tree fixture; `Tree::default()`.
pub type Tree = UBTree<Payload<'O'>, Payload<'B'>>;

/// Observes real key-policy callbacks; `record_key_phase('H', 1)`.
pub fn record_key_phase(phase: char, value: i32) {
    record(phase, 'O', value);
}

/// Gives cleanup after an unpublished pair failure in the shared bound100 fixture.
/// Example: `failed_union_cleanup_suffix("NB1")`.
pub fn failed_union_cleanup_suffix(rejection: &str) -> &'static str {
    match rejection {
        "NB1" => "DB100:1,",
        "NO2" => "DO0:2,DB0:2,DB100:1,",
        "AB1" => "DO0:2,DB10:2,DO0:2,DB0:2,DB100:1,",
        _ => panic!("rejection {rejection}; expected unpublished pair failure"),
    }
}
