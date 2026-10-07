//! Real native EBTree Add tree/map publication and policy dispatch.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "support/ubtree_failed_pair_repair.rs"]
pub mod failed_pair_repair;
#[path = "ebtree_native_add_contracts/fixture.rs"]
mod fixture;
#[path = "support/ubtree_observed_key.rs"]
pub mod observed_key;
use fixture::*;
#[path = "ebtree_native_add_contracts/borrowed.rs"]
mod borrowed;
#[test]
fn successful_native_add_publishes_tree_then_binding_then_prior_leaf_rebind() {
    for count in 0..=3 {
        exact_case(count, 9, "XX0");
    }
}
#[test]
fn duplicate_rejection_never_copies_input_payloads() {
    for count in 1..=3 {
        exact_case(count, 1, "CO1");
    }
}
#[test]
fn root_bounds_and_object_copy_failures_preserve_empty_graph_and_map() {
    for rejection in ["CB1", "CO1"] {
        exact_case(0, 9, rejection);
    }
}
#[test]
fn map_key_copy_failure_preserves_published_tree_and_native_stale_binding() {
    exact_case(0, 9, "CO2");
    for count in 1..=3 {
        exact_case(count, 9, "CO1");
    }
}
#[test]
fn bound_search_failure_leaves_existing_map_bindings() {
    for rejection in ["CB1", "JB1"] {
        exact_case(1, 9, rejection);
    }
}
#[test]
fn union_drop_failure_keeps_new_last_node_without_new_map_binding() {
    exact_case(1, 9, "DB1");
}
#[test]
fn published_parent_failure_preserves_partial_object_and_previous_last_identity() {
    for rejection in ["AB3", "AO3", "NO3"] {
        exact_case(1, 9, rejection);
    }
}

#[test]
fn rust_hash_panic_state_matches_native_pretermination_snapshot() {
    for ordinal in 1..=3 {
        policy_panic_case('H', ordinal);
    }
}
#[test]
fn rust_equal_panic_state_matches_native_pretermination_snapshot() {
    for ordinal in 1..=3 {
        policy_panic_case('Q', ordinal);
    }
}
