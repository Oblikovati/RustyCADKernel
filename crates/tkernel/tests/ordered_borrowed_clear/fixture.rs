use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy};
use std::{
    cell::{Cell, RefCell},
    sync::{Mutex, MutexGuard},
};
// Common allocator handle counts are process-global, so the entire oracle target shares this gate.
static CLEAR_ORACLE_SERIAL: Mutex<()> = Mutex::new(());
pub fn oracle_guard() -> MutexGuard<'static, ()> {
    CLEAR_ORACLE_SERIAL.lock().unwrap()
}
pub struct ClearTrace<'a> {
    pub old: &'a BaseAllocator,
    pub next: &'a BaseAllocator,
    pub events: RefCell<String>,
    pub active: Cell<bool>,
    pub rejected: Cell<Option<char>>,
    pub failure_events: RefCell<Option<String>>,
    common: bool,
    baseline: i32,
}
impl<'a> ClearTrace<'a> {
    pub fn new(old: &'a BaseAllocator, next: &'a BaseAllocator, common: bool) -> Self {
        Self {
            old,
            next,
            events: RefCell::new(String::new()),
            active: Cell::new(false),
            rejected: Cell::new(None),
            failure_events: RefCell::new(None),
            common,
            baseline: old.ref_count(),
        }
    }
    pub fn old_refs(&self) -> i32 {
        self.old.ref_count() - if self.common { self.baseline } else { 0 }
    }
    pub fn old_active(&self) -> i32 {
        if self.common {
            -1
        } else {
            active_allocations(self.old)
        }
    }
    fn note(&self, role: char, id: i32) {
        if !self.active.get() {
            return;
        }
        self.events.borrow_mut().push_str(&format!(
            "{role}{id}o{}n{}a{},",
            self.old_refs(),
            self.next.ref_count(),
            self.old_active()
        ));
    }
    fn reject_drop(&self, role: char, id: i32) {
        if !self.active.get() || self.rejected.get() != Some(role) {
            return;
        }
        self.rejected.set(None);
        *self.failure_events.borrow_mut() = Some(self.events.borrow().clone());
        panic!("destructor rejected {role}:{id}; expected normal completion");
    }
    pub fn verify(&self, label: &str, stats: [i64; 11]) {
        let events = self.events.borrow();
        let events = if events.is_empty() { "-" } else { &events };
        let stats = stats.map(|value| value.to_string()).join(" ");
        assert_eq!(format!("{label} {stats} {events}"), native_line(label));
    }
    pub fn verify_termination_prefix(&self, label: &str) {
        let line = native_line(label);
        let expected = line.split_whitespace().last().unwrap();
        assert_eq!(self.failure_events.borrow().as_deref(), Some(expected));
        assert_eq!(
            native_line(&format!("{label}_exit")),
            format!("{label}_exit 73")
        );
    }
}
pub fn active_allocations(arena: &BaseAllocator) -> i32 {
    arena
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}
pub struct ClearPayload<'a> {
    pub id: i32,
    pub role: char,
    pub trace: &'a ClearTrace<'a>,
}
impl Drop for ClearPayload<'_> {
    fn drop(&mut self) {
        self.trace.note(self.role, self.id);
        self.trace.reject_drop(self.role, self.id);
    }
}
pub struct ClearPolicy;
impl HashPolicy<ClearPayload<'_>> for ClearPolicy {
    fn hash(&self, _: &ClearPayload<'_>) -> usize {
        0
    }
    fn equal(&self, left: &ClearPayload<'_>, right: &ClearPayload<'_>) -> bool {
        left.id == right.id
    }
}
fn native_line(label: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ordered_borrowed_clear.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap()
}
