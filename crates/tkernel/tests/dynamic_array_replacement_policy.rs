//! Actual replacement slots and injected value operations reproduce native lifetime traces.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{DynamicArray, DynamicArrayStorageExtension};
use std::{
    cell::RefCell,
    mem::MaybeUninit,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
mod dynamic_array_replacement_policy {
    pub mod contracts;
    pub mod oracle;
}
type ReplacementTrace = Rc<RefCell<Vec<String>>>;
type ReplacementStorage<'a> = DynamicArrayStorageExtension<'a, ReplacementItem>;
struct ReplacementItem {
    value: i32,
    trace: ReplacementTrace,
}
impl ReplacementItem {
    fn new(value: i32, trace: &ReplacementTrace) -> Self {
        record_event(trace, 'C', value);
        assert_ne!(value, 99, "constructor value99: expected injected failure");
        Self {
            value,
            trace: Rc::clone(trace),
        }
    }
    fn assign_from(&mut self, donor: &mut Self) {
        record_event(&self.trace, 'A', donor.value);
        self.value = donor.value;
        donor.value = -7;
        assert_ne!(
            self.value, 77,
            "assignment value77: expected injected failure"
        );
    }
    fn transfer_move(donor: &mut Self) -> Self {
        record_event(&donor.trace, 'M', donor.value);
        let value = donor.value;
        donor.value = -7;
        Self {
            value,
            trace: Rc::clone(&donor.trace),
        }
    }
    fn transfer_copy(donor: &mut Self) -> Self {
        record_event(&donor.trace, 'K', donor.value);
        assert_ne!(donor.value, 88, "copy value88: expected injected failure");
        Self {
            value: donor.value,
            trace: Rc::clone(&donor.trace),
        }
    }
}
impl Drop for ReplacementItem {
    fn drop(&mut self) {
        record_event(&self.trace, 'D', self.value);
    }
}
fn record_event(trace: &ReplacementTrace, kind: char, value: i32) {
    trace.borrow_mut().push(format!("{kind}{value}"));
}
#[derive(Clone, Copy)]
enum ReplacementMode {
    Assign,
    Move,
    Copy,
    Direct,
}
unsafe fn apply_replacement(
    pointer: *mut ReplacementItem,
    request: i32,
    mode: ReplacementMode,
    trace: &ReplacementTrace,
) {
    let construct = || ReplacementItem::new(request, trace);
    // Caller owns the live slot and repairs it after a construction/transfer panic.
    unsafe { dispatch_replacement(pointer, mode, construct) };
}
unsafe fn dispatch_replacement(
    pointer: *mut ReplacementItem,
    mode: ReplacementMode,
    construct: impl FnOnce() -> ReplacementItem,
) {
    use ReplacementItem as Item;
    use ReplacementMode::{Assign, Copy, Direct, Move};
    type Ops<'a> = ReplacementStorage<'a>;
    // Explicit modes select the native capability branch; Rust traits do not infer it.
    unsafe {
        match mode {
            Assign => Ops::replace_at_assign(pointer, construct, Item::assign_from),
            Move => Ops::replace_at_reconstruct(pointer, construct, Item::transfer_move),
            Copy => Ops::replace_at_reconstruct(pointer, construct, Item::transfer_copy),
            Direct => Ops::replace_at_direct(pointer, construct),
        }
    }
}

fn take_trace(trace: &ReplacementTrace) -> String {
    std::mem::take(&mut *trace.borrow_mut()).join(",")
}
