//! Native failed pair construction ownership, explicit typed repair, and reset safety.
#![allow(unsafe_code)]
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "support/ubtree_failed_pair_repair.rs"]
pub mod failed_pair_repair;
/// Shared native paired-tree construction and observation helpers.
#[path = "ubtree_pair_contracts/fixture.rs"]
pub mod fixture;
use failed_pair_repair::{PairRepair, initialized_slots};
use fixture::*;
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingNode, UBTree};
type ObservedNode = BoundingNode<Payload<'O'>, Payload<'B'>>;
fn operation(tree: &mut Tree, allocator: &BaseAllocator, rejection: &str) -> String {
    let inputs = (Payload(99), Payload(3), Payload(30));
    start(allocator, rejection_tuple(rejection));
    assert!(caught(|| {
        tree.gemmate_ref(0, &inputs.0, &inputs.1, &inputs.2);
    }));
    finish()
}
fn clear_phase(tree: &mut Tree, allocator: &BaseAllocator) -> String {
    start(allocator, ('X', 'X', 0));
    tree.clear();
    let cleared = finish();
    assert_eq!(allocations(allocator), 1);
    cleared
}
fn repair_phase(
    repair: PairRepair<'_, ObservedNode>,
    allocator: &BaseAllocator,
    rejection: &str,
) -> String {
    start(allocator, ('X', 'X', 0));
    // SAFETY: same original pair, initialized slots measured by native constructor phases.
    unsafe { repair.reclaim(initialized_slots(rejection)) };
    let events = finish();
    assert_eq!(allocations(allocator), 0);
    if events.is_empty() {
        String::from("-")
    } else {
        events
    }
}
fn exact_repair(branch: bool, rejection: &str) {
    let (mut tree, allocator) = populated(branch);
    let repair = PairRepair::<ObservedNode>::capture(&allocator);
    let events = operation(&mut tree, &allocator, rejection);
    let outstanding = allocations(&allocator);
    let cleared = clear_phase(&mut tree, &allocator);
    let repaired = repair_phase(repair, &allocator, rejection);
    let prefix = format!("retained {} {rejection} ", u8::from(branch));
    let native =
        include_str!("../../../tools/oracle/tkernel_ubtree_failed_pair_contracts.expected")
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap();
    assert_eq!(
        format!("{prefix}1 {events} {outstanding} {cleared} 1 {repaired} 0"),
        native
    );
}
fn rejections() -> [&'static str; 8] {
    ["NB1", "NB2", "NO1", "NO2", "AB1", "AB2", "AO1", "AO2"]
}
#[test]
fn retained_pair_and_test_only_repair_match_all_native_failure_phases() {
    for branch in [false, true] {
        for rejection in rejections() {
            exact_repair(branch, rejection);
        }
    }
}
fn incremental_failure(rejection: &str) {
    let allocator = BaseAllocator::incremental(4096);
    let initial = allocator.allocate_raw(0).unwrap();
    let mut tree = Tree::with_allocator(allocator.clone());
    tree.add_ref(&Payload(1), &Payload(10));
    operation(&mut tree, &allocator, rejection);
    assert!(allocator.reset(false).is_err());
    tree.clear();
    allocator.reset(false).unwrap();
    assert_eq!(allocator.allocate_raw(0).unwrap(), initial);
}
#[test]
fn abandoned_incremental_pair_releases_lease_but_live_root_still_prevents_reset() {
    for rejection in rejections() {
        incremental_failure(rejection);
    }
}
#[test]
fn abandoned_payload_borrows_are_never_accessed_after_their_owner_expires() {
    let allocator = BaseAllocator::incremental(4096);
    {
        let text = String::from("borrowed");
        let mut tree = UBTree::with_allocator(allocator.clone());
        tree.add_ref(&text.as_str(), &Payload(10));
        start(&allocator, ('A', 'B', 2));
        assert!(caught(|| {
            tree.gemmate_ref(0, &Payload(99), &text.as_str(), &Payload(20));
        }));
        finish();
        tree.clear();
    }
    // The abandoned bytes contain expired references; reset frees them without reading payloads.
    allocator.reset(true).unwrap();
}
