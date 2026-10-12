//! A custom memory provider can outlive the genuine allocator without retaining its native object.
#![allow(unsafe_code)]
use rustycad_tkernel::{
    ncollection::{
        AllocatorObject, AllocatorPhysicalStorage, AllocatorSource, AllocatorStorageLease,
        BaseAllocator,
    },
    standard::{Failure, Handle, SharedState},
};
use std::{
    alloc::Layout,
    collections::HashMap,
    ptr::NonNull,
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
};
#[derive(Default)]
struct IndependentHeapBacking {
    ranges: SharedState<HashMap<usize, (AtomicPtr<u8>, Layout)>>,
    released: AtomicUsize,
}
impl IndependentHeapBacking {
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        let layout = Layout::from_size_align(size.max(1), 32).ok()?;
        // SAFETY: nonzero validated layout allocates an independent physical heap range.
        let pointer = NonNull::new(unsafe { std::alloc::alloc(layout) })?;
        self.ranges.write().insert(
            pointer.as_ptr().addr(),
            (AtomicPtr::new(pointer.as_ptr()), layout),
        );
        Some(pointer)
    }
}
// SAFETY: metadata retains original pointers/layouts independently of the native allocator object.
unsafe impl AllocatorPhysicalStorage for IndependentHeapBacking {
    unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        if pointer.is_null() {
            return Ok(());
        }
        let (original, layout) = self.ranges.write().remove(&pointer.addr()).unwrap();
        // SAFETY: original full allocation pointer/layout survives all narrowed typed reborrows.
        unsafe { std::alloc::dealloc(original.load(Ordering::Relaxed), layout) };
        self.released.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
struct IndependentHeapAllocator {
    parent: AllocatorObject,
    backing: Arc<IndependentHeapBacking>,
    retired: Arc<AtomicUsize>,
}
rustycad_tkernel::impl_transient!(IndependentHeapAllocator, "Martensite_IndependentHeapAllocator",
    AllocatorObject, parent, interfaces: [dyn AllocatorSource]);
// SAFETY: the actual backing lease owns every returned range beyond this object's retirement.
unsafe impl AllocatorSource for IndependentHeapAllocator {
    fn allocator_object_base(&self) -> &AllocatorObject {
        // Native hierarchy and ownership remain attached to this exact embedded base.
        &self.parent
    }
    fn physical_storage(&self) -> AllocatorStorageLease {
        // Only independent memory is retained, not an Arc to this derived allocator object.
        AllocatorStorageLease::new(self.backing.clone())
    }
    fn allocate(&self, size: usize) -> Option<NonNull<u8>> {
        // The external provider is injected through this genuine allocator's construction.
        self.backing.allocate(size)
    }
    fn allocate_optimal(&self, size: usize) -> Option<NonNull<u8>> {
        // Both real virtual entrypoints use the same injected physical provider.
        self.backing.allocate(size)
    }
    unsafe fn free(&self, pointer: *mut u8) -> Result<(), Failure> {
        // SAFETY: the original live range belongs to this same independent provider.
        unsafe { self.backing.free(pointer) }
    }
}
impl Drop for IndependentHeapAllocator {
    fn drop(&mut self) {
        // Native object lifetime is visible independently of physical allocation lifetime.
        self.retired.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn injected_backing_preserves_payload_after_real_allocator_object_drop() {
    let backing = Arc::new(IndependentHeapBacking::default());
    let retired = Arc::new(AtomicUsize::default());
    let owner = independent_heap_source(&backing, &retired);
    let resource = BaseAllocator::from_handle(&owner);
    let payload = resource.allocate_value(53_u64);
    let lease = resource.storage_lease();
    assert_eq!(owner.ref_count(), 2);
    drop((owner, resource));
    assert_eq!(retired.load(Ordering::SeqCst), 1);
    assert_eq!(*payload, 53);
    let repeated = lease.storage_lease();
    assert_eq!(repeated.ref_count(), 0);
    drop((lease, repeated));
    drop(payload);
    assert_eq!(backing.released.load(Ordering::SeqCst), 1);
    assert!(backing.ranges.read().is_empty());
}

fn independent_heap_source(backing: &Arc<IndependentHeapBacking>, retired: &Arc<AtomicUsize>)
    -> Handle<IndependentHeapAllocator> {
    Handle::new(IndependentHeapAllocator { parent: AllocatorObject::default(),
        backing: backing.clone(), retired: retired.clone() })
}
