//! Every actual heap operation in the measured window, without allocating inside the observer.
#![allow(unsafe_code)]
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Allocate,
    Zeroed,
    Reallocate,
    Free,
}
#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub operation: Operation,
    pub size: usize,
    pub align: usize,
}
const EMPTY: Event = Event {
    operation: Operation::Free,
    size: 0,
    align: 0,
};
#[derive(Clone, Copy)]
struct Trace {
    events: [Event; 32],
    length: usize,
    overflow: bool,
}
thread_local! { static TRACE: Cell<Option<Trace>> = const { Cell::new(None) }; }
fn record(operation: Operation, layout: Layout) {
    let _ = TRACE.try_with(|slot| {
        if let Some(mut trace) = slot.get() {
            if trace.length == trace.events.len() {
                trace.overflow = true;
            } else {
                trace.events[trace.length] = Event {
                    operation,
                    size: layout.size(),
                    align: layout.align(),
                };
                trace.length += 1;
            }
            slot.set(Some(trace));
        }
    });
}
struct ObservedSystem;
#[global_allocator]
static ALLOCATOR: ObservedSystem = ObservedSystem;
// SAFETY: observation cannot allocate or panic; System retains all pointer/layout obligations.
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(Operation::Allocate, layout);
        // SAFETY: the unchanged nonzero layout comes from GlobalAlloc's caller.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(Operation::Zeroed, layout);
        // SAFETY: the unchanged nonzero layout comes from GlobalAlloc's caller.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, address: *mut u8, layout: Layout) {
        record(Operation::Free, layout);
        // SAFETY: this is the original pointer/layout pair supplied by GlobalAlloc's caller.
        unsafe { System.dealloc(address, layout) };
    }
    unsafe fn realloc(&self, address: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(Operation::Reallocate, layout);
        // SAFETY: System receives the caller's valid old allocation and new requested size.
        unsafe { System.realloc(address, layout, size) }
    }
}
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        TRACE.with(|slot| slot.set(None));
    }
}
pub fn capture<R>(operation: impl FnOnce() -> R) -> (R, Vec<Event>) {
    begin();
    let _reset = Reset;
    let result = operation();
    let trace = TRACE.with(|slot| slot.take().unwrap());
    assert!(
        !trace.overflow,
        "heap trace exceeds 32 events: expected bounded ReSize"
    );
    (result, trace.events[..trace.length].to_vec())
}

fn begin() {
    TRACE.with(|slot| {
        assert!(
            slot.get().is_none(),
            "nested heap trace: expected no active observer"
        );
        slot.set(Some(Trace {
            events: [EMPTY; 32],
            length: 0,
            overflow: false,
        }));
    });
}
