// Created on: 2002-04-12
// Created by: Alexander KARTOMIN (akm)
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

// Purpose:     Basic class for memory allocation wizards.
//              Defines  the  interface  for devising  different  allocators
//              firstly to be used  by collections of NCollection, though it
//              it is not  deferred. It allocates/frees  the memory  through
//              Standard procedures, thus it is  unnecessary (and  sometimes
//              injurious) to have  more than one such  allocator.  To avoid
//              creation  of multiple  objects the  constructors  were  maid
//              inaccessible.  To  create the  BaseAllocator use  the method
//              CommonBaseAllocator.
//              Note that this object is managed by Handle.

//! Allocator ownership shared by all allocations and allocator-aware collections.
// Unsafe is confined to implementing allocator-api2's contract and forwarding frees.
// Arena payloads use UnsafeCell; no broad mutable borrow may invalidate live slots.
use crate::standard::Failure;
use allocator_api2::{
    alloc::{AllocError, Allocator, Global, Layout},
    boxed::Box as AllocBox,
};
use std::{
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
};
mod arena;
use arena::Arena;

/// A shared allocator identity and backing storage; `BaseAllocator::common().allocate(16)`.
#[derive(Clone)]
pub struct BaseAllocator {
    inner: Arc<AllocatorKind>,
}
enum AllocatorKind {
    Global,
    Incremental(Mutex<Arena>),
}
/// Owned bytes freed through the creating allocator; `allocation.as_slice()`.
pub struct AllocatedBytes {
    storage: allocator_api2::vec::Vec<u8, BaseAllocator>,
}
/// A typed allocation which retains its allocator; `allocator.allocate_value(42)`.
pub struct AllocatedBox<T> {
    storage: AllocBox<T, BaseAllocator>,
}
/// Contiguous typed storage retaining the allocation owner; `array.as_slice()`.
pub struct AllocatedArray<T> {
    storage: allocator_api2::vec::Vec<T, BaseAllocator>,
}
impl<T> AllocatedArray<T> {
    /// Borrows constructed elements; `array.as_slice()[0]`.
    pub fn as_slice(&self) -> &[T] {
        &self.storage
    }
    /// Borrows elements for mutation; `array.as_mut_slice().reverse()`.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage
    }
}
impl BaseAllocator {
    /// Constructs contiguous elements with this allocator; `allocator.allocate_array(5, |i| i)`.
    pub fn allocate_array<T>(
        &self,
        count: usize,
        constructor: impl FnMut(usize) -> T,
    ) -> AllocatedArray<T> {
        let mut storage = allocator_api2::vec::Vec::with_capacity_in(count, self.clone());
        storage.extend((0..count).map(constructor));
        AllocatedArray { storage }
    }

    /// Returns the shared process allocator; `BaseAllocator::common() == BaseAllocator::common()`.
    pub fn common() -> Self {
        static COMMON: OnceLock<BaseAllocator> = OnceLock::new();
        COMMON
            .get_or_init(|| Self {
                inner: Arc::new(AllocatorKind::Global),
            })
            .clone()
    }
    /// Creates a distinct incremental arena; `BaseAllocator::incremental(12*1024)`.
    pub fn incremental(block_size: usize) -> Self {
        Self {
            inner: Arc::new(AllocatorKind::Incremental(Mutex::new(Arena::new(
                block_size,
            )))),
        }
    }
    /// Allocates initialized byte storage; `allocator.allocate(16).as_slice().len() == 16`.
    /// Zero initialization defines bytes which upstream leaves unspecified.
    pub fn allocate(&self, size: usize) -> AllocatedBytes {
        let mut storage = allocator_api2::vec::Vec::with_capacity_in(size, self.clone());
        storage.resize(size, 0);
        AllocatedBytes { storage }
    }
    /// Allocates correctly aligned typed storage; `*allocator.allocate_value(42) == 42`.
    pub fn allocate_value<T>(&self, value: T) -> AllocatedBox<T> {
        AllocatedBox {
            storage: AllocBox::new_in(value, self.clone()),
        }
    }
    /// Rewinds an arena only after live allocations are freed; `allocator.reset(false)?` retains chunks.
    pub fn reset(&self, release: bool) -> Result<(), Failure> {
        match &*self.inner {
            AllocatorKind::Global => Ok(()),
            AllocatorKind::Incremental(arena) => arena
                .lock()
                .expect("allocator mutex poisoned")
                .reset(release),
        }
    }
}
impl Default for BaseAllocator {
    fn default() -> Self {
        Self::common()
    }
}
impl PartialEq for BaseAllocator {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for BaseAllocator {}
impl std::fmt::Debug for BaseAllocator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BaseAllocator")
            .field(&Arc::as_ptr(&self.inner))
            .finish()
    }
}
impl AllocatedBytes {
    /// Borrows allocated bytes; `bytes.as_slice().len()`.
    pub fn as_slice(&self) -> &[u8] {
        &self.storage
    }
    /// Mutates allocated bytes; `bytes.as_mut_slice().fill(0xAB)`.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.storage
    }
}
impl<T> std::ops::Deref for AllocatedBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.storage
    }
}
impl<T> std::ops::DerefMut for AllocatedBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.storage
    }
}
// SAFETY: Global delegates its layout/ownership contract to allocator-api2.
// Incremental allocations are disjoint, aligned regions of stable boxed chunks.
// Every allocator clone keeps those chunks alive. Reset rejects live allocations;
// deallocate only releases the live count, retaining each region until reset/drop.
unsafe impl Allocator for BaseAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        match &*self.inner {
            AllocatorKind::Global => Global.allocate(layout),
            AllocatorKind::Incremental(arena) => arena
                .lock()
                .expect("allocator mutex poisoned")
                .allocate(layout),
        }
    }
    unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout) {
        match &*self.inner {
            // SAFETY: the Allocator caller supplies an owned block from this same
            // allocator and its original layout, as required by the trait.
            AllocatorKind::Global => unsafe {
                Global.deallocate(pointer, layout);
            },
            AllocatorKind::Incremental(arena) => {
                arena.lock().expect("allocator mutex poisoned").deallocate()
            }
        }
    }
}
