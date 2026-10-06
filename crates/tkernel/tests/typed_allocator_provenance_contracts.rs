//! Reborrowed typed payloads must be freed without accessing adjacent header bytes.
use rustycad_tkernel::ncollection::BaseAllocator;

#[test]
fn boxes_with_small_payloads_and_reborrows_free_using_the_original_layout() {
    for allocator in [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::aligned(64),
    ] {
        let mut owned = allocator.allocate_value([1_u8, 2, 3]);
        owned[1] = 8;
        assert_eq!(*owned, [1, 8, 3]);
        drop(owned);
    }
}

#[repr(align(128))]
struct AlignedPayload([u8; 3]);

#[test]
fn typed_payload_alignment_overrides_smaller_allocator_alignment() {
    let allocator = BaseAllocator::aligned(32);
    let mut owned = allocator.allocate_value(AlignedPayload([1, 2, 3]));
    assert_eq!(std::ptr::from_ref(&*owned).addr() % 128, 0);
    owned.0[2] = 9;
    assert_eq!(owned.0, [1, 2, 9]);
}

#[test]
fn typed_arrays_and_zero_sized_values_follow_the_same_layout_contract() {
    for allocator in [
        BaseAllocator::common(),
        BaseAllocator::global_heap(),
        BaseAllocator::aligned(64),
    ] {
        let mut array = allocator.allocate_array(3, |index| [index as u8; 3]);
        array.as_mut_slice()[1][0] = 8;
        assert_eq!(array.as_slice()[1], [8, 1, 1]);
        drop(array);
        drop(allocator.allocate_value(()));
    }
}
