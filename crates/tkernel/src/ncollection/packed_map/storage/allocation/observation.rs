//! Private allocation-boundary fault injection; never unwind through GlobalAlloc.
use std::cell::RefCell;
#[derive(Default)]
struct Trace {
    events: Vec<String>,
    fail: bool,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
pub(in super::super::super) fn allocation(size: usize) {
    let fail = TRACE.with(|trace| {
        let mut trace = trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return false;
        };
        trace.events.push(format!("allocate:{size}"));
        trace.fail
    });
    assert!(
        !fail,
        "injected bucket request {size}: expected allocation success"
    );
}
pub(in super::super::super) fn release(null: bool) {
    TRACE.with(|trace| {
        if let Some(trace) = trace.borrow_mut().as_mut() {
            trace.events.push(format!("free:{}", u8::from(null)));
        }
    });
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        TRACE.with(|trace| *trace.borrow_mut() = None);
    }
}
pub(in super::super::super::super) fn capture<R>(
    fail: bool,
    run: impl FnOnce() -> R,
) -> (R, Vec<String>) {
    begin(fail);
    let _reset = Reset;
    let result = run();
    let events = TRACE.with(|trace| trace.borrow_mut().as_mut().unwrap().events.clone());
    (result, events)
}

fn begin(fail: bool) {
    TRACE.with(|trace| {
        assert!(
            trace.borrow().is_none(),
            "nested trace: expected inactive observer"
        );
        *trace.borrow_mut() = Some(Trace {
            fail,
            ..Trace::default()
        });
    });
}
