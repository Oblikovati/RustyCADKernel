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

//! Explicit single-inheritance descriptors complement Rust's concrete TypeId.
use std::any::TypeId;
/// One registered native type and its parent; descriptors have static lifetime.
pub struct RuntimeType {
    name: &'static str,
    rust_type: TypeId,
    parent: Option<&'static RuntimeType>,
}
impl RuntimeType {
    /// Register a concrete Rust type's native name and parent; `RuntimeType::new::<T>("T", parent)`.
    pub fn new<T: 'static>(name: &'static str, parent: Option<&'static Self>) -> Self {
        Self {
            name,
            rust_type: TypeId::of::<T>(),
            parent,
        }
    }
    /// Read the native spelling; `descriptor.name()`.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// Read the concrete Rust identity associated with this descriptor; `descriptor.rust_type()`.
    pub fn rust_type(&self) -> TypeId {
        self.rust_type
    }
    /// Borrow the direct parent, if any; `descriptor.parent()`.
    pub fn parent(&self) -> Option<&'static Self> {
        self.parent
    }
    /// Check ancestry by registered type identity; `derived.is_subtype(base)`.
    pub fn is_subtype(&self, base: &Self) -> bool {
        let mut current: Option<&Self> = Some(self);
        while let Some(candidate) = current {
            if candidate.rust_type == base.rust_type {
                return true;
            }
            current = candidate.parent;
        }
        false
    }
    /// Check ancestry by native type name; `derived.is_subtype_name("Standard_Transient")`.
    pub fn is_subtype_name(&self, name: &str) -> bool {
        let mut current: Option<&Self> = Some(self);
        while let Some(candidate) = current {
            if candidate.name == name {
                return true;
            }
            current = candidate.parent;
        }
        false
    }
}
/// Static type registration; implement once per concrete class, e.g. with `impl_transient!`.
pub trait RuntimeTyped: 'static {
    /// Return the one static descriptor for this concrete class; `T::static_type()`.
    fn static_type() -> &'static RuntimeType;
}
