#![allow(unsafe_code)]
use crate::allocation_trace::*;
use crate::observed_key::Key;
use rustycad_tkernel::ncollection::{BaseAllocator, EBTree};
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
pub fn retained_pair_case(rejection: &str) {
    use crate::failed_pair_repair::{PairRepair, initialized_slots};
    use rustycad_tkernel::ncollection::BoundingNode;
    let (mut tree, allocator) = populated(1);
    let repair = PairRepair::<BoundingNode<Key, Payload<'B'>>>::capture(&allocator);
    let (failed, result, events) = apply(&mut tree, &allocator, 9, rejection);
    assert!(failed);
    let row = format!(
        "editable 1 9 {rejection} {events} 1 {result} {}",
        state(&tree, 9)
    );
    assert_eq!(row, native_row(1, 9, rejection));
    assert_native_clear(&mut tree, &allocator, 1, 9, rejection);
    assert_eq!(allocations(&allocator), 1);
    // SAFETY: exact failed typed pair, known completed children, and no reset/intervening owner.
    unsafe { repair.reclaim(initialized_slots(rejection)) };
    assert_eq!(allocations(&allocator), 0);
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
