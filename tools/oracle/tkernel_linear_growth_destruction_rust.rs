//! Public token probes retain native abandoned replacements; unit tests repair them explicitly.
use rustycad_tkernel::ncollection::{LinearVector, LinearVectorGrowth};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "../../crates/tkernel/tests/support/linear_growth_destruction_fixture.rs"]
mod fixture;
use fixture::*;
fn run(spec: &DestructionSpec) {
    let trace = DestructionTrace::default();
    let mut token = filled(spec, &trace);
    // SAFETY: all slots/payloads are exclusively owned; no live references alias the receiver.
    let failure = catch_unwind(AssertUnwindSafe(|| unsafe {
        token.reserve(spec.minimum, |item| item.relocate(spec.moving))
    }));
    if failure.is_ok() {
        trace.dead_end.set(0);
    }
    println!("{}", row("native", spec, &token, &trace, failure.is_ok()));
    if failure.is_err() {
        trace.fail_drop.set(None);
        let repair = catch_unwind(AssertUnwindSafe(|| {
            token.repair_with(|i| repaired_item(i, &trace))
        }));
        if repair.is_err() {
            println!("{}", row("repair-failure", spec, &token, &trace, false));
            trace.fail_repair.set(None);
            token.repair_with(|i| repaired_item(i, &trace));
        }
        println!("{}", row("restored", spec, &token, &trace, false));
    }
    trace.active.set(false);
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for spec in DESTRUCTION_CASES {
        run(spec);
    }
}
