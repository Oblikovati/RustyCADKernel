//! Public API probe: failed construction intentionally keeps native leaked allocations.
//! Run normally to compare native traces; Miri uses separate instrumented unit-test repair.
use rustycad_tkernel::ncollection::LinearVector;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "../../crates/tkernel/tests/support/linear_copy_constructor_fixture.rs"]
mod fixture;
use fixture::{COPY_CASES, CopySpec, CopyTrace, filled, row};

fn run(spec: &CopySpec) {
    let trace = CopyTrace::default();
    let source = filled(spec.capacity, spec.count, 10, &trace);
    let mut receiver = filled(spec.receiver_count, spec.receiver_count, 90, &trace);
    let before = if spec.assigning { receiver.data_ptr() } else { source.data_ptr() };
    trace.failing.set(spec.fail); trace.active.set(true);
    let mut copied = None;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        if spec.assigning { receiver.assign(&source); } else { copied = Some(source.clone()); }
    }));
    let result = if spec.assigning { Some(&receiver) } else { copied.as_ref() };
    println!("{}", row("native", spec, &source, result, before, &trace, outcome.is_ok()));
    trace.active.set(false);
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for spec in &COPY_CASES { run(spec); }
}
