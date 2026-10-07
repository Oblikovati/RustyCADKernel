//! Actual filler virtual insertion spans; queue and self-assignment policies are separate.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "ubtree_filler_dispatch_contracts/fixture.rs"]
mod fixture;
#[path = "support/ubtree_observed_key.rs"]
pub mod observed_key;
use fixture::{Case, native_case};
#[test]
fn native_base_and_editable_dispatch_match_both_random_orders() {
    for editable in [false, true] {
        for (count, full) in [
            (1, false),
            (1, true),
            (2, false),
            (2, true),
            (3, false),
            (3, true),
        ] {
            native_case(Case::new(editable, count, full));
        }
    }
}
#[test]
fn derived_duplicates_reject_before_copies_during_explicit_and_implicit_fill() {
    for full in [false, true] {
        for implicit in [false, true] {
            native_case(Case {
                duplicate: true,
                implicit,
                ..Case::new(true, 3, full)
            });
        }
    }
}
#[test]
fn dropping_filler_retains_selected_native_insertion_dispatch() {
    native_case(Case {
        implicit: true,
        ..Case::new(false, 2, true)
    });
    native_case(Case {
        implicit: true,
        ..Case::new(true, 2, false)
    });
}
#[test]
fn copied_input_failures_preserve_native_virtual_call_prefixes() {
    for editable in [false, true] {
        for rejection in ["1B1", "1O1"] {
            native_case(Case {
                rejection,
                ..Case::new(editable, 2, true)
            });
        }
    }
}
#[test]
fn editable_binding_failures_retain_committed_tree_in_filler_dispatch() {
    native_case(Case {
        rejection: "1O2",
        ..Case::new(true, 1, true)
    });
    native_case(Case {
        rejection: "2O1",
        ..Case::new(true, 2, true)
    });
}

#[path = "ubtree_filler_dispatch_contracts/compatibility.rs"]
mod compatibility;
#[path = "ubtree_filler_dispatch_contracts/phase_gap.rs"]
mod phase_gap;
