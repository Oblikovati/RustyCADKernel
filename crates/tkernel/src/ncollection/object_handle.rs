// Created on: 2009-01-30
// Created by: Andrey BETENEV (abv)
// Copyright (c) 2009-2014 OPEN CASCADE SAS
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

//! Reference-counted handles to objects that are not transients (`NCollection_Handle<T>`).
//!
//! Upstream wraps the object in a private `Ptr : Standard_Transient` adaptor and derives from
//! `handle<Standard_Transient>`; the adaptor has no RTTI of its own, so its dynamic type is
//! `Standard_Transient`. Rust keeps the same adaptor inside a standard `Handle`.
use crate::standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient, Transient};
use std::any::{Any, TypeId};

/// The native `Ptr` adaptor: a transient owning one object.
struct HandleAdaptor<T> {
    parent: StandardTransient,
    object: T,
}

impl<T: Send + Sync + 'static> RuntimeTyped for HandleAdaptor<T> {
    // No RTTI of its own upstream: it reports Standard_Transient's name and descriptor.
    const TYPE_NAME: &'static str = StandardTransient::TYPE_NAME;
    fn static_type() -> &'static RuntimeType {
        StandardTransient::static_type() // `Ptr` declares no RTTI of its own
    }
    fn project_from(object: &dyn crate::standard::Transient) -> Option<&Self> {
        crate::standard::project_concrete::<Self>(object)
    }
}

impl<T: Send + Sync + 'static> Transient for HandleAdaptor<T> {
    fn dynamic_type(&self) -> &'static RuntimeType {
        StandardTransient::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.parent.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.parent
    }
}

/// A shared handle to any object (`NCollection_Handle<T>`); `ObjectHandle::new(vec![1, 2])`.
pub struct ObjectHandle<T: Send + Sync + 'static> {
    handle: Handle<HandleAdaptor<T>>,
}

impl<T: Send + Sync + 'static> ObjectHandle<T> {
    /// Takes ownership of a new object (`NCollection_Handle(T*)` with a non-null pointer).
    ///
    /// Example: `ObjectHandle::new(5).get() == Some(&5)`.
    pub fn new(object: T) -> Self {
        Self {
            handle: Handle::new(HandleAdaptor {
                parent: StandardTransient::default(),
                object,
            }),
        }
    }

    /// Borrows the object, `None` for a null handle (`get`, `operator->`, `operator*`).
    ///
    /// Shared ownership only grants shared borrows in Rust; mutate through `T`'s interior
    /// mutability. Example: `handle.get().map(|v| v.len())`.
    pub fn get(&self) -> Option<&T> {
        self.handle.get().map(|adaptor| &adaptor.object)
    }

    /// True when no object is held (inherited `IsNull`); `ObjectHandle::<i32>::default().is_null()`.
    pub fn is_null(&self) -> bool {
        self.handle.is_null()
    }

    /// The same object as a `Handle<Standard_Transient>` (the native base class).
    ///
    /// Example: `ObjectHandle::new(1).transient().ref_count() == 2`.
    pub fn transient(&self) -> Handle<StandardTransient> {
        self.handle.cast()
    }

    /// Recovers the typed handle when `other` holds an adaptor of `T`, else null (`DownCast`).
    ///
    /// Example: `ObjectHandle::<i32>::down_cast(&h.transient()).get() == Some(&1)`.
    pub fn down_cast(other: &Handle<StandardTransient>) -> Self {
        Self {
            handle: other.cast(),
        }
    }
}

impl<T: Send + Sync + 'static> Default for ObjectHandle<T> {
    /// The null handle (`NCollection_Handle()` or a null pointer).
    fn default() -> Self {
        Self {
            handle: Handle::default(),
        }
    }
}

impl<T: Send + Sync + 'static> Clone for ObjectHandle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<T: Send + Sync + 'static> PartialEq for ObjectHandle<T> {
    /// Identity of the shared object, as the inherited handle comparison.
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
