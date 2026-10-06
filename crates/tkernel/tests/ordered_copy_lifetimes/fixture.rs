use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy};
use std::cell::{Cell, RefCell};
pub struct CopyTrace<'a> {
    pub arena: &'a BaseAllocator,
    pub events: RefCell<String>,
    pub copies: Cell<usize>,
    pub capture: Cell<bool>,
    pub retained: Cell<*mut rustycad_tkernel::ncollection::MapLinkNode>,
    pub rejected_copy: Cell<usize>,
    pub rejected_policy: Cell<bool>,
    pub rejected_hash: Cell<bool>,
    pub rejected_equal: Cell<bool>,
}
impl<'a> CopyTrace<'a> {
    pub fn new(arena: &'a BaseAllocator) -> Self {
        Self {
            arena,
            events: RefCell::new(String::new()),
            copies: Cell::new(0),
            capture: Cell::new(false),
            retained: Cell::new(std::ptr::null_mut()),
            rejected_copy: Cell::new(0),
            rejected_policy: Cell::new(false),
            rejected_hash: Cell::new(false),
            rejected_equal: Cell::new(false),
        }
    }
    pub fn active(&self) -> i32 {
        self.arena
            .accumulating_chain()
            .iter()
            .map(|block| block.alloc_count)
            .sum()
    }
    pub fn note(&self, event: &str) {
        self.events.borrow_mut().push_str(&format!(
            "{event}r{}a{},",
            self.arena.ref_count(),
            self.active()
        ));
    }
    pub fn reset(&self, copy: usize, policy: bool, hash: bool, equal: bool) {
        self.events.borrow_mut().clear();
        self.copies.set(0);
        self.rejected_copy.set(copy);
        self.rejected_policy.set(policy);
        self.rejected_hash.set(hash);
        self.rejected_equal.set(equal);
    }
}
pub struct CopyPayload<'a> {
    pub id: i32,
    pub trace: &'a CopyTrace<'a>,
}
impl Clone for CopyPayload<'_> {
    fn clone(&self) -> Self {
        self.trace.note(&format!("C{}", self.id));
        self.trace.copies.set(self.trace.copies.get() + 1);
        assert_ne!(
            self.trace.copies.get(),
            self.trace.rejected_copy.get(),
            "payload copy rejected; expected accepted copy"
        );
        if self.id == 4 && self.trace.capture.get() {
            self.trace.capture_latest();
        }
        Self {
            id: self.id,
            trace: self.trace,
        }
    }
}
impl Drop for CopyPayload<'_> {
    fn drop(&mut self) {
        self.trace.note(&format!("D{}", self.id));
    }
}
pub struct CopyPolicy<'a>(pub &'a CopyTrace<'a>);
impl Clone for CopyPolicy<'_> {
    fn clone(&self) -> Self {
        self.0.note("HC");
        assert!(
            !self.0.rejected_policy.get(),
            "policy copy rejected; expected accepted policy"
        );
        Self(self.0)
    }
}
impl Drop for CopyPolicy<'_> {
    fn drop(&mut self) {
        self.0.note("HD");
    }
}
impl HashPolicy<CopyPayload<'_>> for CopyPolicy<'_> {
    fn hash(&self, key: &CopyPayload<'_>) -> usize {
        self.0.note(&format!("h{}", key.id));
        assert!(
            !(self.0.rejected_hash.get() && key.id == 1),
            "hash rejected key 1; expected accepted key"
        );
        0
    }
    fn equal(&self, a: &CopyPayload<'_>, b: &CopyPayload<'_>) -> bool {
        self.0.note(&format!("e{}:{}", a.id, b.id));
        assert!(
            !self.0.rejected_equal.get(),
            "equality rejected; expected accepted keys"
        );
        a.id == b.id
    }
}
pub fn native_row(label: &str, row: String) {
    let expected = include_str!("../../../../tools/oracle/tkernel_ordered_copy_lifetimes.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(row, expected);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CopyFailure {
    None,
    Empty,
    Policy,
    Hash,
    Equal,
    Payload(usize),
}
impl CopyTrace<'_> {
    pub fn prepare(&self, mode: CopyFailure) {
        let rejected = if let CopyFailure::Payload(index) = mode {
            index
        } else {
            0
        };
        self.reset(
            rejected,
            mode == CopyFailure::Policy,
            mode == CopyFailure::Hash,
            mode == CopyFailure::Equal,
        );
    }
    pub fn verify(&self, label: &str, outcome: Option<&str>) {
        let prefix = outcome.map_or_else(|| label.to_owned(), |value| format!("{label} {value}"));
        native_row(
            label,
            format!(
                "{prefix} {} {} {}",
                self.arena.ref_count(),
                self.active(),
                self.events.borrow()
            ),
        );
    }
}

impl CopyTrace<'_> {
    fn capture_latest(&self) {
        // A zero-byte raw Acc allocation exposes the current cursor without moving it;
        // balancing its native count leaves the in-progress node allocation untouched.
        let pointer = self.arena.allocate_raw(0).unwrap();
        self.retained.set(pointer.as_ptr().cast());
        // SAFETY: only the just-acquired zero-byte raw allocation count is returned here.
        unsafe {
            self.arena.free_raw(pointer).unwrap();
        }
    }
}
pub fn reclaim<N>(prefix: &str, trace: &CopyTrace<'_>) {
    let pointer = std::ptr::NonNull::new(trace.retained.get().cast::<N>()).unwrap();
    trace.events.borrow_mut().clear();
    // SAFETY: this exact completed node survived native failed-copy cleanup; this thread
    // excludes reset/free until adoption acquires its lease. No other owner remains.
    let owner = unsafe { trace.arena.adopt_abandoned_value(pointer) };
    drop(owner);
    trace.verify(&format!("{prefix}_reclaimed"), None);
}

pub fn verify_reset(prefix: &str, trace: &CopyTrace<'_>, first: std::ptr::NonNull<u8>) {
    let drops = trace.events.borrow().matches('D').count() - 1;
    trace.events.borrow_mut().clear();
    trace.arena.reset(false).unwrap();
    let reused = trace.arena.allocate_raw(0).unwrap();
    let label = format!("{prefix}_reset");
    native_row(
        &label,
        format!(
            "{label} {} {} {drops} {}",
            trace.arena.ref_count(),
            usize::from(first == reused),
            trace.events.borrow().len()
        ),
    );
}
