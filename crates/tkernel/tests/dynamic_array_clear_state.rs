//! Clear interruption keeps native destruction phases while isolating dead slots from safe arrays.
#![allow(unsafe_code)]
#[path = "dynamic_array_clear_state/contracts.rs"]
mod contracts;
#[path = "dynamic_array_clear_state/payload.rs"]
mod payload;
#[path = "dynamic_array_clear_state/trace.rs"]
mod trace;
#[test]
fn clear_interruption_and_repair_rows_match_pinned_native() {
    let mut rows = Vec::new();
    for release in [false, true] {
        for fail in [0, 1, 2, 3, 5] {
            rows.extend(trace::clear_rows(release, fail));
        }
    }
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_dynamic_clear_state.expected")
    );
}
