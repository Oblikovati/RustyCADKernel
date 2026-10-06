use crate::allocation_trace::*;
use rustycad_tkernel::ncollection::{BaseAllocator, EBTree, HashKey};
#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Key(pub Payload<'O'>);
impl Clone for Key {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        record_key_phase('Q', self.0.0);
        self.0 == other.0
    }
}
impl HashKey for Key {
    fn occt_hash(&self) -> usize {
        record_key_phase('H', self.0.0);
        (self.0.0 % 2).occt_hash()
    }
}
pub type Editable = EBTree<Key, Payload<'B'>>;
pub fn populated(count: usize) -> (Editable, BaseAllocator) {
    let allocator = BaseAllocator::accumulating(65536).unwrap();
    let mut tree = Editable::with_allocator(allocator.clone());
    for index in 1..=count {
        let index = i32::try_from(index).unwrap();
        tree.add_native_ref(&Key(Payload(index)), &Payload(index * 10));
    }
    (tree, allocator)
}
pub fn native_row(count: usize, target: i32, rejection: &str) -> &'static str {
    let prefix = format!("editable {count} {target} {rejection} ");
    include_str!("../../../../tools/oracle/tkernel_ebtree_add_contracts.expected")
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap()
}
pub fn apply(
    tree: &mut Editable,
    allocator: &BaseAllocator,
    target: i32,
    rejection: &str,
) -> (bool, i32, String) {
    let inputs = (Key(Payload(target)), Payload(target * 10));
    start(allocator, rejection_tuple(rejection));
    let mut result = -1;
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        result = i32::from(tree.add_native_ref(&inputs.0, &inputs.1));
    }))
    .is_err();
    (failed, result, finish())
}
fn binding(tree: &Editable, value: i32) -> String {
    let key = Key(Payload(value));
    if !tree.contains(&key) {
        return String::from("-");
    }
    let node = tree.find_node(&key).unwrap();
    let kind = if node.is_leaf() { 'L' } else { 'B' };
    format!("{}{kind}", node.object().0.0)
}
pub fn state(tree: &Editable, target: i32) -> String {
    let base = tree.as_ubtree();
    let last = base.last_node().map_or(-1, |node| node.object().0.0);
    let bounds = base.root().map_or(-1, |node| node.bounds().0);
    format!(
        "{last} {bounds} {} {} {}",
        binding(tree, 1),
        binding(tree, target),
        allocations(base.allocator())
    )
}
pub fn exact_case(count: usize, target: i32, rejection: &str) {
    let (mut tree, allocator) = populated(count);
    let (failed, result, events) = apply(&mut tree, &allocator, target, rejection);
    let row = format!(
        "editable {count} {target} {rejection} {events} {} {result} {}",
        u8::from(failed),
        state(&tree, target)
    );
    assert_eq!(row, native_row(count, target, rejection));
    assert_native_clear(&mut tree, &allocator, count, target, rejection);
}
pub fn partial_case(rejection: &str) {
    let (mut tree, allocator) = populated(1);
    let (failed, result, events) = apply(&mut tree, &allocator, 9, rejection);
    assert!(failed);
    assert_eq!(result, -1);
    let native: Vec<&str> = native_row(1, 9, rejection).split_whitespace().collect();
    let prefix = native[4].strip_suffix("DB100:2,").unwrap();
    assert_eq!(
        events.strip_prefix(prefix).unwrap(),
        failed_union_cleanup_suffix(rejection)
    );
    assert_eq!(state(&tree, 9), "1 10 1L - 1");
    assert_partial_clear(&mut tree, &allocator, rejection);
}
fn assert_partial_clear(tree: &mut Editable, allocator: &BaseAllocator, rejection: &str) {
    start(allocator, ('X', 'X', 0));
    tree.clear();
    assert_eq!(finish(), "DO1:1,DO1:1,DB10:1,");
    assert_eq!(allocations(allocator), 0);
    let prefix = format!("clear 1 9 {rejection} ");
    assert_eq!(clear_row(&prefix), format!("{prefix}DO1:2,DO1:2,DB10:2, 1"));
}
fn clear_row(prefix: &str) -> &'static str {
    include_str!("../../../../tools/oracle/tkernel_ebtree_add_contracts.expected")
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap()
}
fn assert_native_clear(
    tree: &mut Editable,
    allocator: &BaseAllocator,
    count: usize,
    target: i32,
    rejection: &str,
) {
    start(allocator, ('X', 'X', 0));
    tree.clear();
    let row = format!(
        "clear {count} {target} {rejection} {} {}",
        finish(),
        allocations(allocator)
    );
    let prefix = format!("clear {count} {target} {rejection} ");
    assert_eq!(row, clear_row(&prefix));
}
pub fn policy_panic_case(phase: char, ordinal: usize) {
    let (mut tree, allocator) = populated(1);
    let selector = format!("{phase}{ordinal}");
    let (failed, result, events) = apply(&mut tree, &allocator, 9, &format!("{phase}O{ordinal}"));
    assert!(failed);
    assert_eq!(result, -1);
    let prefix = format!("{selector} 86 terminate ");
    let native = include_str!("../../../../tools/oracle/tkernel_ebtree_add_noexcept.expected")
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap();
    // Native terminates; only its phase state is compared to this Rust recovery extension.
    assert_eq!(format!("{events} {}", state(&tree, 9)), native);
    tree.clear();
    assert_eq!(allocations(&allocator), 0);
}
