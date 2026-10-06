use rustycad_tkernel::{
    ncollection::{BaseAllocator, HSequence, Sequence},
    standard::Handle,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI32, Ordering},
};

#[derive(Clone, Default)]
pub struct AppendJournal {
    calls: Arc<Mutex<String>>,
    failed_copy: Arc<AtomicI32>,
}
impl AppendJournal {
    /// Records callback/copy/drop order; `journal.record("g;")`.
    pub fn record(&self, event: &str) {
        self.calls.lock().unwrap().push_str(event);
    }
    /// Starts an isolated copy-failure observation; `journal.reset(2)`.
    pub fn reset(&self, fail: i32) {
        self.calls.lock().unwrap().clear();
        self.failed_copy.store(fail, Ordering::Relaxed);
    }
    /// Reads ordered effects without clearing; `journal.events()`.
    pub fn events(&self) -> String {
        self.calls.lock().unwrap().clone()
    }
}
pub struct AppendItem {
    pub value: i32,
    journal: AppendJournal,
}
impl Clone for AppendItem {
    fn clone(&self) -> Self {
        self.journal.record(&format!("c{};", self.value));
        assert_ne!(
            self.value,
            self.journal.failed_copy.load(Ordering::Relaxed),
            "copy item={}; expected a non-failing item",
            self.value
        );
        Self {
            value: self.value,
            journal: self.journal.clone(),
        }
    }
}
impl Drop for AppendItem {
    fn drop(&mut self) {
        self.journal.record(&format!("d{};", self.value));
    }
}
pub struct DerivedSequence {
    pub base: HSequence<AppendItem>,
    pub alternate: HSequence<AppendItem>,
    journal: AppendJournal,
    pub redirect: bool,
    pub fail_getter: bool,
}
rustycad_tkernel::impl_transient!(
    DerivedSequence,
    "ProjectedAppendSequence",
    HSequence<AppendItem>,
    base
);
impl DerivedSequence {
    /// Models explicitly selected derived ChangeSequence lookup; `source.selected()`.
    pub fn selected(&self) -> &HSequence<AppendItem> {
        self.journal.record("g;");
        assert!(
            !self.fail_getter,
            "getter fail=true; expected successful projection"
        );
        if self.redirect {
            &self.alternate
        } else {
            &self.base
        }
    }
    /// Builds a handled source with separately owned alternate sequence; `DerivedSequence::new(arena, &journal)`.
    pub fn new(arena: &BaseAllocator, journal: &AppendJournal) -> Self {
        Self {
            base: wrapper(arena, &[], journal),
            alternate: wrapper(arena, &[], journal),
            journal: journal.clone(),
            redirect: false,
            fail_getter: false,
        }
    }
}
/// Fills a sequence through the existing owner/node engine; `wrapper(&arena, &[1,2], &journal)`.
pub fn wrapper(
    arena: &BaseAllocator,
    values: &[i32],
    journal: &AppendJournal,
) -> HSequence<AppendItem> {
    let mut items = Sequence::with_allocator(arena.clone());
    for value in values {
        items.append(AppendItem {
            value: *value,
            journal: journal.clone(),
        });
    }
    HSequence::from_owned(items)
}
/// Constructs the ordinary handled source without derived assumptions; `base_handle(&journal)`.
pub fn base_handle(journal: &AppendJournal) -> Handle<HSequence<AppendItem>> {
    Handle::new(wrapper(&BaseAllocator::default(), &[1, 2], journal))
}
/// Reads current values under one shared guard; `values(&sequence)`.
pub fn values(sequence: &HSequence<AppendItem>) -> String {
    let result = sequence.sequence(|s| {
        s.iter()
            .map(|x| x.value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    });
    if result.is_empty() {
        "-".to_owned()
    } else {
        result
    }
}
