//! Private allocation-boundary trace of native bucket requests and releases.
use std::cell::RefCell;
thread_local! { static TRACE: RefCell<Option<Vec<String>>> = const { RefCell::new(None) }; }
pub(in super::super::super) fn allocation(size: usize) {
    record(format!("allocate:{size}"));
}
pub(in super::super::super) fn release(null: bool) {
    record(format!("free:{}", u8::from(null)));
}
fn record(event: String) {
    TRACE.with(|trace| {
        if let Some(events) = trace.borrow_mut().as_mut() {
            events.push(event);
        }
    });
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        TRACE.with(|trace| *trace.borrow_mut() = None);
    }
}
/// Run `run` while recording bucket allocation events; `capture(|| map.resize(1000))`.
pub(in super::super::super::super) fn capture<R>(run: impl FnOnce() -> R) -> (R, Vec<String>) {
    TRACE.with(|trace| {
        assert!(
            trace.borrow().is_none(),
            "nested trace: expected inactive observer"
        );
        *trace.borrow_mut() = Some(Vec::new());
    });
    let _reset = Reset;
    let result = run();
    let events = TRACE.with(|trace| trace.borrow_mut().as_mut().unwrap().clone());
    (result, events)
}
