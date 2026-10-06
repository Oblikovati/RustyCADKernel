//! Native borrowed Add callback order and graph publication on ordinary/throwing paths.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "ubtree_add_contracts/fixture.rs"]
mod fixture;
use fixture::*;
#[test]
fn empty_root_allocation_precedes_both_copies_and_unwind_frees_it() {
    for rejection in ["XX0", "CB1", "CO1"] {
        exact_case(0, rejection);
    }
}
#[test]
fn borrowed_leaf_insertion_uses_native_pair_and_callback_order() {
    for rejection in ["XX0", "RB1", "CB1", "JB1"] {
        exact_case(1, rejection);
    }
}
#[test]
fn borrowed_nested_insertion_preserves_search_and_partial_bound_mutations() {
    for count in [2, 3] {
        for rejection in ["XX0", "RB1", "CB1", "JB1"] {
            exact_case(count, rejection);
        }
    }
}
#[test]
fn temporary_bound_drop_failure_observes_already_published_last_node() {
    exact_case(1, "DB1");
    exact_case(2, "DB3");
    exact_case(3, "DB3");
}
#[test]
fn search_extent_and_second_union_copy_failures_match_native_cleanup() {
    for rejection in ["EB1", "EB2", "CB2"] {
        exact_case(2, rejection);
    }
}
#[test]
fn unpublished_pair_failure_retains_native_prefix_and_explicit_rust_cleanup() {
    for rejection in ["NB1", "NO2", "AB1"] {
        partial_case(rejection);
    }
}
#[test]
fn borrowed_add_exact_function_binding_accepts_borrowed_payloads() {
    let object = String::from("borrowed");
    borrowed_call(object.as_str());
}
fn borrowed_call<'a>(object: &'a str) {
    use allocation_trace::Payload;
    use rustycad_tkernel::ncollection::UBTree;
    let mut tree = UBTree::<&'a str, Payload<'B'>>::default();
    let call: fn(&mut UBTree<&'a str, Payload<'B'>>, &&'a str, &Payload<'B'>) -> bool =
        UBTree::add_ref;
    assert!(call(&mut tree, &object, &Payload(10)));
    assert!(call(&mut tree, &object, &Payload(20)));
    assert_eq!(*tree.last_node().unwrap().object(), "borrowed");
}
