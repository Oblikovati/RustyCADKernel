use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy};
use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard, PoisonError};
/// Serializes the tests of this binary. The default-constructor rows read the process-wide
/// handle count of the common allocator against a baseline; any test running beside them that
/// clones or drops an allocator handle would shift that count, so every test holds this lock.
pub fn serial() -> MutexGuard<'static, ()> {
    static SERIAL: Mutex<()> = Mutex::new(());
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}
/// Records copy (C), hash (h), equality (e) and destructor (D) callbacks with the allocator
/// handle count (`r`); the native arena node counts are not compared (see native_node_counts).
pub struct CopyTrace<'a> {
    pub arena: &'a BaseAllocator,
    pub events: RefCell<String>,
}
impl<'a> CopyTrace<'a> {
    pub fn new(arena: &'a BaseAllocator) -> Self {
        Self {
            arena,
            events: RefCell::new(String::new()),
        }
    }
    pub fn note(&self, event: &str) {
        self.events
            .borrow_mut()
            .push_str(&format!("{event}r{},", self.arena.ref_count()));
    }
    pub fn clear(&self) {
        self.events.borrow_mut().clear();
    }
    pub fn verify(&self, label: &str, outcome: Option<&str>) {
        let prefix = outcome.map_or_else(|| label.to_owned(), |value| format!("{label} {value}"));
        native_copy_row(
            label,
            format!(
                "{prefix} {} * {}",
                self.arena.ref_count(),
                self.events.borrow()
            ),
        );
    }
}
pub struct CopyPayload<'a> {
    pub id: i32,
    pub trace: &'a CopyTrace<'a>,
}
impl Clone for CopyPayload<'_> {
    fn clone(&self) -> Self {
        self.trace.note(&format!("C{}", self.id));
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
        0
    }
    fn equal(&self, a: &CopyPayload<'_>, b: &CopyPayload<'_>) -> bool {
        self.0.note(&format!("e{}:{}", a.id, b.id));
        a.id == b.id
    }
}
fn expected_row(label: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ordered_copy_lifetimes.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap()
}
pub fn native_row(label: &str, row: String) {
    assert_eq!(row, expected_row(label));
}
// The live-node column (just before the events) and each event's `a` count are not compared.
fn native_copy_row(label: &str, row: String) {
    let expected = expected_row(label);
    let column = expected.split(' ').count() - 2;
    let expected = super::native_node_counts::mask_column(expected, column);
    assert_eq!(
        row,
        super::native_node_counts::without_node_counts(&expected)
    );
}
