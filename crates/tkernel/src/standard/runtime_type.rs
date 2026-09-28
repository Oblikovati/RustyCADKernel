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
//!
//! A descriptor can also carry interface casters. An abstract OCCT class with
//! virtual methods (for example `Geom2d_Curve`) is a Rust trait, and its handles
//! are `Handle<dyn Trait>`. Rust cannot change a `&dyn Transient` into another
//! trait object by itself, so each concrete class registers, in its own
//! descriptor, one safe function per abstract ancestor that does the view change
//! (see docs/geom-design.md).
use super::Transient;
use std::any::{Any, TypeId};

/// Safe view change from a live object to an interface that it implements.
///
/// The concrete class supplies the function, normally through `impl_transient!`;
/// `InterfaceCaster::<dyn MyTrait>::new(my_cast_fn)`.
pub struct InterfaceCaster<I: ?Sized + 'static> {
    cast: for<'x> fn(&'x dyn Transient) -> Option<&'x I>,
}
impl<I: ?Sized + 'static> InterfaceCaster<I> {
    /// Wrap a view-change function; `InterfaceCaster::new(my_cast_fn)`.
    pub const fn new(cast: for<'x> fn(&'x dyn Transient) -> Option<&'x I>) -> Self {
        Self { cast }
    }
    /// Borrow `object` as the interface; `caster.cast(object)`.
    pub fn cast<'x>(&self, object: &'x dyn Transient) -> Option<&'x I> {
        (self.cast)(object)
    }
}
/// One registered interface: its Rust identity and its type-erased caster.
pub struct InterfaceEntry {
    interface: TypeId,
    caster: Box<dyn Any + Send + Sync>,
}
impl InterfaceEntry {
    /// Register the caster of interface `I`;
    /// `InterfaceEntry::new(InterfaceCaster::<dyn MyTrait>::new(my_cast_fn))`.
    pub fn new<I: ?Sized + 'static>(caster: InterfaceCaster<I>) -> Self {
        Self {
            interface: TypeId::of::<I>(),
            caster: Box::new(caster),
        }
    }
}
/// One registered native type and its parent; descriptors have static lifetime.
pub struct RuntimeType {
    name: &'static str,
    rust_type: TypeId,
    parent: Option<&'static RuntimeType>,
    interfaces: Vec<InterfaceEntry>,
}
impl RuntimeType {
    /// Register a Rust type's native name and parent; `RuntimeType::new::<T>("T", parent)`.
    ///
    /// `T` can be a trait object (`dyn MyTrait`) that stands for an abstract class.
    pub fn new<T: ?Sized + 'static>(name: &'static str, parent: Option<&'static Self>) -> Self {
        Self::with_interfaces::<T>(name, parent, Vec::new())
    }
    /// Register a concrete class with the interface casters of its abstract
    /// ancestors; `RuntimeType::with_interfaces::<T>("T", parent, entries)`.
    pub fn with_interfaces<T: ?Sized + 'static>(
        name: &'static str,
        parent: Option<&'static Self>,
        interfaces: Vec<InterfaceEntry>,
    ) -> Self {
        Self {
            name,
            rust_type: TypeId::of::<T>(),
            parent,
            interfaces,
        }
    }
    /// Read the native spelling; `descriptor.name()`.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// Read the Rust identity associated with this descriptor; `descriptor.rust_type()`.
    pub fn rust_type(&self) -> TypeId {
        self.rust_type
    }
    /// Borrow the direct parent, if any; `descriptor.parent()`.
    pub fn parent(&self) -> Option<&'static Self> {
        self.parent
    }
    /// Find the caster that this class registered for interface `I`;
    /// `descriptor.interface_caster::<dyn MyTrait>()`.
    pub fn interface_caster<I: ?Sized + 'static>(&self) -> Option<&InterfaceCaster<I>> {
        self.interfaces
            .iter()
            .find(|entry| entry.interface == TypeId::of::<I>())?
            .caster
            .downcast_ref::<InterfaceCaster<I>>()
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
/// Static type registration; implement once per class with `impl_transient!`
/// (concrete structs) or `impl_transient_interface!` (abstract classes as traits).
pub trait RuntimeTyped: 'static {
    /// Return the one static descriptor for this class; `T::static_type()`.
    fn static_type() -> &'static RuntimeType;
    /// Borrow `Self` inside a live object: the embedded subobject of a concrete
    /// class, or the interface view of an abstract class; `T::project_from(object)`.
    fn project_from(object: &dyn Transient) -> Option<&Self>;
}
/// Borrow a live object as an interface through the caster of its dynamic
/// descriptor; `project_interface::<dyn MyTrait>(object)`.
pub fn project_interface<I: ?Sized + 'static>(object: &dyn Transient) -> Option<&I> {
    object.dynamic_type().interface_caster::<I>()?.cast(object)
}
