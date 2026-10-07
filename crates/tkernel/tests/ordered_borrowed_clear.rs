//! Native borrowed-handle destruction observations; Rust panic recovery is a separate domain.
#[path = "ordered_borrowed_clear/contracts.rs"]
mod contracts;
#[path = "ordered_borrowed_clear/fixture.rs"]
mod fixture;
#[path = "ordered_borrowed_clear/modes.rs"]
mod modes;
// Shared support; this target uses only the event-suffix filter.
#[allow(dead_code)]
#[path = "support/native_node_counts.rs"]
mod native_node_counts;
