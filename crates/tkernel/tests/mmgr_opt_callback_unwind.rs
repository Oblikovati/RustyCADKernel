//! Defined native callback exceptions leave allocation state at the callback phase.
use rustycad_tkernel::standard::{MMgrOpt, StorageAddress};
#[path = "mmgr_opt_callback_unwind/mod.rs"]
mod callback_fixture;
use callback_fixture::{
    CallbackScope, PanicPhase, assert_native_row, captured_allocations, catches, event_shapes,
    install_replacing_callback, replaced_count, set_phase,
};

fn failed_free_case(cell: usize, size: usize, label: &str) {
    let (_scope, (mut allocator, block)) = CallbackScope::with_setup(PanicPhase::Free, || {
        let mut allocator = MMgrOpt::new(true, false, cell, 100, 512);
        let block = allocator.allocate(size).unwrap();
        allocator.bytes_mut(block).unwrap().fill(9);
        (allocator, block)
    });
    let caught = catches(|| allocator.free(Some(block)).unwrap());
    let preserved = allocator
        .bytes(block)
        .is_ok_and(|bytes| bytes == vec![9; size]);
    assert_native_row(&format!(
        "free-{label} {} {} {} {}",
        u8::from(caught),
        u8::from(preserved),
        allocator.purge(false),
        event_shapes()
    ));
    retry_free(&mut allocator, block, label);
}

fn retry_free(allocator: &mut MMgrOpt, block: StorageAddress, label: &str) {
    set_phase(PanicPhase::Never);
    allocator.free(Some(block)).unwrap();
    assert!(allocator.bytes(block).is_err());
    assert_native_row(&format!("retry-{label} {}", event_shapes()));
}

#[test]
fn failed_free_notification_keeps_medium_storage_live_and_retryable() {
    failed_free_case(0, 24, "medium");
    // The same helper also verifies that retry retires the safe handle only after notification.
}

#[test]
fn failed_free_notification_keeps_small_pool_and_large_storage_live() {
    failed_free_case(16, 16, "small");
    failed_free_case(0, 600, "large");
}

fn populated_old_block(allocator: &mut MMgrOpt) -> StorageAddress {
    let old = allocator.allocate(24).unwrap();
    allocator.bytes_mut(old).unwrap().fill(9);
    old
}

fn populated_callback_case(phase: PanicPhase) -> (CallbackScope, (MMgrOpt, StorageAddress)) {
    CallbackScope::with_setup(phase, || {
        let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
        let old = populated_old_block(&mut allocator);
        (allocator, old)
    })
}

#[test]
fn reallocation_free_notification_failure_keeps_both_blocks_live_after_copy() {
    let (_scope, (mut allocator, old)) = populated_callback_case(PanicPhase::Free);
    let caught = catches(|| {
        allocator.reallocate(Some(old), 40).unwrap();
    });
    let moved = captured_allocations()[0];
    let old_live = allocator.bytes(old).is_ok_and(|bytes| bytes == [9; 24]);
    let moved_bytes = allocator.bytes(moved).unwrap();
    assert_native_row(&format!(
        "realloc-free {} {} {} {} {}",
        u8::from(caught),
        u8::from(old_live),
        u8::from(moved_bytes[..24] == [9; 24]),
        u8::from(moved_bytes[24..] == [0; 16]),
        event_shapes()
    ));
    repair_pair(&mut allocator, old, moved);
}

fn repair_pair(allocator: &mut MMgrOpt, old: StorageAddress, moved: StorageAddress) {
    set_phase(PanicPhase::Never);
    allocator.free(Some(old)).unwrap();
    allocator.free(Some(moved)).unwrap();
}

#[test]
fn reallocation_allocation_notification_failure_preserves_old_and_uncopied_new_block() {
    let (_scope, (mut allocator, old)) = populated_callback_case(PanicPhase::Allocate);
    let caught = catches(|| {
        allocator.reallocate(Some(old), 40).unwrap();
    });
    let moved = captured_allocations()[0];
    assert_native_row(&format!(
        "realloc-allocate {} {} {} {}",
        u8::from(caught),
        u8::from(allocator.bytes(old).unwrap() == [9; 24]),
        u8::from(allocator.bytes(moved).unwrap() == [0; 40]),
        event_shapes()
    ));
    repair_pair(&mut allocator, old, moved);
}

#[test]
fn failed_allocation_callback_retains_captured_storage_for_explicit_repair() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let _scope = CallbackScope::new(PanicPhase::Allocate);
    let caught = catches(|| {
        allocator.allocate(40).unwrap();
    });
    let block = captured_allocations()[0];
    assert_native_row(&format!(
        "allocate-unwind {} {} {}",
        u8::from(caught),
        u8::from(allocator.bytes(block).unwrap() == [0; 40]),
        event_shapes()
    ));
    set_phase(PanicPhase::Never);
    allocator.free(Some(block)).unwrap();
}

#[test]
fn fitting_reallocation_and_null_free_do_not_notify() {
    let (_scope, (mut allocator, old)) = populated_callback_case(PanicPhase::Allocate);
    let same = allocator.reallocate(Some(old), 8).unwrap();
    allocator.free(None).unwrap();
    assert_native_row(&format!(
        "silent-paths {} {}",
        u8::from(same == old),
        event_shapes().len()
    ));
    set_phase(PanicPhase::Never);
    allocator.free(Some(old)).unwrap();
}

#[test]
fn callback_can_replace_itself_without_a_held_global_lock() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let _scope = CallbackScope::new(PanicPhase::Never);
    install_replacing_callback();
    let block = allocator.allocate(40).unwrap();
    allocator.free(Some(block)).unwrap();
    assert_native_row(&format!("replace {} {}", replaced_count(), event_shapes()));
}

#[test]
fn invalid_safe_handles_are_rejected_before_notification() {
    let mut allocator = MMgrOpt::new(true, false, 0, 100, 512);
    let _scope = CallbackScope::new(PanicPhase::Free);
    let invalid = StorageAddress::new(1).unwrap();
    assert!(allocator.free(Some(invalid)).is_err());
    assert!(allocator.reallocate(Some(invalid), 40).is_err());
    assert!(event_shapes().is_empty());
}
