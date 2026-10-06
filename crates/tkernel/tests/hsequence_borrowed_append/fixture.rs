use rustycad_tkernel::ncollection::{BaseAllocator, HSequence, Sequence, SequenceObserver};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Clone, Default)]
pub struct CopyJournal(Arc<Mutex<String>>);
impl CopyJournal {
    /// Starts a new observed phase; `journal.clear()`.
    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
    /// Records real constructor/destructor effects; `journal.record("c9;")`.
    pub fn record(&self, event: &str) {
        self.0.lock().unwrap().push_str(event);
    }
    /// Copies the current event sequence; `journal.events()`.
    pub fn events(&self) -> String {
        let events = self.0.lock().unwrap().clone();
        if events.is_empty() {
            "-".to_owned()
        } else {
            events
        }
    }
}
pub struct BorrowedItem {
    pub number: i32,
    fail: bool,
    reads: AtomicUsize,
    journal: CopyJournal,
}
impl BorrowedItem {
    /// Builds a donor without Default, Clone or copy assignment; `BorrowedItem::new(9, false, &journal)`.
    pub fn new(number: i32, fail: bool, journal: &CopyJournal) -> Self {
        Self {
            number,
            fail,
            reads: AtomicUsize::new(0),
            journal: journal.clone(),
        }
    }
    /// Reads a mutable source observation made during native const copy; `donor.reads()`.
    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::Relaxed)
    }
}
impl Drop for BorrowedItem {
    fn drop(&mut self) {
        self.journal.record(&format!("d{};", self.number));
    }
}
/// Uses the allocator's checked diagnostic to count actual live typed nodes; `live_nodes(&arena)`.
/// A successful zero-live reset only rewinds empty storage; no live pointer is invalidated.
pub fn live_nodes(arena: &BaseAllocator) -> usize {
    match arena.reset(false) {
        Ok(()) => 0,
        Err(error) => {
            assert!(error.message().contains("live allocations: expected 0"));
            error
                .message()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap()
        }
    }
}
/// Observes actual allocation and canonical size before constructing a result; `copy_item(donor, arena, observer)`.
pub fn copy_item(
    donor: &BorrowedItem,
    arena: &BaseAllocator,
    observer: &SequenceObserver,
) -> BorrowedItem {
    donor.reads.fetch_add(1, Ordering::Relaxed);
    donor.journal.record(&format!(
        "c{}/{}/{};",
        donor.number,
        live_nodes(arena),
        observer.snapshot().size
    ));
    assert!(
        !donor.fail,
        "copy donor={}; expected a non-failing donor",
        donor.number
    );
    BorrowedItem::new(donor.number, donor.fail, &donor.journal)
}
pub struct BorrowedCase {
    pub arena: BaseAllocator,
    pub journal: CopyJournal,
    pub owner: HSequence<BorrowedItem>,
    pub observer: SequenceObserver,
}
impl BorrowedCase {
    /// Creates a real allocator-backed prefix; `BorrowedCase::new(2)`.
    pub fn new(prefix: i32) -> Self {
        let arena = BaseAllocator::incremental(4096);
        let journal = CopyJournal::default();
        let mut items = Sequence::with_allocator(arena.clone());
        for number in 0..prefix {
            items.append(BorrowedItem::new(number, false, &journal));
        }
        let observer = items.observer();
        Self {
            arena,
            journal,
            owner: HSequence::from_owned(items),
            observer,
        }
    }
    /// Reads current values without holding the lock across callbacks; `case.values()`.
    pub fn values(&self) -> String {
        let result = self.owner.sequence(|s| {
            s.iter()
                .map(|x| x.number.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });
        if result.is_empty() {
            "-".to_owned()
        } else {
            result
        }
    }
    /// Runs the explicit copy policy through the wrapper; `case.append(&donor)`.
    pub fn append(&self, donor: &BorrowedItem) {
        self.owner.append_ref_with_constructor(donor, |source| {
            copy_item(source, &self.arena, &self.observer)
        });
    }
}
