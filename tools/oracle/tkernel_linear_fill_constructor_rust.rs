//! Public fill failure probes retain native leaks; strict Miri uses explicit test-only repair.
use rustycad_tkernel::ncollection::LinearVector;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "../../crates/tkernel/tests/support/linear_fill_constructor_fixture.rs"]
mod fixture;
use fixture::{CopyItem, CopyTrace, FILL_CASES, FillSpec, row};

fn run(spec: &FillSpec) {
    let trace = CopyTrace::default();
    let donor = (!spec.owned).then(|| CopyItem::new(99, &trace));
    trace.failing.set(spec.fail);
    trace.active.set(true);
    let result = catch_unwind(AssertUnwindSafe(|| {
        if spec.owned {
            LinearVector::with_value(spec.count, CopyItem::new(99, &trace))
        } else {
            LinearVector::with_value_ref(spec.count, donor.as_ref().unwrap())
        }
    }));
    println!(
        "{}",
        row("native", spec, result.as_ref().ok(), &trace, result.is_ok())
    );
    trace.active.set(false);
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    for spec in &FILL_CASES {
        run(spec);
    }
}
