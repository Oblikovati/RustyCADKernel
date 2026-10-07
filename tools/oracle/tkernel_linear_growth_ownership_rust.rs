//! Ordinary public growth failure intentionally retains native allocation leaks.
use rustycad_tkernel::ncollection::LinearVector;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[path = "../../crates/tkernel/tests/support/linear_growth_fixture.rs"]
mod fixture;
use fixture::*;
fn public_capture_and_panic_identity() {
    let owner = std::rc::Rc::new(());
    let captured = std::rc::Rc::clone(&owner);
    let token = Box::new(77_u64);
    let identity = std::ptr::from_ref(&*token);
    let mut pending = Some(token);
    let mut vector = LinearVector::with_capacity(1);
    vector.append(1);
    let failure = catch_unwind(AssertUnwindSafe(|| {
        vector.emplace_append_with_relocation(
            |_| std::panic::panic_any(pending.take().unwrap()),
            || {
                drop(captured);
                2
            },
        );
    }))
    .unwrap_err();
    let payload = failure.downcast::<Box<u64>>().unwrap();
    assert_eq!(std::ptr::from_ref(&**payload), identity);
    assert_eq!(std::rc::Rc::strong_count(&owner), 1);
    assert_eq!(vector.as_slice(), [1]);
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    public_capture_and_panic_identity();
    for spec in GROWTH_CASES {
        let trace = GrowthTrace::default();
        let mut vector = filled(spec, &trace);
        let before = vector.data_ptr();
        let result = catch_unwind(AssertUnwindSafe(|| operation(spec, &mut vector, &trace)));
        println!(
            "{}",
            row("native", spec, &vector, before, &trace, result.is_ok())
        );
        trace.active.set(false);
    }
}
