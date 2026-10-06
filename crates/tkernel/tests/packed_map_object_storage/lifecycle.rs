#![allow(unsafe_code)]
use super::{expected, node_trace, release, reserve};
use rustycad_tkernel::ncollection::{AllocatorTypes, PackedInteger, PackedMap, StandardAllocator};
use std::ptr::NonNull;
fn construct_maps<T: PackedInteger>(pointer: NonNull<PackedMap<T>>, count: usize) {
    for index in 0..count {
        // SAFETY: the byte owner reserves count aligned, nonoverlapping map slots; none is live yet.
        let slot = unsafe { pointer.add(index) };
        let returned =
            PackedMap::<T>::placement_storage(size_of::<PackedMap<T>>(), slot.as_ptr().cast());
        assert_eq!(returned, slot.as_ptr().cast());
        unsafe {
            StandardAllocator::<PackedMap<T>>::new().construct(slot, PackedMap::new(700 + index))
        };
    }
}
fn populate<T: PackedInteger>(pointer: NonNull<PackedMap<T>>, count: usize) {
    for index in 0..count {
        // SAFETY: construct_maps initialized each live slot; byte storage stays exclusively owned.
        let map = unsafe { &mut *pointer.as_ptr().add(index) };
        for key in [0, 41, u64::MAX] {
            map.add(T::from_bits(key));
        }
    }
}
fn destroy_maps<T: PackedInteger>(pointer: NonNull<PackedMap<T>>, count: usize) {
    for index in (0..count).rev() {
        // SAFETY: each slot is destroyed once in reverse construction order, before freeing bytes.
        unsafe { StandardAllocator::<PackedMap<T>>::new().destroy(pointer.add(index)) };
    }
}
fn state<T: PackedInteger>(
    pointer: NonNull<PackedMap<T>>,
    count: usize,
    allocations: usize,
) -> String {
    let mut result = allocations.to_string();
    for index in 0..count {
        // SAFETY: slots remain initialized until the following measured destroy_maps call.
        let map = unsafe { &*pointer.as_ptr().add(index) };
        result += &format!("|{}|{}", map.bucket_count(), map.len());
        for value in map {
            result += &format!("|{}", value.bits());
        }
    }
    result
}
pub fn lifetimes<T: PackedInteger>(name: &str) {
    for count in [1, 3] {
        let storage = reserve::<T>(count != 1, count * size_of::<PackedMap<T>>()).unwrap();
        let pointer = NonNull::new(storage.as_ptr().cast_mut().cast::<PackedMap<T>>()).unwrap();
        construct_maps(pointer, count);
        let (_, allocated) = node_trace::measure::<T, _>(|| populate(pointer, count));
        let mut actual = state(pointer, count, allocated.allocations);
        let (_, destroyed) = node_trace::measure::<T, _>(|| destroy_maps(pointer, count));
        actual += &format!("|{}", destroyed.releases);
        release::<T>(count != 1, Some(storage));
        expected(&format!("{name}:lifetime:{count}"), &actual);
    }
}
pub fn failed_placement<T: PackedInteger>(name: &str) {
    let mut storage = reserve::<T>(false, size_of::<PackedMap<T>>() + 1).unwrap();
    storage.as_mut_slice()[size_of::<PackedMap<T>>()] = 165;
    let pointer = NonNull::new(storage.as_ptr().cast_mut().cast::<PackedMap<T>>()).unwrap();
    let (result, counts) = node_trace::measure::<T, _>(|| failed_constructor(pointer));
    PackedMap::<T>::release_placement_storage(pointer.as_ptr().cast(), pointer.as_ptr().cast());
    let retained = storage.as_slice()[size_of::<PackedMap<T>>()] == 165;
    let actual = format!(
        "{}|{}|{}|{}",
        u8::from(result),
        counts.allocations,
        counts.releases,
        u8::from(retained)
    );
    release::<T>(false, Some(storage));
    expected(&format!("{name}:failed_placement"), &actual);
}
fn failed_constructor<T: PackedInteger>(pointer: NonNull<PackedMap<T>>) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: live aligned storage remains uninitialized when its constructor unwinds.
        unsafe {
            StandardAllocator::<PackedMap<T>>::new()
                .construct_with(pointer, fail_after_one_node::<T>)
        };
    }))
    .is_err()
}
fn fail_after_one_node<T: PackedInteger>() -> PackedMap<T> {
    let mut map = PackedMap::default();
    map.add(T::from_bits(41));
    // A byte payload cannot be mistaken for a 16/24-byte node by the allocation observer.
    std::panic::panic_any(17_u8);
}
