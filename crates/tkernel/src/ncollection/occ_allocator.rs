// Copyright (c) 2023 OPEN CASCADE SAS
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

//! Typed allocator rebinding preserves a shared allocation resource.
use super::{AllocatorList, BaseAllocator};
use std::marker::PhantomData;
/// Rust counterparts of allocator typedefs; `<OccAllocator<i32> as AllocatorTypes>::Value`.
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
}
/// Typed view of an allocator; `OccAllocator::<u8>::default().rebind::<i32>()`.
pub struct OccAllocator<T> {
    resource: BaseAllocator,
    element: PhantomData<fn() -> T>,
}
impl<T> AllocatorTypes for OccAllocator<T> {
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
impl<T> OccAllocator<T> {
    /// Binds an element type to a resource; `OccAllocator::<i32>::new(arena)`.
    pub fn new(resource: BaseAllocator) -> Self {
        Self {
            resource,
            element: PhantomData,
        }
    }
    /// Changes element type while retaining resource identity; `allocator.rebind::<f64>()`.
    pub fn rebind<U>(&self) -> OccAllocator<U> {
        OccAllocator::new(self.resource.clone())
    }
    /// Creates contiguous storage using this resource; `allocator.vector().push(value)`.
    pub fn vector(&self) -> AllocatorVector<T> {
        AllocatorVector {
            elements: allocator_api2::vec::Vec::new_in(self.resource.clone()),
        }
    }
    /// Creates linked storage using this resource for each node; `allocator.list().push_back(value)`.
    pub fn list(&self) -> AllocatorList<T> {
        AllocatorList::with_allocator(self.resource.clone())
    }
}
impl<T> OccAllocator<T> {
    // The bound resource, for containers that take an `allocator_type` and keep its
    // `Allocator()` handle (NCollection_DynamicArray's allocator_type constructor).
    pub(crate) fn bound_resource(&self) -> &BaseAllocator {
        &self.resource
    }
}
impl<T> Default for OccAllocator<T> {
    fn default() -> Self {
        Self::new(BaseAllocator::common())
    }
}
impl<T> Clone for OccAllocator<T> {
    fn clone(&self) -> Self {
        self.rebind()
    }
}
impl<T, U> PartialEq<OccAllocator<U>> for OccAllocator<T> {
    fn eq(&self, other: &OccAllocator<U>) -> bool {
        self.resource == other.resource
    }
}
impl<T> Eq for OccAllocator<T> {}
impl<T> std::fmt::Debug for OccAllocator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OccAllocator").field(&self.resource).finish()
    }
}
/// Contiguous allocator-owned elements; `allocator.vector()`.
pub struct AllocatorVector<T> {
    elements: allocator_api2::vec::Vec<T, BaseAllocator>,
}
impl<T> AllocatorVector<T> {
    /// Appends an element, growing through the selected allocator; `vector.push(42)`.
    pub fn push(&mut self, value: T) {
        self.elements.push(value);
    }
}
impl<T: Clone> AllocatorVector<T> {
    /// Extends by cloning a fill value; `vector.resize(10, 0)`.
    pub fn resize(&mut self, length: usize, value: T) {
        self.elements.resize(length, value);
    }
}
impl<T> std::ops::Deref for AllocatorVector<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.elements
    }
}
impl<T> std::ops::DerefMut for AllocatorVector<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.elements
    }
}
