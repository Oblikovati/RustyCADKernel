//! Native handle-retaining moves, typed placement operations, and raw free/reallocate policy.
#![allow(unsafe_code)]
use rustycad_tkernel::ncollection::{
    AllocatorTypes, BaseAllocator, OccAllocator, StandardAllocator,
};
use std::cell::Cell;
use std::ptr::NonNull;
use std::rc::Rc;

#[test]
fn moved_handle_constructors_leave_native_source_handle_present() {
    let arena = BaseAllocator::incremental(256);
    let mut handle = Some(arena.clone());
    let copied = OccAllocator::<i32>::with_allocator_ref(&handle);
    let moved = OccAllocator::<i32>::with_allocator_move_from(&mut handle);
    assert_eq!(copied, moved);
    assert_eq!(handle.as_ref(), Some(&arena));
    let mut source = moved;
    let constructed = OccAllocator::move_from(&mut source);
    assert_eq!(source, constructed);
    assert_eq!(source.allocator(), Some(&arena));
}
#[test]
fn returning_assignments_preserve_destination_identity_and_move_source() {
    let arena = BaseAllocator::incremental(256);
    let source = OccAllocator::<u8>::new(arena.clone());
    let mut target = OccAllocator::<i32>::default();
    let address = &mut target as *mut _;
    assert_eq!(target.assign_returning(&source) as *mut _, address);
    assert_eq!(target.allocator(), Some(&arena));
    assert_eq!(target.assign_self() as *mut _, address);
    let mut donor = OccAllocator::<i32>::default();
    assert_eq!(target.move_assign_returning(&mut donor) as *mut _, address);
    assert_eq!(target, donor);
    assert!(donor.allocator().is_none());
    assert!(!std::hint::black_box(
        OccAllocator::<i32>::PROPAGATE_ON_CONTAINER_MOVE_ASSIGNMENT
    ));
}
#[test]
fn move_assignment_keeps_nonnull_donor_handle() {
    let arena = BaseAllocator::incremental(256);
    let mut source = OccAllocator::<i32>::new(arena.clone());
    let mut destination = OccAllocator::default();
    destination.move_assign_returning(&mut source);
    assert_eq!(destination, source);
    assert_eq!(source.allocator(), Some(&arena));
}
#[test]
fn stateless_nullable_constructor_and_assignment_preserve_identity() {
    let arena = BaseAllocator::incremental(256);
    let mut target = StandardAllocator::<i32>::from_optional_base(None);
    let source = StandardAllocator::<u8>::from_optional_base(Some(&arena));
    assert_eq!(target, source);
    let address = &mut target as *mut _;
    assert_eq!(target.assign_returning(&source) as *mut _, address);
    assert_eq!(target.assign_self() as *mut _, address);
    target.assign(&source);
}
#[test]
fn occ_hint_and_cross_type_free_use_the_raw_allocator_policy() {
    let allocator = OccAllocator::<i32>::default();
    let hint = 17_u8;
    let storage = allocator
        .allocate_with_hint(1, std::ptr::from_ref(&hint).cast())
        .unwrap();
    // SAFETY: one properly aligned raw i32 slot is live, then released exactly once.
    unsafe {
        allocator.construct(storage, 42);
        assert_eq!(storage.read(), 42);
        allocator.destroy(storage);
        allocator
            .deallocate_ptr(storage.as_ptr().cast::<u8>(), 999)
            .unwrap();
        allocator
            .deallocate_ptr::<i32>(std::ptr::null_mut(), 0)
            .unwrap();
    }
}
#[test]
fn nullable_free_preserves_accumulating_allocator_error_policy() {
    let allocator = OccAllocator::<i32>::new(BaseAllocator::accumulating(256).unwrap());
    // SAFETY: null is a valid input to the native nullable free entry point.
    let failure = unsafe { allocator.deallocate_ptr::<u8>(std::ptr::null_mut(), 0) }.unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
}
#[test]
fn standard_hint_nullable_free_and_reallocate_null_preserve_raw_layout() {
    let allocator = StandardAllocator::<i32>::new();
    let hint = 17_u8;
    let storage = allocator
        .allocate_with_hint(1, std::ptr::from_ref(&hint).cast())
        .unwrap();
    // SAFETY: storage belongs to this raw policy; null has the native allocation meaning.
    unsafe {
        allocator.deallocate_ptr(storage.as_ptr(), 99).unwrap();
        allocator.deallocate_ptr(std::ptr::null_mut(), 0).unwrap();
        let allocated = allocator.reallocate_ptr(std::ptr::null_mut(), 1).unwrap();
        allocated.write(42);
        let grown = allocator.reallocate_ptr(allocated.as_ptr(), 8).unwrap();
        assert_eq!(grown.read(), 42);
        allocator.deallocate_ptr(grown.as_ptr(), 8).unwrap();
    }
}
#[test]
fn failed_nonzero_reallocate_leaves_original_block_live() {
    let allocator = StandardAllocator::<u64>::new();
    let storage = allocator.allocate(1).unwrap();
    // SAFETY: an impossible nonzero layout refuses the request before touching the live block.
    unsafe {
        storage.write(42);
        assert!(
            allocator
                .reallocate(storage, usize::MAX / std::mem::size_of::<u64>())
                .is_none()
        );
        assert_eq!(storage.read(), 42);
        allocator.deallocate(storage, 1).unwrap();
    }
}
#[test]
fn null_zero_reallocate_is_distinct_from_live_zero_reallocate() {
    let allocator = StandardAllocator::<i32>::new();
    // SAFETY: a null input allocates, and its returned zero-byte raw block is freed once.
    unsafe {
        let empty = allocator.reallocate_ptr(std::ptr::null_mut(), 0).unwrap();
        allocator.deallocate(empty, 0).unwrap();
    }
}
#[test]
#[cfg(any(all(target_os = "linux", target_env = "gnu"), target_env = "msvc"))]
fn native_zero_reallocate_consumes_live_storage() {
    let allocator = StandardAllocator::<i32>::new();
    let storage = allocator.allocate(1).unwrap();
    // SAFETY: the configured NATIVE CRT frees this block on zero-size failure; never reuse it.
    assert!(unsafe { allocator.reallocate(storage, 0) }.is_none());
}

struct Tracked {
    number: i32,
    copies: Rc<Cell<i32>>,
    drops: Rc<Cell<i32>>,
}
impl Clone for Tracked {
    fn clone(&self) -> Self {
        self.copies.set(self.copies.get() + 1);
        Self {
            number: self.number,
            copies: Rc::clone(&self.copies),
            drops: Rc::clone(&self.drops),
        }
    }
}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
#[test]
fn placement_copy_and_closure_construction_run_matching_destructors() {
    let copies = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let source = Tracked {
        number: 42,
        copies: Rc::clone(&copies),
        drops: Rc::clone(&drops),
    };
    let allocator = StandardAllocator::<Tracked>::new();
    let storage = allocator.allocate(1).unwrap();
    // SAFETY: each construction initializes the allocated slot after its previous value is destroyed.
    unsafe {
        allocator.construct_ref(storage, &source);
        assert_eq!((copies.get(), storage.as_ref().number), (1, 42));
        allocator.destroy(storage);
        allocator.construct_with(storage, || source.clone());
        assert_eq!(copies.get(), 2);
        allocator.destroy(storage);
        allocator.deallocate(storage, 1).unwrap();
    }
    assert_eq!(drops.get(), 2);
}
#[test]
fn failed_constructor_does_not_initialize_raw_slot() {
    let allocator = OccAllocator::<i32>::default();
    let storage: NonNull<i32> = allocator.allocate(1).unwrap();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: storage is writable and uninitialized; the closure panics before any write.
        unsafe {
            allocator.construct_with(storage, || panic!("injected constructor failure"));
        }
    }));
    assert!(failed.is_err());
    // SAFETY: construction failed, so only the raw allocation needs to be released.
    unsafe {
        allocator.deallocate(storage, 1).unwrap();
    }
}
