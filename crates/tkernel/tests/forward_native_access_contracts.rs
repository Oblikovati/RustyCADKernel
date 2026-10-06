//! Native delegation order stays observable independently of a host's More result.
use rustycad_tkernel::ncollection::forward_range::*;
use std::{cell::RefCell, panic::AssertUnwindSafe, rc::Rc};

struct TraceHost {
    position: i32,
    trace: Rc<RefCell<String>>,
    fail_read: bool,
    fail_next: bool,
}
struct TraceAccessor;
impl ReadAccessor<TraceHost> for TraceAccessor {
    type Item = i32;
    type Value = i32;
    fn read(host: &TraceHost) -> i32 {
        host.trace.borrow_mut().push('V');
        assert!(
            !host.fail_read,
            "read failed at {}; expected readable host",
            host.position
        );
        host.position
    }
    fn snapshot(item: i32) -> i32 {
        item
    }
}
impl ForwardHost for TraceHost {
    type Accessors = AccessorSet<Present<TraceAccessor>, Missing, Missing>;
    fn more(&self) -> bool {
        self.trace.borrow_mut().push('M');
        false
    }
    fn advance(&mut self) {
        self.trace.borrow_mut().push('N');
        self.position += 1;
        assert!(
            !self.fail_next,
            "next failed at {}; expected advancing host",
            self.position
        );
    }
}
fn host() -> TraceHost {
    TraceHost {
        position: 7,
        trace: Rc::new(RefCell::new(String::new())),
        fail_read: false,
        fail_next: false,
    }
}
fn oracle(row: &str) {
    let native = include_str!("../../../tools/oracle/tkernel_forward_native_access.expected");
    assert!(
        native.lines().any(|expected| expected == row),
        "unexpected native row: {row}"
    );
}

#[test]
fn exhausted_host_still_delegates_native_dereference_and_postfix() {
    let mut host = host();
    let mut iter = ForwardRangeIter::new(&mut host);
    let current = iter.dereference();
    let previous = iter.postfix_increment();
    oracle(&format!(
        "access {current} {} {} {}",
        *previous,
        host.position,
        host.trace.borrow()
    ));
}

#[test]
fn failed_native_accessor_does_not_advance_or_query_more() {
    let mut host = host();
    host.fail_read = true;
    let mut iter = ForwardRangeIter::new(&mut host);
    assert!(std::panic::catch_unwind(AssertUnwindSafe(|| iter.postfix_increment())).is_err());
    oracle(&format!(
        "read_failure {} {}",
        host.position,
        host.trace.borrow()
    ));
}

#[test]
fn failed_native_next_keeps_host_partial_update() {
    let mut host = host();
    host.fail_next = true;
    let mut iter = ForwardRangeIter::new(&mut host);
    assert!(std::panic::catch_unwind(AssertUnwindSafe(|| iter.postfix_increment())).is_err());
    oracle(&format!(
        "next_failure {} {}",
        host.position,
        host.trace.borrow()
    ));
}

#[test]
fn arrow_and_postfix_expose_native_public_storage() {
    let arrow = ForwardArrowProxy::new(42);
    let mut host = host();
    let mut previous = ForwardRangeIter::new(&mut host).postfix_increment();
    previous.value = 19;
    oracle(&format!(
        "proxy {} {} {}",
        std::ptr::eq(arrow.arrow(), &arrow.value) as u8,
        *arrow,
        *previous
    ));
}

#[test]
fn shared_range_native_access_advances_other_adapters_without_more() {
    let host = host();
    let trace = host.trace.clone();
    let range = ForwardRange::new(host);
    let mut first = range.begin();
    let second = range.cbegin();
    assert_eq!(*first.postfix_increment(), 7);
    assert_eq!(second.dereference(), 8);
    assert_eq!(&*trace.borrow(), "VNV");
}
