use super::{BaseAllocator, Sequence};
use std::cell::{Cell, RefCell};

pub struct CopyTrace<'a> {
    pub arena: &'a BaseAllocator,
    pub events: RefCell<String>,
    pub failing: Cell<i32>,
}
pub struct CopyItem<'a> {
    pub number: i32,
    pub reads: Cell<usize>,
    pub trace: &'a CopyTrace<'a>,
}
impl Clone for CopyItem<'_> {
    fn clone(&self) -> Self {
        self.record_copy();
        Self {
            number: self.number,
            reads: Cell::new(0),
            trace: self.trace,
        }
    }
}
impl CopyItem<'_> {
    fn record_copy(&self) {
        self.reads.set(self.reads.get() + 1);
        self.trace.events.borrow_mut().push_str(&format!(
            "c{}/{};",
            self.number,
            live_allocations(self.trace.arena)
        ));
        assert_ne!(
            self.number,
            self.trace.failing.get(),
            "copy item failed; expected a non-failing item"
        );
    }
}
impl Drop for CopyItem<'_> {
    fn drop(&mut self) {
        self.trace
            .events
            .borrow_mut()
            .push_str(&format!("d{};", self.number));
    }
}
/// Counts actual outstanding arena allocations, including abandoned raw nodes; `live_allocations(arena)`.
pub fn live_allocations(arena: &BaseAllocator) -> i32 {
    arena
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}
/// Constructs the native donor sequence without invoking copy hooks; `make_source(&trace, 3)`.
pub fn make_source<'a>(trace: &'a CopyTrace<'a>, length: i32) -> Sequence<CopyItem<'a>> {
    let mut source = Sequence::with_allocator(trace.arena.clone());
    for number in 1..=length {
        source.emplace_append(|| CopyItem {
            number,
            reads: Cell::new(0),
            trace,
        });
    }
    source
}
/// Reads a trace without inventing any unobserved operations; `events(&trace)`.
pub fn events(trace: &CopyTrace<'_>) -> String {
    let events = trace.events.borrow().clone();
    if events.is_empty() {
        "-".to_owned()
    } else {
        events
    }
}
/// Observes donor values and mutable const-copy read counts; `source_state(&source)`.
pub fn source_state(source: &Sequence<CopyItem<'_>>) -> String {
    let values = source
        .iter()
        .map(|item| item.number.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let reads = source
        .iter()
        .map(|item| item.reads.get().to_string())
        .collect::<Vec<_>>()
        .join(",");
    if source.is_empty() {
        "- -".to_owned()
    } else {
        format!("{values} {reads}")
    }
}
