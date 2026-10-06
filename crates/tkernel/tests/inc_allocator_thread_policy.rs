//! Native enabled allocation skips head repair; synchronization remains a Rust safety contract.
mod inc_thread_fixture;
use inc_thread_fixture::Trace;
use rustycad_tkernel::ncollection::BaseAllocator;

#[test]
fn fast_path_preserves_nearly_full_and_zero_space_heads() {
    for enabled in [false, true] {
        let mut trace = Trace::new(&format!("threshold-{}", u8::from(enabled)), enabled);
        for size in [1024, 1016, 8, 0, 1] {
            trace.allocate(size, true);
        }
        trace.assert_native();
    }
}

#[test]
fn fast_path_keeps_head_after_available_space_order_changes() {
    for enabled in [false, true] {
        let mut trace = Trace::new(&format!("sorted-{}", u8::from(enabled)), enabled);
        for size in [1600, 1700, 64, 64, 64] {
            trace.allocate(size, false);
        }
        trace.assert_native();
    }
}

#[test]
fn rewind_keeps_policy_and_restores_newest_block_first() {
    for enabled in [false, true] {
        let mut trace = Trace::new(&format!("rewind-{}", u8::from(enabled)), enabled);
        trace.allocate(2048, false);
        trace.allocate(2048, false);
        trace.reset(false);
        trace.allocate(8, false);
        trace.allocate(8, false);
        trace.assert_native();
    }
}

#[test]
fn release_restores_default_size_without_resetting_policy() {
    for enabled in [false, true] {
        let mut trace = Trace::new(&format!("release-{}", u8::from(enabled)), enabled);
        trace.allocate(2048, false);
        trace.allocate(2048, false);
        trace.reset(true);
        for size in [1024, 11256, 8, 8] {
            trace.allocate(size, false);
        }
        trace.assert_native();
    }
}

#[test]
fn fifth_block_growth_remains_identical_in_both_modes() {
    for enabled in [false, true] {
        let mut trace = Trace::new(&format!("growth-{}", u8::from(enabled)), enabled);
        for size in [2048, 2048, 2048, 2048, 2048, 8, 8] {
            trace.allocate(size, false);
        }
        trace.assert_native();
    }
}

#[test]
fn toggling_shared_handle_changes_policy_without_reordering_immediately() {
    let mut trace = Trace::new("toggle", true);
    for size in [1600, 1700, 64, 64] {
        trace.allocate(size, false);
    }
    trace.arena.clone().set_thread_safe(false).unwrap();
    trace.allocate(8, false);
    trace.allocate(8, false);
    trace.arena.clone().set_thread_safe(true).unwrap();
    trace.allocate(8, false);
    trace.allocate(8, false);
    trace.assert_native();
}

#[test]
fn toggling_policy_preserves_typed_allocation_lease_and_values() {
    let arena = BaseAllocator::incremental(2048);
    let value = arena.allocate_value(41_u64);
    for enabled in [true, false, true, true, false] {
        arena.set_thread_safe(enabled).unwrap();
        assert_eq!(*value, 41);
        assert!(arena.reset(false).is_err());
        assert!(arena.reset(true).is_err());
    }
    drop(value);
    arena.reset(false).unwrap();
    arena.reset(true).unwrap();
}
