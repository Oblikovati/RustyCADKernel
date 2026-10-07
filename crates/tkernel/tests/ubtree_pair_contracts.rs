#![allow(unsafe_code)]
//! Actual paired allocation and complete native borrowed Gemmate callback phases.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "support/ubtree_failed_pair_repair.rs"]
pub mod failed_pair_repair;
#[path = "ubtree_pair_contracts/fixture.rs"]
mod fixture;
use failed_pair_repair::{PairRepair, initialized_slots};
use fixture::*;
use rustycad_tkernel::ncollection::{BaseAllocator, BoundingNode};
#[path = "ubtree_pair_contracts/safety.rs"]
mod safety;
fn construct_case(branch: bool, rejection: &str) {
    let (mut tree, allocator) = populated(branch);
    let before = tree.root().unwrap().child_id(0);
    let repair = PairRepair::<BoundingNode<Payload<'O'>, Payload<'B'>>>::capture(&allocator);
    let inputs = (Payload(99), Payload(3), Payload(30));
    start(&allocator, rejection_tuple(rejection));
    let failed = caught(|| {
        tree.gemmate_ref(0, &inputs.0, &inputs.1, &inputs.2);
    });
    let events = finish();
    let published = before != tree.root().unwrap().child_id(0);
    check_construction(&tree, branch, rejection, failed, published, &events);
    verify_pair_ownership(&mut tree, &allocator, repair, published, rejection, branch);
}
fn check_construction(
    tree: &Tree,
    branch: bool,
    rejection: &str,
    failed: bool,
    published: bool,
    events: &str,
) {
    let prefix = format!("construct {} {rejection}", u8::from(branch));
    let native: Vec<&str> = native_row(&prefix).split_whitespace().collect();
    assert_eq!(u8::from(failed).to_string(), native[4]);
    assert_eq!(u8::from(published).to_string(), native[5]);
    assert_eq!(tree.root().unwrap().bounds().0.to_string(), native[6]);
    assert_eq!(tree.root().unwrap().object().0.to_string(), native[7]);
    assert_eq!(events, native[3]);
}
fn verify_pair_ownership(
    tree: &mut Tree,
    allocator: &BaseAllocator,
    repair: PairRepair<'_, BoundingNode<Payload<'O'>, Payload<'B'>>>,
    published: bool,
    rejection: &str,
    branch: bool,
) {
    assert_eq!(allocations(allocator), 2 + usize::from(branch));
    if published {
        assert!(pair_is_contiguous(tree));
        assert!(repair.matches(tree.child(tree.root().unwrap(), 0).unwrap()));
    } else {
        // SAFETY: native prepublication failure retains this exact pair and completed slots.
        unsafe { repair.reclaim(initialized_slots(rejection)) };
    }
    tree.clear();
    assert_eq!(allocations(allocator), 0);
}
#[test]
fn native_borrowed_child_construction_and_assignment_matrix() {
    for branch in [false, true] {
        for rejection in [
            "XX0", "NB1", "NB2", "NO1", "NO2", "AB1", "AB2", "AB3", "AO1", "AO2", "AO3", "NO3",
        ] {
            construct_case(branch, rejection);
        }
    }
}
fn destroy_case(index: i32) {
    let (mut tree, allocator) = populated(true);
    tree.gemmate_ref(0, &Payload(99), &Payload(3), &Payload(30));
    start(&allocator, ('X', 'X', 0));
    if index < 0 {
        tree.clear();
    } else {
        tree.kill(0, index);
    }
    let events = finish();
    let prefix = format!("destroy {index}");
    let native: Vec<&str> = native_row(&prefix).split_whitespace().collect();
    assert_eq!(events, native[2]);
    assert_eq!(allocations(&allocator).to_string(), native[4]);
    tree.clear();
    assert_eq!(allocations(&allocator), 0);
}
#[test]
fn native_clear_and_kill_retain_pair_blocks_during_payload_destruction() {
    for index in [-1, 0, 1] {
        destroy_case(index);
    }
}
#[test]
fn implicit_destruction_matches_native_preorder_and_retained_blocks() {
    let (mut tree, allocator) = populated(true);
    tree.gemmate_ref(0, &Payload(99), &Payload(3), &Payload(30));
    start(&allocator, ('X', 'X', 0));
    drop(tree);
    let events = finish();
    assert_eq!(
        events,
        native_row("implicit ").split_whitespace().nth(1).unwrap()
    );
    assert_eq!(allocations(&allocator), 0);
}
