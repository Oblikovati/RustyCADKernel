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

//! Runtime-typed allocator objects delegate storage to the existing allocator implementation.
use super::BaseAllocator;
use crate::standard::StandardTransient;
/// Polymorphic allocator object; `AllocatorObject::default().resource().allocate(16)`.
#[repr(C)]
#[derive(Default)]
pub struct AllocatorObject {
    parent: StandardTransient,
    resource: BaseAllocator,
}
impl AllocatorObject {
    /// Borrow the real allocation owner; `object.resource().allocate_value(42)`.
    pub fn resource(&self) -> &BaseAllocator {
        &self.resource
    }
}
crate::impl_transient!(
    AllocatorObject,
    "NCollection_BaseAllocator",
    StandardTransient,
    parent
);
/// Incremental arena object supporting base-handle casts; `IncrementalAllocatorObject::default()`.
#[repr(C)]
pub struct IncrementalAllocatorObject {
    parent: AllocatorObject,
}
impl Default for IncrementalAllocatorObject {
    fn default() -> Self {
        Self {
            parent: AllocatorObject {
                parent: StandardTransient::default(),
                resource: BaseAllocator::incremental(24_600),
            },
        }
    }
}
impl IncrementalAllocatorObject {
    /// Borrow the backing incremental arena; `object.resource().reset(false)`.
    pub fn resource(&self) -> &BaseAllocator {
        self.parent.resource()
    }
}
crate::impl_transient!(
    IncrementalAllocatorObject,
    "NCollection_IncAllocator",
    AllocatorObject,
    parent
);
/// Global-heap allocator object; `HeapAllocatorObject::default()`.
#[repr(C)]
#[derive(Default)]
pub struct HeapAllocatorObject {
    parent: AllocatorObject,
}
impl HeapAllocatorObject {
    /// Borrow the backing heap allocator; `object.resource().allocate(16)`.
    pub fn resource(&self) -> &BaseAllocator {
        self.parent.resource()
    }
}
crate::impl_transient!(
    HeapAllocatorObject,
    "NCollection_HeapAllocator",
    AllocatorObject,
    parent
);
