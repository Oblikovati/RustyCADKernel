use super::*;
#[test]
fn sequential_native_growth_inspects_zero_descriptors() {
    for increment in [1, 2, 4, 16] {
        let mut target = DynamicArray::with_increment(increment);
        reserve_blocks(&mut target, 64);
        for value in 0..128_i32 {
            target.set_value(value, value).unwrap();
            assert!(!target.needs_tail_scan);
            assert_eq!(assert_tail_state(&target), 0);
        }
        target.erase_last();
        assert!(!target.needs_tail_scan);
        assert_tail_state(&target);
    }
}
#[test]
fn initialized_tail_uses_exact_scan_until_canonical_reset() {
    let mut target = DynamicArray::<i32>::with_increment(2);
    reserve_blocks(&mut target, 4);
    target[5] = 50;
    assert!(target.needs_tail_scan);
    assert_eq!(assert_tail_state(&target), 3);
    target.set_value(0, 7).unwrap();
    assert_eq!(target[5], 50);
    assert_eq!(assert_tail_state(&target), 3);
    target.set_value(7, 8).unwrap();
    assert!(target.needs_tail_scan);
    assert!(!target.has_initialized_tail());
    assert_eq!(assert_tail_state(&target), 4);
    target.clear(false);
    assert!(!target.needs_tail_scan);
    assert_eq!(assert_tail_state(&target), 0);
}
#[test]
fn copy_move_insert_and_empty_stride_preserve_gate_invariants() {
    let mut source = DynamicArray::with_increment(2);
    source.set_value(5, 5_i32).unwrap();
    source.insert_before(1, 9).unwrap();
    assert_tail_state(&source);
    let mut copied = source.clone();
    assert!(!copied.needs_tail_scan);
    let mut moved = DynamicArray::default();
    moved.move_from(&mut copied);
    assert_tail_state(&moved);
    assert_tail_state(&copied);
    moved.clear(false);
    moved.set_increment(1);
    moved.append(4);
    assert!(!moved.needs_tail_scan);
    assert_tail_state(&moved);
}
#[test]
fn protected_lifetime_publication_and_parameter_reset_canonicalize() {
    let mut target = DynamicArray::<i32>::with_increment(2);
    reserve_blocks(&mut target, 2);
    target[3] = 30;
    target[1] = 10;
    assert!(target.needs_tail_scan);
    unsafe { target.storage_extension().set_used_size(4).unwrap() };
    assert!(!target.needs_tail_scan);
    assert_tail_state(&target);
    retained_reset(&mut target);
    target.storage_extension().clear_blocks();
    let source = DynamicArray::with_increment(1);
    target
        .storage_extension()
        .copy_parameters(&source, true)
        .unwrap();
    assert_tail_state(&target);
}
fn retained_reset(target: &mut DynamicArray<i32>) {
    let source = DynamicArray::with_increment(2);
    unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, false)
            .unwrap()
    };
    assert!(!target.needs_tail_scan);
    assert_tail_state(target);
    // Same stride, unchanged original live values: restore cleanup ownership before release.
    unsafe { target.storage_extension().set_used_size(4).unwrap() };
}

#[test]
fn failed_raw_validation_preserves_uncertain_gate() {
    let owner = BaseAllocator::accumulating(4096).unwrap();
    let mut target = DynamicArray::with_allocator(2, Some(owner));
    reserve_blocks(&mut target, 1);
    target[1] = 20_i32;
    let source = DynamicArray::with_allocator(2, Some(BaseAllocator::incremental(4096)));
    let result = unsafe {
        target
            .storage_extension()
            .copy_parameters_retained(&source, true)
    };
    assert!(result.is_err());
    assert!(unsafe { target.storage_extension().set_used_size(3) }.is_err());
    assert!(target.needs_tail_scan);
    assert_tail_state(&target);
    target.storage_extension().clear_blocks();
    assert!(!target.needs_tail_scan);
    assert_tail_state(&target);
}
#[test]
fn partial_copy_failure_only_publishes_canonical_complete_blocks() {
    let mut source = DynamicArray::with_increment(2);
    for value in 1..=3 {
        source.append(value);
    }
    let mut target = DynamicArray::with_increment(2);
    let error = target
        .storage_extension()
        .copy_date_with(&source, |value| {
            if *value == 3 {
                return Err(crate::standard::Failure::new("copy failed"));
            }
            Ok(*value)
        })
        .unwrap_err();
    assert_eq!(target.used, 2);
    assert!(!target.needs_tail_scan);
    assert_tail_state(&target);
    drop(error);
}
#[test]
fn move_transfers_uncertainty_and_copy_omits_hidden_owned_tail() {
    let mut source = DynamicArray::<i32>::with_increment(2);
    reserve_blocks(&mut source, 2);
    source[3] = 3;
    let copied = source.clone();
    assert!(!copied.needs_tail_scan);
    assert_tail_state(&copied);
    let mut target = DynamicArray::default();
    target.move_from(&mut source);
    assert!(target.needs_tail_scan);
    assert_eq!(assert_tail_state(&target), 2);
    assert!(!source.needs_tail_scan);
    assert_tail_state(&source);
}
