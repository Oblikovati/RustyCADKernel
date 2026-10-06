//! Actual paired allocation and complete native borrowed Gemmate callback phases.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "ubtree_pair_contracts/fixture.rs"]
mod fixture;
use fixture::*;
#[path = "ubtree_pair_contracts/safety.rs"]
mod safety;
fn construct_case(branch: bool, rejection: &str) {
    let (mut tree, allocator) = populated(branch);
    let before = tree.root().unwrap().child_id(0);
    let inputs = (Payload(99), Payload(3), Payload(30));
    start(&allocator, rejection_tuple(rejection));
    let failed = caught(|| {
        tree.gemmate_ref(0, &inputs.0, &inputs.1, &inputs.2);
    });
    let events = finish();
    let published = before != tree.root().unwrap().child_id(0);
    check_construction(&tree, branch, rejection, failed, published, &events);
    let expected_live = 1 + usize::from(branch) + usize::from(published);
    assert_eq!(allocations(&allocator), expected_live);
    if published {
        assert!(pair_is_contiguous(&tree));
    }
    tree.clear();
    assert_eq!(allocations(&allocator), 0);
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
    if published {
        assert_eq!(events, native[3]);
        return;
    }
    check_cleanup(events, native[3], rejection, branch);
}
fn check_cleanup(events: &str, native: &str, rejection: &str, branch: bool) {
    assert!(events.starts_with(native), "{rejection}: {events}");
    let extra = &events[native.len()..];
    assert_eq!(extra, cleanup_suffix(rejection, branch), "{rejection}");
}
fn cleanup_suffix(rejection: &str, branch: bool) -> String {
    let old_object = if branch { 0 } else { 1 };
    let old_bounds = if branch { 30 } else { 10 };
    let count = if branch { 3 } else { 2 };
    format_cleanup(cleanup_values(rejection, old_object, old_bounds), count)
}
fn cleanup_values(rejection: &str, old_object: i32, old_bounds: i32) -> Vec<i32> {
    match rejection {
        "NB1" | "NO1" => vec![],
        "NB2" | "NO2" => vec![0, 0],
        "AB1" => vec![0, old_bounds, 0, 0],
        "AO1" => vec![old_object, old_bounds, 0, 0],
        "AO2" => vec![old_object, old_bounds, 3, 0],
        "AB2" => vec![old_object, old_bounds, 3, 30],
        _ => panic!("rejection {rejection}; expected prepublication failure"),
    }
}
fn format_cleanup(values: Vec<i32>, count: usize) -> String {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let kind = if index % 2 == 0 { 'O' } else { 'B' };
            format!("D{kind}{value}:{count},")
        })
        .collect()
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
