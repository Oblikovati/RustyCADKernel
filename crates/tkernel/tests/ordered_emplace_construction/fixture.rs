use super::scenarios::{EmplaceCase, Operation, RejectedPhase};
use rustycad_tkernel::ncollection::{BaseAllocator, HashPolicy, OrderedMap};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
};
pub struct EmplaceTrace<'a> {
    pub arena: &'a BaseAllocator,
    pub events: RefCell<String>,
    enabled: Cell<bool>,
    rejected: Cell<RejectedPhase>,
}
impl<'a> EmplaceTrace<'a> {
    pub fn new(arena: &'a BaseAllocator) -> Self {
        Self {
            arena,
            events: RefCell::new(String::new()),
            enabled: Cell::new(false),
            rejected: Cell::new(RejectedPhase::None),
        }
    }
    pub fn note(&self, event: &str) {
        if self.enabled.get() {
            self.events.borrow_mut().push_str(&format!("{event},"));
        }
    }
    pub fn active(&self) -> i32 {
        self.arena
            .accumulating_chain()
            .iter()
            .map(|block| block.alloc_count)
            .sum()
    }
    pub fn configure(&self, test: &EmplaceCase) {
        self.enabled.set(true);
        self.rejected.set(test.rejected);
    }
    pub fn construct(&self, id: i32) -> i32 {
        self.note(&format!("C{id}a{}", self.active()));
        assert!(
            self.rejected.get() != RejectedPhase::Construct,
            "constructor rejected key {id}; expected completed key"
        );
        id
    }
}
pub trait KeyIdentity {
    fn identity(&self) -> i32;
    fn display(&self) -> String {
        self.identity().to_string()
    }
}
impl KeyIdentity for i32 {
    fn identity(&self) -> i32 {
        *self
    }
}
pub struct EmplaceKey<'a> {
    pub id: i32,
    pub tag: i32,
    pub trace: &'a EmplaceTrace<'a>,
}
impl KeyIdentity for EmplaceKey<'_> {
    fn identity(&self) -> i32 {
        self.id
    }
    fn display(&self) -> String {
        format!("{}:{}", self.id, self.tag)
    }
}
impl Drop for EmplaceKey<'_> {
    fn drop(&mut self) {
        self.trace.note(&format!("D{}t{}", self.id, self.tag));
    }
}
pub struct EmplacePolicy<'a>(pub &'a EmplaceTrace<'a>);
impl<K: KeyIdentity> HashPolicy<K> for EmplacePolicy<'_> {
    fn hash(&self, key: &K) -> usize {
        let id = key.identity();
        self.0.note(&format!("h{id}"));
        assert!(
            !(self.0.rejected.get() == RejectedPhase::Hash && id == 999),
            "hash rejected key {id}; expected accepted key"
        );
        id as usize
    }
    fn equal(&self, left: &K, right: &K) -> bool {
        self.0
            .note(&format!("e{}:{}", left.identity(), right.identity()));
        assert!(
            self.0.rejected.get() != RejectedPhase::Equal,
            "equality rejected {}:{}; expected accepted keys",
            left.identity(),
            right.identity()
        );
        left.identity() == right.identity()
    }
}
pub fn invoke<K: KeyIdentity>(
    map: &mut OrderedMap<K, EmplacePolicy<'_>>,
    operation: Operation,
    construct: impl FnOnce() -> K,
) -> String {
    let result = catch_unwind(AssertUnwindSafe(|| match operation {
        Operation::Emplace => format!("b{}", usize::from(map.emplace_with(construct))),
        Operation::Emplaced => format!("r{}", map.emplaced_with(construct).identity()),
        Operation::TryEmplace => format!("b{}", usize::from(map.try_emplace_with(construct))),
        Operation::TryEmplaced => format!("r{}", map.try_emplaced_with(construct).identity()),
    }));
    result.unwrap_or_else(|_| "throw".to_owned())
}
fn native_row(label: &str, actual: String) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_ordered_emplace_construction.expected")
            .lines()
            .find(|line| line.starts_with(&format!("{label} ")))
            .unwrap();
    // Column 5 and the `a` event suffixes are arena node counts (see native_node_counts).
    let comparable = |row: &str| {
        let row = super::native_node_counts::mask_column(row, 5);
        super::native_node_counts::without_node_counts(&row)
    };
    assert_eq!(comparable(&actual), comparable(expected));
}
pub fn snapshot<K: KeyIdentity>(
    label: &str,
    result: &str,
    map: &OrderedMap<K, EmplacePolicy<'_>>,
    trace: &EmplaceTrace<'_>,
    same_first: bool,
) {
    let stats = statistics(map, trace, same_first);
    let order = order(map);
    native_row(
        label,
        format!("{label} {result} {stats} {} {order}", trace.events.borrow()),
    );
}
fn order<K: KeyIdentity>(map: &OrderedMap<K, EmplacePolicy<'_>>) -> String {
    let order: String = map
        .iter()
        .map(|key| format!("{},", key.display()))
        .collect();
    if order.is_empty() {
        "-".to_owned()
    } else {
        order
    }
}

fn statistics<K: KeyIdentity>(
    map: &OrderedMap<K, EmplacePolicy<'_>>,
    trace: &EmplaceTrace<'_>,
    same_first: bool,
) -> String {
    format!(
        "{} {} {} {} {}",
        map.len(),
        map.bucket_count(),
        usize::from(!map.buckets().is_none()),
        trace.active(),
        usize::from(same_first)
    )
}
