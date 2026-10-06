//! Native operator-> distinguishes owning proxy storage from external borrowed identity.
use rustycad_tkernel::ncollection::forward_range::*;
use std::{cell::RefCell, rc::Rc};

struct ArrowItem {
    value: i32,
    trace: Rc<RefCell<String>>,
}
struct OwnedArrowHost(Rc<RefCell<String>>);
struct BorrowedArrowHost<'a>(&'a ArrowItem, Rc<RefCell<String>>);
struct OwnedArrowAccessor;
struct BorrowedArrowAccessor;

impl ReadAccessor<OwnedArrowHost> for OwnedArrowAccessor {
    type Item = ArrowItem;
    type Value = ArrowItem;
    fn read(host: &OwnedArrowHost) -> ArrowItem {
        host.0.borrow_mut().push('V');
        ArrowItem {
            value: 7,
            trace: host.0.clone(),
        }
    }
    fn snapshot(item: ArrowItem) -> ArrowItem {
        item.trace.borrow_mut().push('S');
        item
    }
}
impl<'a> ReadAccessor<BorrowedArrowHost<'a>> for BorrowedArrowAccessor {
    type Item = &'a ArrowItem;
    type Value = ArrowItem;
    fn read(host: &BorrowedArrowHost<'a>) -> &'a ArrowItem {
        host.1.borrow_mut().push('R');
        host.0
    }
    fn snapshot(item: &ArrowItem) -> ArrowItem {
        item.trace.borrow_mut().push('S');
        ArrowItem {
            value: item.value,
            trace: item.trace.clone(),
        }
    }
}
impl ForwardHost for OwnedArrowHost {
    type Accessors = AccessorSet<Present<OwnedArrowAccessor>, Missing, Missing>;
    fn more(&self) -> bool {
        self.0.borrow_mut().push('M');
        false
    }
    fn advance(&mut self) {
        self.0.borrow_mut().push('N');
    }
}
impl ForwardHost for BorrowedArrowHost<'_> {
    type Accessors = AccessorSet<Present<BorrowedArrowAccessor>, Missing, Missing>;
    fn more(&self) -> bool {
        self.1.borrow_mut().push('M');
        false
    }
    fn advance(&mut self) {
        self.1.borrow_mut().push('N');
    }
}
fn oracle(row: &str) {
    let native = include_str!("../../../tools/oracle/tkernel_forward_native_arrow.expected");
    assert!(
        native.lines().any(|expected| expected == row),
        "expected native row, got {row}"
    );
}

#[test]
fn native_value_arrow_owns_nonclone_value_without_more_or_snapshot() {
    let trace = Rc::new(RefCell::new(String::new()));
    let range = ForwardRange::new(OwnedArrowHost(trace.clone()));
    let proxy = range.cbegin().arrow_value();
    oracle(&format!(
        "value_arrow {} {} {}",
        proxy.value.value,
        std::ptr::eq(proxy.arrow(), &proxy.value) as u8,
        trace.borrow()
    ));
}

#[test]
fn native_reference_arrow_preserves_original_reference_identity() {
    let trace = Rc::new(RefCell::new(String::new()));
    let item = ArrowItem {
        value: 42,
        trace: trace.clone(),
    };
    let mut host = BorrowedArrowHost(&item, trace.clone());
    let iter = ForwardRangeIter::new(&mut host);
    let pointer = iter.arrow_reference();
    oracle(&format!(
        "reference_arrow {} {} {}",
        item.value,
        std::ptr::eq(pointer, &item) as u8,
        trace.borrow()
    ));
}
