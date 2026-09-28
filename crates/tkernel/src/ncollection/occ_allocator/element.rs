// Copyright (c) 2002-2023 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

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
    pub fn assign<U>(&mut self, _other: &StandardAllocator<U>) {}

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
    /// prefix. `None` (upstream: Standard_OutOfMemory) leaves the original block valid.
    ///
    /// # Safety
    /// `pointer` must come from `allocate`/`reallocate` of a standard allocator, not freed yet.
    pub unsafe fn reallocate(&self, pointer: NonNull<T>, count: usize) -> Option<NonNull<T>> {
        let bytes = count.checked_mul(size_of::<T>())?;
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
