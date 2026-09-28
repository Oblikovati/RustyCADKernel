// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

use super::{Handle, RuntimeTyped, Transient};
use std::{any::TypeId, marker::PhantomData, sync::Arc};
/// A const handle reference projected to a base type, with no extra owning reference.
pub struct HandleView<'a, T: RuntimeTyped> {
    owner: Option<&'a Arc<dyn Transient>>,
    projected: PhantomData<T>,
}
impl<'a, T: RuntimeTyped> HandleView<'a, T> {
    pub(super) fn new(owner: Option<&'a Arc<dyn Transient>>) -> Self {
        Self {
            owner: owner.filter(|owner| owner.project(TypeId::of::<T>()).is_some()),
            projected: PhantomData,
        }
    }
    /// The viewed owner, for building a new owning handle from this view.
    pub(super) fn owner(&self) -> Option<&'a Arc<dyn Transient>> {
        self.owner
    }
    /// Check a borrowed null handle; `view.is_null()`.
    pub fn is_null(&self) -> bool {
        self.owner.is_none()
    }
    /// Borrow the projected subobject; `view.get()`.
    pub fn get(&self) -> Option<&T> {
        self.owner?.project(TypeId::of::<T>())?.downcast_ref()
    }
    /// Observe the projected address; `view.as_ptr()`.
    pub fn as_ptr(&self) -> *const T {
        self.get().map_or(std::ptr::null(), std::ptr::from_ref)
    }
    /// Borrow the viewed object as a transient, as a `const Standard_Transient*`;
    /// `occ::is_kind::<Base>(view.transient())`.
    pub fn transient(&self) -> Option<&'a dyn Transient> {
        self.owner.map(|owner| owner.as_ref())
    }
}
impl<T: RuntimeTyped, U: RuntimeTyped> PartialEq<Handle<U>> for HandleView<'_, T> {
    fn eq(&self, other: &Handle<U>) -> bool {
        match (self.owner, other.owner.as_ref()) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
}
