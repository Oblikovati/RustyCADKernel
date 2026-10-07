//! The existing process-wide callback survives initialization without an initialization event.
use rustycad_tkernel::standard::{MMgrOpt, StorageAddress};
use std::cell::Cell;

thread_local! {
    static INITIALIZE_CALLBACK_COUNT: Cell<usize> = const { Cell::new(0) };
}

fn observe(_: bool, _: StorageAddress, _: usize, _: usize) {
    // Per-thread counting excludes unrelated allocations in this test binary's parallel cases.
    INITIALIZE_CALLBACK_COUNT.with(|count| count.set(count.get() + 1));
}

struct RestoreCallback;
impl Drop for RestoreCallback {
    fn drop(&mut self) {
        // Restore even after a failed assertion so other tests never inherit this observer.
        MMgrOpt::set_call_back_function(None);
    }
}

#[test]
fn initialization_preserves_global_callback_without_emitting_allocation_events() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let _restore = RestoreCallback;
    MMgrOpt::set_call_back_function(Some(observe));
    INITIALIZE_CALLBACK_COUNT.with(|count| count.set(0));
    allocator.initialize_native_free_lists();
    let initialized = INITIALIZE_CALLBACK_COUNT.with(Cell::get);
    allocator.allocate(40).unwrap();
    let after_allocate = INITIALIZE_CALLBACK_COUNT.with(Cell::get);
    let row = format!("initialize-callback {initialized} {after_allocate}");
    let fixture = include_str!("../../../../tools/oracle/tkernel_mmgr_opt_initialize.expected");
    assert!(
        fixture.lines().any(|expected| expected == row),
        "missing native row: {row}"
    );
}
