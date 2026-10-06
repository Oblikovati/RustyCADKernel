//! Standard-allocator typedefs and element helpers shared by the two typed allocators, and
//! the stateless `NCollection_Allocator<T>`.
use super::super::BaseAllocator;
use crate::standard::Failure;
use allocator_api2::alloc::{AllocError, Allocator, Layout};
use std::{marker::PhantomData, ptr::NonNull};

/// Rust counterparts of allocator typedefs and of the element members every C++ standard
/// allocator repeats (`address`, `construct`, `destroy`); `<OccAllocator<i32> as AllocatorTypes>::Value`.
pub trait AllocatorTypes {
    /// Allocated element.
    type Value;
    /// Mutable address.
    type Pointer;
    /// Constant address.
    type ConstPointer;
    /// Allocation count.
    type Size;
    /// Address difference.
    type Difference;
    /// Exclusive element borrow.
    type Reference<'a>
    where
        Self::Value: 'a;
    /// Shared element borrow.
    type ConstReference<'a>
    where
        Self::Value: 'a;

    /// The address of an element (`address(const_reference)`); `alloc.address(&item)`.
    fn address(&self, item: &Self::Value) -> *const Self::Value {
        item
    }
    /// The mutable address of an element (`address(reference)`); `alloc.address_mut(&mut item)`.
    fn address_mut(&self, item: &mut Self::Value) -> *mut Self::Value {
        item
    }
    /// Constructs a value in place (`construct`, placement new); `unsafe { alloc.construct(p, 5) }`.
    ///
    /// # Safety
    /// `pointer` must be valid for writes and aligned for `U`; any previous value is not dropped.
    unsafe fn construct<U>(&self, pointer: NonNull<U>, value: U) {
        // SAFETY: forwarded from this function's contract.
        unsafe { pointer.write(value) }
    }
    /// Constructs from forwarded constructor arguments represented by a closure; `unsafe { alloc.construct_with(p, || 42) }`.
    ///
    /// # Safety
    /// `pointer` must be writable and aligned for `U`, with no live value requiring a drop.
    unsafe fn construct_with<U>(&self, pointer: NonNull<U>, construct: impl FnOnce() -> U) {
        // SAFETY: pointer validity is the caller's contract; a failed constructor writes nothing.
        unsafe { self.construct(pointer, construct()) }
    }
    /// Copy-constructs from a borrowed object; `unsafe { alloc.construct_ref(p, &source) }`.
    ///
    /// # Safety
    /// `pointer` must be writable, aligned, and not overlap `source` or any live value.
    unsafe fn construct_ref<U: Clone>(&self, pointer: NonNull<U>, source: &U) {
        // SAFETY: forwarded from this method's contract.
        unsafe { self.construct_with(pointer, || source.clone()) }
    }
    /// Runs the destructor in place (`destroy`); `unsafe { alloc.destroy(p) }`.
    ///
    /// # Safety
    /// `pointer` must hold a valid, initialized `U` that is not used afterwards.
    unsafe fn destroy<U>(&self, pointer: NonNull<U>) {
        // SAFETY: forwarded from this function's contract.
        unsafe { pointer.drop_in_place() }
    }
}

/// The stateless OCCT standard allocator (`NCollection_Allocator<T>`): every instance uses
/// `Standard::AllocateOptimal`/`Free`/`Reallocate` and all instances compare equal.
///
/// Example: `StandardAllocator::<i32>::new() == StandardAllocator::<f64>::new()`.
pub struct StandardAllocator<T> {
    element: PhantomData<fn() -> T>,
}

impl<T> StandardAllocator<T> {
    /// A new allocator (`NCollection_Allocator()`); `StandardAllocator::<u8>::new()`.
    pub fn new() -> Self {
        Self {
            element: PhantomData,
        }
    }
    /// The constructor taking an allocator handle, which upstream ignores.
    ///
    /// Example: `StandardAllocator::<u8>::from_base(&BaseAllocator::incremental(0))`.
    pub fn from_base(_resource: &BaseAllocator) -> Self {
        Self::new()
    }
    /// The same allocator for another element type (converting constructor); `a.rebind::<u16>()`.
    pub fn rebind<U>(&self) -> StandardAllocator<U> {
        StandardAllocator::new()
    }
    /// Assignment from another element type, a no-op upstream; `a.assign(&b)`.
    pub fn assign<U>(&mut self, other: &StandardAllocator<U>) {
        self.assign_returning(other);
    }

    /// Raw storage for `count` elements (`allocate`); `None` where upstream would get NULL,
    /// including a byte count that overflows. Example: `alloc.allocate(8)`.
    pub fn allocate(&self, count: usize) -> Option<NonNull<T>> {
        let bytes = count.checked_mul(size_of::<T>())?;
        Some(BaseAllocator::common().allocate_optimal_raw(bytes)?.cast())
    }
    /// Releases storage from `allocate` (`deallocate`).
    ///
    /// # Safety
    /// `pointer` must come from `allocate`/`reallocate` of a standard allocator, not freed yet.
    pub unsafe fn deallocate(&self, pointer: NonNull<T>, _count: usize) -> Result<(), Failure> {
        // SAFETY: forwarded from this function's contract.
        unsafe { BaseAllocator::common().free_raw(pointer.cast()) }
    }
    /// Resizes storage to `count` elements (`reallocate`, `Standard::Reallocate`), keeping the
    /// prefix. Nonzero-size failure leaves the original block valid. For the configured NATIVE
    /// backend on GNU/Linux and MSVC, zero bytes frees the block and returns `None`, matching
    /// the native Standard_OutOfMemory quirk; the original pointer must not then be reused.
    ///
    /// # Safety
    /// `pointer` must come from `allocate`/`reallocate` of a standard allocator, not freed yet.
    pub unsafe fn reallocate(&self, pointer: NonNull<T>, count: usize) -> Option<NonNull<T>> {
        let bytes = count.checked_mul(size_of::<T>())?;
        if bytes == 0 && crate::standard::memory::REALLOC_ZERO_FREES {
            // SAFETY: native realloc(p, 0) consumes this live raw block on the recorded CRTs.
            unsafe { self.deallocate(pointer, count) }.ok()?;
            return None;
        }
        // SAFETY: forwarded from this function's contract.
        Some(unsafe { BaseAllocator::standard_reallocate(pointer.cast(), bytes) }?.cast())
    }
}

impl<T> AllocatorTypes for StandardAllocator<T> {
    type Value = T;
    type Pointer = *mut T;
    type ConstPointer = *const T;
    type Size = usize;
    type Difference = isize;
    type Reference<'a>
        = &'a mut T
    where
        T: 'a;
    type ConstReference<'a>
        = &'a T
    where
        T: 'a;
}

impl<T> Default for StandardAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Clone for StandardAllocator<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for StandardAllocator<T> {}
impl<T, U> PartialEq<StandardAllocator<U>> for StandardAllocator<T> {
    /// Every standard allocator is interchangeable (`operator==` always true).
    fn eq(&self, _other: &StandardAllocator<U>) -> bool {
        true
    }
}
impl<T> Eq for StandardAllocator<T> {}
impl<T> std::fmt::Debug for StandardAllocator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StandardAllocator")
    }
}

// SAFETY: forwards to the common allocator, whose blocks satisfy the Allocator contract and
// can be freed through any instance, as every standard allocator shares that one policy.
unsafe impl<T> Allocator for StandardAllocator<T> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Allocator::allocate(&BaseAllocator::common(), layout)
    }
    unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        // SAFETY: forwarded from the Allocator caller's contract.
        unsafe { Allocator::deallocate(&BaseAllocator::common(), pointer, layout) }
    }
}
