//! Test-only allocation counters, filtered to actual packed-node layouts within marked operations.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{PackedInteger, PackedMapNode};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
#[derive(Clone, Copy, Default)]
pub(super) struct NodeCounts {
    pub allocations: usize,
    pub releases: usize,
}
#[derive(Clone, Copy)]
struct TraceState {
    size: usize,
    align: usize,
    counts: NodeCounts,
}
thread_local! { static NODE_TRACE: Cell<Option<TraceState>> = const { Cell::new(None) }; }
struct ObservedSystem;
#[global_allocator]
static TEST_ALLOCATOR: ObservedSystem = ObservedSystem;
fn observe(layout: Layout, allocation: bool) {
    let _ = NODE_TRACE.try_with(|slot| {
        let Some(mut state) = slot.get() else {
            return;
        };
        if (state.size, state.align) != (layout.size(), layout.align()) {
            return;
        }
        if allocation {
            state.counts.allocations += 1;
        } else {
            state.counts.releases += 1;
        }
        slot.set(Some(state));
    });
}
// SAFETY: observation does not touch payloads or allocate; System owns every actual allocation.
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: GlobalAlloc's caller supplies the valid nonzero layout.
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() {
            observe(layout, true);
        }
        result
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the allocation and zeroing are delegated unchanged to System.
        let result = unsafe { System.alloc_zeroed(layout) };
        if !result.is_null() {
            observe(layout, true);
        }
        result
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        observe(layout, false);
        // SAFETY: this is the same pointer/layout pair received by GlobalAlloc.
        unsafe { System.dealloc(pointer, layout) };
    }
}
struct TraceReset;
impl Drop for TraceReset {
    fn drop(&mut self) {
        NODE_TRACE.with(|slot| slot.set(None));
    }
}
pub(super) fn measure<T: PackedInteger, R>(operation: impl FnOnce() -> R) -> (R, NodeCounts) {
    let node = Layout::new::<PackedMapNode<T>>();
    NODE_TRACE.with(|slot| {
        assert!(
            slot.get().is_none(),
            "nested node trace: expected no active trace"
        );
        slot.set(Some(TraceState {
            size: node.size(),
            align: node.align(),
            counts: NodeCounts::default(),
        }));
    });
    let _reset = TraceReset;
    let result = operation();
    let counts = NODE_TRACE.with(|slot| slot.get().unwrap().counts);
    (result, counts)
}
