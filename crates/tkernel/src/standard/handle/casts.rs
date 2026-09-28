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

//! Construction, assignment and casts that `opencascade::handle` spells as C++ overloads.
use super::{Handle, HandleView, RuntimeTyped, Transient};
use std::any::TypeId;
impl<T: RuntimeTyped> Handle<T> {
    /// `handle<T>::DownCast(other)`: shares `other`'s object when it is a `T`, else null.
    /// The C++ overload set (from a base, and the deprecated same/unrelated-type forms)
    /// is one runtime check here; `Handle::<Derived>::down_cast(&base)`.
    pub fn down_cast<U: RuntimeTyped>(other: &Handle<U>) -> Self {
        other.cast::<T>()
    }
    /// `handle<T>(const T*)` / `DownCast(const T2*)` for an object a handle already owns:
    /// a view is Rust's non-owning pointer, and the new handle adds one reference, or is
    /// null when the view is null or not a `T`; `Handle::<Base>::from_view(&derived.view())`.
    pub fn from_view<U: RuntimeTyped>(view: &HandleView<'_, U>) -> Self {
        Self::adopt(view.owner())
    }
    /// Moves ownership into a handle of another type without touching the count, as the
    /// converting move constructor does; a non-`U` object gives null (C++ rejects it at
    /// compile time) and releases the reference; `derived.into_cast::<Base>()`.
    pub fn into_cast<U: RuntimeTyped>(mut self) -> Handle<U> {
        // Taking the owner leaves `self` null, so its Drop releases nothing.
        let Some(owner) = self.owner.take() else {
            return Handle::default();
        };
        if owner.project(TypeId::of::<U>()).is_none() {
            owner.transient_base().decrement_ref_counter();
            return Handle::default();
        }
        Handle {
            owner: Some(owner),
            projected: std::marker::PhantomData,
        }
    }
    /// `reset(new U(...))`: releases the current object and owns a fresh `U`, which may be
    /// `T` or a class derived from it; a `U` that is not a `T` (a C++ compile error) leaves
    /// the handle null; `base.reset(Derived::default())`.
    pub fn reset<U: Transient + RuntimeTyped>(&mut self, value: U) {
        *self = Handle::new(value).into_cast::<T>();
    }
    /// `std::hash<Handle(T)>`: the owned object's address, 0 for null; `handle.hash_code()`.
    pub fn hash_code(&self) -> usize {
        self.address()
    }
}
