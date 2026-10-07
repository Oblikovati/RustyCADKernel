use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, MapLinkNode, StandardAllocator};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    ptr::NonNull,
};
pub struct ResizeTrace {
    pub hashes: [usize; 3],
    pub rejected: Cell<usize>,
    pub calls: Cell<usize>,
    pub active: Cell<bool>,
    pub events: RefCell<String>,
    pub policy_panic: Cell<bool>,
    pub policy_drops: Cell<usize>,
}
impl ResizeTrace {
    pub fn new(hashes: [usize; 3]) -> Self {
        Self {
            hashes,
            rejected: Cell::new(0),
            calls: Cell::new(0),
            active: Cell::new(false),
            events: RefCell::new(String::new()),
            policy_panic: Cell::new(false),
            policy_drops: Cell::new(0),
        }
    }
    pub fn prepare(&self, rejected: usize) {
        self.rejected.set(rejected);
        self.calls.set(0);
        self.events.borrow_mut().clear();
        self.active.set(true);
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct CallbackPanic {
    pub ordinal: usize,
    pub id: i32,
}
pub struct ResizeKey<'a> {
    pub id: i32,
    pub trace: &'a ResizeTrace,
}
impl Drop for ResizeKey<'_> {
    fn drop(&mut self) {
        if self.trace.active.get() {
            self.trace
                .events
                .borrow_mut()
                .push_str(&format!("D{},", self.id));
        }
    }
}
pub struct ResizePolicy<'a>(pub &'a ResizeTrace);
impl HashPolicy<ResizeKey<'_>> for ResizePolicy<'_> {
    fn hash(&self, key: &ResizeKey<'_>) -> usize {
        if self.0.active.get() {
            self.note_hash(key.id);
        }
        self.0.hashes[key.id as usize - 1]
    }
    fn equal(&self, left: &ResizeKey<'_>, right: &ResizeKey<'_>) -> bool {
        left.id == right.id
    }
}
impl ResizePolicy<'_> {
    fn note_hash(&self, id: i32) {
        self.0.events.borrow_mut().push_str(&format!("H{id},"));
        let ordinal = self.0.calls.get() + 1;
        self.0.calls.set(ordinal);
        if ordinal == self.0.rejected.get() {
            std::panic::panic_any(CallbackPanic { ordinal, id });
        }
    }
}
impl Drop for ResizePolicy<'_> {
    fn drop(&mut self) {
        self.0.policy_drops.set(self.0.policy_drops.get() + 1);
        if self.0.policy_panic.replace(false) {
            panic!("policy Drop rejected once; expected successful teardown");
        }
    }
}
pub const PATTERNS: [(&str, [usize; 3]); 4] = [
    ("same", [0, 0, 0]),
    ("split", [0, 101, 202]),
    ("merge", [0, 1009, 2018]),
    ("mixed", [0, 101, 1009]),
];
pub fn active(arena: &BaseAllocator) -> i32 {
    arena
        .accumulating_chain()
        .iter()
        .map(|block| block.alloc_count)
        .sum()
}
pub fn native_line(label: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ordered_resize_progress.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{label} ")))
        .unwrap()
}
pub struct GraphSnapshot {
    pub text: String,
    pub reached: HashSet<usize>,
    pub unique: bool,
}
impl GraphSnapshot {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            reached: HashSet::new(),
            unique: true,
        }
    }
    fn finish(mut self) -> Self {
        if self.text.is_empty() {
            self.text.push('-');
        }
        self
    }
}
pub unsafe fn graph(
    buckets: *const *mut MapLinkNode,
    count: usize,
    key: unsafe fn(*mut MapLinkNode) -> i32,
) -> GraphSnapshot {
    let mut snapshot = GraphSnapshot::empty();
    for bucket in 0..=count {
        let head = unsafe { *buckets.add(bucket) };
        if head.is_null() {
            continue;
        }
        snapshot.text.push_str(&format!("{bucket}:"));
        unsafe {
            graph_chain(&mut snapshot, head, key);
        }
        snapshot.text.push(';');
    }
    snapshot.finish()
}
unsafe fn graph_chain(
    snapshot: &mut GraphSnapshot,
    mut node: *mut MapLinkNode,
    key: unsafe fn(*mut MapLinkNode) -> i32,
) {
    let mut local = HashSet::new();
    while !node.is_null() {
        if !local.insert(node.addr()) {
            snapshot.unique = false;
            snapshot.text.push_str("cycle");
            break;
        }
        snapshot.unique = snapshot.reached.insert(node.addr()) && snapshot.unique;
        snapshot
            .text
            .push_str(&format!("{},", unsafe { key(node) }));
        node = unsafe { (*node).next() };
    }
}
pub unsafe fn reclaim_bucket(pointer: *const *mut MapLinkNode, count: usize) {
    // SAFETY: only called after recorded native-style abandonment transfers this exact allocation.
    unsafe {
        StandardAllocator::new()
            .deallocate(NonNull::new(pointer.cast_mut()).unwrap(), count + 1)
            .unwrap();
    }
}
pub unsafe fn reclaim_nodes<N>(arena: &BaseAllocator, nodes: impl Iterator<Item = *mut N>) {
    for node in nodes {
        // SAFETY: exact initialized abandoned node, with reset/free excluded through lease reacquisition.
        drop(unsafe { arena.adopt_abandoned_value(NonNull::new(node).unwrap()) });
    }
}
pub fn arenas() -> [BaseAllocator; 5] {
    [
        BaseAllocator::fresh_common(),
        BaseAllocator::fresh_heap(),
        BaseAllocator::incremental(4096),
        BaseAllocator::accumulating(4096).unwrap(),
        BaseAllocator::aligned(64),
    ]
}
