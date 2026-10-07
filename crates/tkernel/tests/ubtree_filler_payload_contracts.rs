//! Explicit queue construction and safe native self-assignment capability contracts.
#[path = "support/ubtree_allocation_trace.rs"]
pub mod allocation_trace;
#[path = "ubtree_filler_payload_contracts/policy.rs"]
mod policy;
#[path = "ubtree_filler_payload_contracts/queue.rs"]
mod queue;
use allocation_trace::*;
use policy::Moving;
fn caught(operation: impl FnOnce()) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).is_err()
}
fn event_text() -> String {
    let events = finish();
    if events.is_empty() {
        String::from("-")
    } else {
        events
    }
}
fn assert_native(prefix: &str, row: &str) {
    let native =
        include_str!("../../../tools/oracle/tkernel_ubtree_filler_payload_contracts.expected")
            .lines()
            .find(|line| line.starts_with(prefix))
            .unwrap();
    assert_eq!(row, native);
}
#[test]
fn copy_only_queue_construction_and_failure_phases_match_native() {
    for before in [false, true] {
        for rejection in ["XX0", "CO1", "CO2", "CB1", "CB2", "DB1", "DO1"] {
            queue::queue_case::<Payload<'O'>, Payload<'B'>>(false, before, rejection);
        }
    }
}
#[test]
fn move_capable_queue_construction_and_failure_phases_match_native() {
    for before in [false, true] {
        for rejection in ["XX0", "CO1", "CB1", "MO1", "MB1", "DO1", "DB1"] {
            queue::queue_case::<Moving<'O'>, Moving<'B'>>(true, before, rejection);
        }
    }
}
#[path = "ubtree_filler_payload_contracts/assignment.rs"]
mod assignment;
#[path = "ubtree_filler_payload_contracts/compatibility.rs"]
mod compatibility;
