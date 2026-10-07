use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy};
use std::cell::{Cell, RefCell};
pub struct BorrowTrace<'a> {
    pub arena: &'a BaseAllocator,
    pub events: RefCell<String>,
    pub active: Cell<bool>,
    pub fail_value: Cell<bool>,
    pub fail_key: Cell<bool>,
    pub fail_hash: Cell<bool>,
    pub fail_equal: Cell<bool>,
}
impl<'a> BorrowTrace<'a> {
    pub fn new(arena: &'a BaseAllocator) -> Self {
        Self {
            arena,
            events: RefCell::new(String::new()),
            active: Cell::new(false),
            fail_value: Cell::new(false),
            fail_key: Cell::new(false),
            fail_hash: Cell::new(false),
            fail_equal: Cell::new(false),
        }
    }
    pub fn allocations(&self) -> i32 {
        self.arena
            .accumulating_chain()
            .iter()
            .map(|block| block.alloc_count)
            .sum()
    }
    pub fn note(&self, event: &str) {
        if self.active.get() {
            self.events
                .borrow_mut()
                .push_str(&format!("{event}a{},", self.allocations()));
        }
    }
    pub fn construct(&self, id: i32) -> i32 {
        self.note(&format!("V{id}"));
        assert!(
            !self.fail_value.get(),
            "value {id} constructor rejected; expected successful construction"
        );
        id
    }
    pub fn finish(&self) {
        self.active.set(false);
        self.fail_hash.set(false);
        self.fail_equal.set(false);
    }
}
pub struct BorrowKey<'a> {
    pub id: i32,
    pub tag: i32,
    pub trace: &'a BorrowTrace<'a>,
}
impl Clone for BorrowKey<'_> {
    fn clone(&self) -> Self {
        self.trace.note(&format!("K{}", self.id));
        assert!(
            !self.trace.fail_key.get(),
            "key {} copy rejected; expected successful clone",
            self.id
        );
        Self {
            id: self.id,
            tag: self.tag,
            trace: self.trace,
        }
    }
}
impl Drop for BorrowKey<'_> {
    fn drop(&mut self) {
        self.trace.note(&format!("D{}", self.id));
    }
}
pub struct BorrowPolicy;
impl HashPolicy<BorrowKey<'_>> for BorrowPolicy {
    fn hash(&self, key: &BorrowKey<'_>) -> usize {
        key.trace.note(&format!("H{}", key.id));
        assert!(
            !(key.trace.fail_hash.get() && key.id == 999),
            "key {} hash rejected; expected accepted query",
            key.id
        );
        key.id as usize
    }
    fn equal(&self, left: &BorrowKey<'_>, right: &BorrowKey<'_>) -> bool {
        left.trace.note(&format!("E{}:{}", left.id, right.id));
        assert!(
            !left.trace.fail_equal.get(),
            "keys {}:{} equality rejected; expected accepted comparison",
            left.id,
            right.id
        );
        left.id == right.id
    }
}
pub trait EmplaceValue<'a>: Sized {
    const KIND: &'static str;
    fn construct(trace: &'a BorrowTrace<'a>, id: i32) -> Self;
    fn id(&self) -> i32;
}
impl<'a> EmplaceValue<'a> for i32 {
    const KIND: &'static str = "scalar";
    fn construct(trace: &'a BorrowTrace<'a>, id: i32) -> Self {
        trace.construct(id)
    }
    fn id(&self) -> i32 {
        *self
    }
}
pub struct BorrowValue<'a> {
    pub id: i32,
    pub trace: &'a BorrowTrace<'a>,
}
impl<'a> EmplaceValue<'a> for BorrowValue<'a> {
    const KIND: &'static str = "tracked";
    fn construct(trace: &'a BorrowTrace<'a>, id: i32) -> Self {
        Self {
            id: trace.construct(id),
            trace,
        }
    }
    fn id(&self) -> i32 {
        self.id
    }
}
impl Drop for BorrowValue<'_> {
    fn drop(&mut self) {
        self.trace.note(&format!("W{}", self.id));
    }
}
pub fn native_row(label: &str, row: &str) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ordered_borrowed_emplace.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap();
    assert_eq!(format!("{label} {row}"), expected);
}
