//! Protected allocator calls operate on the canonical shared engine, not detached policy values.
use rustycad_tkernel::ncollection::{BaseAllocator, IncAllocatorExtension};
use std::fmt::Write;

fn native_row(label: &str, actual: String) {
    let fixture =
        include_str!("../../../tools/oracle/tkernel_allocator_protected_operations.expected");
    let expected = fixture
        .lines()
        .find(|row| row.split_whitespace().next() == Some(label));
    assert_eq!(
        Some(actual.as_str()),
        expected,
        "protected native row {label}"
    );
}
#[test]
fn direct_get_key_uses_rounded_block_size_for_all_address_values() {
    for size in [61, 64, 65] {
        let arena = BaseAllocator::accumulating(size).unwrap();
        let extension = arena.accumulating_extension().unwrap();
        let label = format!("keys-{size}");
        let mut row = label.clone();
        for address in [0, 1, 63, 64, 65, usize::MAX] {
            write!(row, " {}", extension.key_for_address(address).value()).unwrap();
        }
        native_row(&label, row);
    }
}
#[test]
fn direct_key_and_lookup_miss_output_are_distinct() {
    let arena = BaseAllocator::accumulating(64).unwrap();
    let extension = arena.accumulating_extension().unwrap();
    let direct = extension.key_for_address(0).value();
    let miss = extension.find_block(0);
    native_row(
        "key-miss",
        format!(
            "key-miss {direct} {} {}",
            miss.key.value(),
            u8::from(miss.block.is_none())
        ),
    );
}
#[test]
fn slow_allocation_repairs_used_heap_even_when_fast_mode_is_enabled() {
    for slow in [false, true] {
        let arena = BaseAllocator::incremental(2048);
        arena.set_thread_safe(true).unwrap();
        let first = arena.allocate_raw(1024).unwrap();
        let second = allocate_selected(&arena, 1016, slow);
        let third = arena.allocate_raw(8).unwrap();
        let label = format!("threshold-{}", u8::from(slow));
        native_row(
            &label,
            format!(
                "{label} {} {}",
                second.as_ptr().addr() - first.as_ptr().addr(),
                u8::from(third.as_ptr() == first.as_ptr().wrapping_add(2040))
            ),
        );
    }
}
fn allocate_selected(arena: &BaseAllocator, size: usize, slow: bool) -> std::ptr::NonNull<u8> {
    if !slow {
        return arena.allocate_raw(size).unwrap();
    }
    arena
        .incremental_extension()
        .unwrap()
        .allocate_slow(size)
        .unwrap()
}
#[test]
fn slow_allocation_restores_decreasing_free_space_order() {
    for slow in [false, true] {
        let [first, second, fast] = sorted_pointer_choices(slow).map(u8::from);
        let label = format!("sorting-{}", u8::from(slow));
        native_row(&label, format!("{label} {first} {second} {fast}"));
    }
}
fn sorted_pointer_choices(slow: bool) -> [bool; 3] {
    let arena = BaseAllocator::incremental(2048);
    arena.set_thread_safe(true).unwrap();
    let first = arena.allocate_raw(1600).unwrap().as_ptr();
    let second = arena.allocate_raw(1700).unwrap().as_ptr();
    arena.allocate_raw(64).unwrap();
    allocate_selected(&arena, 64, slow);
    let next = arena.allocate_raw(64).unwrap().as_ptr();
    let tail = arena.allocate_raw(8).unwrap().as_ptr();
    [
        next == first.wrapping_add(1728),
        next == second.wrapping_add(1700),
        tail == next.wrapping_add(64),
    ]
}
#[test]
fn protected_growth_changes_real_block_capacity_and_reset_behavior() {
    let arena = BaseAllocator::incremental(2048);
    arena.incremental_extension().unwrap().increase_block_size();
    let first = allocate_selected(&arena, 2048, true);
    let second = allocate_selected(&arena, 14336, true);
    arena.reset(false).unwrap();
    let reused = allocate_selected(&arena, 16384, true);
    let same = reused == first;
    let after_release = growth_after_release(&arena);
    native_row(
        "grow-storage",
        format!(
            "grow-storage {} {} {}",
            u8::from(second.as_ptr().addr() == first.as_ptr().addr() + 2048),
            u8::from(same),
            u8::from(after_release)
        ),
    );
}
fn growth_after_release(arena: &BaseAllocator) -> bool {
    arena.reset(true).unwrap();
    let reset = allocate_selected(arena, 2048, true);
    let next = allocate_selected(arena, 10240, true);
    next.as_ptr() == reset.as_ptr().wrapping_add(2048)
}
#[test]
fn protected_operations_keep_live_typed_leases_and_reject_overflow() {
    let arena = BaseAllocator::incremental(2048);
    let owner = arena.allocate_value(17_u64);
    {
        let mut extension: IncAllocatorExtension<'_> = arena.incremental_extension().unwrap();
        extension.increase_block_size();
        assert!(extension.allocate_slow(usize::MAX).is_none());
        assert!(extension.allocate_slow(16).is_some());
    }
    assert_eq!(*owner, 17);
    assert!(arena.reset(false).is_err());
    assert!(arena.reset(true).is_err());
    drop(owner);
    arena.reset(true).unwrap();
}
#[test]
fn incremental_extension_rejects_other_concrete_policies() {
    for arena in [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::accumulating(64).unwrap(),
        BaseAllocator::aligned(16),
    ] {
        let failure = match arena.incremental_extension() {
            Ok(_) => panic!("non-incremental policy accepted protected incremental operations"),
            Err(failure) => failure,
        };
        assert_eq!(failure.exception_type(), "Standard_TypeMismatch");
        assert!(
            failure
                .message()
                .contains("expected NCollection_IncAllocator")
        );
        assert!(failure.message().contains("extension:"));
    }
}
