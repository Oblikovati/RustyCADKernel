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

//! Shared owning handles and non-owning projected views preserve one object identity.
//!
//! `T` is a concrete class (a struct, reached through its embedded subobjects) or
//! an abstract class written as a trait (`Handle<dyn Trait>`, reached through the
//! interface casters of the dynamic descriptor).
use super::{RuntimeType, RuntimeTyped, Transient};
use std::{any::TypeId, marker::PhantomData, sync::Arc};
mod casts;
mod traits;
mod view;
pub use view::HandleView;
/// Nullable shared ownership with runtime checked base projection; `Handle::new(value)`.
pub struct Handle<T: ?Sized + RuntimeTyped> {
    owner: Option<Arc<dyn Transient>>,
    projected: PhantomData<T>,
}
impl<T: ?Sized + RuntimeTyped> Default for Handle<T> {
    fn default() -> Self {
        Self {
            owner: None,
            projected: PhantomData,
        }
    }
}
impl<T: RuntimeTyped> Handle<T> {
    /// Create shared ownership of a fresh concrete object; `Handle::new(StandardTransient::default())`.
    pub fn new(value: T) -> Self
    where
        T: Transient,
    {
        let owner: Arc<dyn Transient> = Arc::new(value);
        Self::adopt(Some(&owner))
    }
    /// Observe the typed address without transferring ownership; `handle.as_ptr()`.
    pub fn as_ptr(&self) -> *const T {
        self.get().map_or(std::ptr::null(), std::ptr::from_ref)
    }
}
impl<T: ?Sized + RuntimeTyped> Handle<T> {
    /// Share `owner` when it projects to `T` (one more reference), else null.
    fn adopt(owner: Option<&Arc<dyn Transient>>) -> Self {
        let Some(owner) = owner.filter(|owner| T::project_from(owner.as_ref()).is_some()) else {
            return Self::default();
        };
        owner.transient_base().increment_ref_counter();
        Self {
            owner: Some(owner.clone()),
            projected: PhantomData,
        }
    }
    /// Check for absent ownership; `handle.is_null()`.
    pub fn is_null(&self) -> bool {
        self.owner.is_none()
    }
    /// Rust's explicit boolean conversion; `handle.is_some()`.
    pub fn is_some(&self) -> bool {
        self.owner.is_some()
    }
    /// Borrow the actual projected subobject or interface view; `handle.get()`.
    pub fn get(&self) -> Option<&T> {
        T::project_from(self.owner.as_deref()?)
    }
    /// Observe the object address, independent of the projected type; `handle.object_address()`.
    ///
    /// Two handles to one object give the same address, whatever their `T`.
    pub fn object_address(&self) -> *const () {
        self.owner
            .as_ref()
            .map_or(std::ptr::null(), |owner| Arc::as_ptr(owner).cast::<()>())
    }
    /// Cast ownership to a supported base/derived type, returning null on failure; `handle.cast::<Base>()`.
    pub fn cast<U: ?Sized + RuntimeTyped>(&self) -> Handle<U> {
        Handle::adopt(self.owner.as_ref())
    }
    /// Borrow a projected view without incrementing the count; `handle.view::<Base>()`.
    pub fn view<U: ?Sized + RuntimeTyped>(&self) -> HandleView<'_, U> {
        HandleView::new(self.owner.as_ref())
    }
    /// Dispatch the original object's dynamic descriptor; `handle.dynamic_type()`.
    pub fn dynamic_type(&self) -> Option<&'static RuntimeType> {
        self.owner.as_ref().map(|owner| owner.dynamic_type())
    }
    /// Read concrete Rust RTTI independently of OCCT descriptor inheritance; `handle.rust_type_id()`.
    pub fn rust_type_id(&self) -> Option<TypeId> {
        self.owner.as_ref().map(|owner| owner.as_ref().type_id())
    }
    /// Check dynamic ancestry, false for null; `handle.is_kind(Base::static_type())`.
    pub fn is_kind(&self, base: &RuntimeType) -> bool {
        self.owner.as_ref().is_some_and(|owner| owner.is_kind(base))
    }
    /// Check dynamic ancestry by name, false for null; `handle.is_kind_name("Standard_Transient")`.
    pub fn is_kind_name(&self, name: &str) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.is_kind_name(name))
    }
    /// Check exact registered type, false for null; `handle.is_instance(Derived::static_type())`.
    pub fn is_instance(&self, exact: &RuntimeType) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.is_instance(exact))
    }
    /// Check exact native name, false for null; `handle.is_instance_name("Derived")`.
    pub fn is_instance_name(&self, name: &str) -> bool {
        self.owner
            .as_ref()
            .is_some_and(|owner| owner.is_instance_name(name))
    }
    /// Read the embedded counter; `handle.ref_count()` is zero for null handles.
    pub fn ref_count(&self) -> i32 {
        self.owner
            .as_ref()
            .map_or(0, |owner| owner.transient_base().ref_count())
    }
    /// Release ownership and become null; `handle.nullify()`.
    pub fn nullify(&mut self) {
        *self = Self::default();
    }
    /// Address of the owned object, 0 for null: the identity `<` and the hash compare.
    fn address(&self) -> usize {
        self.object_address() as usize
    }
}
impl<T: ?Sized + RuntimeTyped> Clone for Handle<T> {
    fn clone(&self) -> Self {
        self.cast::<T>()
    }
    /// Assignment keeps the count untouched when both handles already share the object.
    fn clone_from(&mut self, source: &Self) {
        if *self != *source {
            *self = source.clone();
        }
    }
}
impl<T: ?Sized + RuntimeTyped> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(owner) = &self.owner {
            owner.transient_base().decrement_ref_counter();
        }
    }
}
