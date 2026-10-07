use super::fixture::{CopyItem, CopyTrace, events, live_allocations, make_source, source_state};
use super::{AssertUnwindSafe, BaseAllocator, HSequence, Sequence, catch_unwind};
use std::cell::{Cell, RefCell};

fn observed_success(
    name: &str,
    source: &Sequence<CopyItem<'_>>,
    copied: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
) -> String {
    assert_eq!(source.allocator(), copied.allocator());
    format!(
        "{name} {} {} {} {}\n",
        copied.len(),
        source_state(source),
        live_allocations(trace.arena),
        events(trace)
    )
}
fn construct_observed(
    name: &str,
    wrapped: bool,
    source: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
) -> String {
    if wrapped {
        let copied = HSequence::from_sequence(source);
        let row = copied.sequence(|sequence| observed_success(name, source, sequence, trace));
        trace.events.borrow_mut().clear();
        return row;
    }
    let copied = source.clone();
    let row = observed_success(name, source, &copied, trace);
    trace.events.borrow_mut().clear();
    row
}
fn case_rows(wrapped: bool, length: i32, failure: i32) -> String {
    let arena = BaseAllocator::accumulating(4096).unwrap();
    let trace = CopyTrace {
        arena: &arena,
        events: RefCell::new(String::new()),
        failing: Cell::new(failure),
    };
    let source = make_source(&trace, length);
    let kind = if wrapped { "wrapper" } else { "sequence" };
    let name = format!("{kind}_{length}_{failure}");
    run_case(&name, wrapped, &source, &trace)
}
fn run_case(
    name: &str,
    wrapped: bool,
    source: &Sequence<CopyItem<'_>>,
    trace: &CopyTrace<'_>,
) -> String {
    let baseline = trace.arena.ref_count();
    let attempt = catch_unwind(AssertUnwindSafe(|| {
        construct_observed(name, wrapped, source, trace)
    }));
    let mut rows = match attempt {
        Ok(row) => row,
        Err(_) => failure_row(name, source, trace),
    };
    rows += &ending_row(name, trace, baseline);
    rows
}
fn ending_row(name: &str, trace: &CopyTrace<'_>, baseline: i32) -> String {
    format!(
        "{name}_end {} {} {}\n",
        live_allocations(trace.arena),
        events(trace),
        u8::from(trace.arena.ref_count() == baseline)
    )
}
fn failure_row(name: &str, source: &Sequence<CopyItem<'_>>, trace: &CopyTrace<'_>) -> String {
    let row = format!(
        "{name} - {} {} {}\n",
        source_state(source),
        live_allocations(trace.arena),
        events(trace)
    );
    trace.events.borrow_mut().clear();
    row
}
/// Generates all native rows through ordinary Sequence Clone and HSequence construction; `all_rows()`.
pub fn all_rows() -> String {
    let mut rows = String::new();
    for wrapped in [false, true] {
        rows += &case_rows(wrapped, 0, 0);
        for failure in 0..=3 {
            rows += &case_rows(wrapped, 3, failure);
        }
    }
    rows
}
