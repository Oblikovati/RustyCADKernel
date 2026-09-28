// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2025 OPEN CASCADE SAS
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

//! `Standard_Type`: one registered descriptor per class, with its parent chain.
//!
//! C++ keys the registry by `typeid(T).name()`; Rust keys it by `TypeId`, the same
//! per-class identity, and reports `std::any::type_name` as the system name.
//!
//! A descriptor can also carry interface casters. An abstract OCCT class with
//! virtual methods (for example `Geom2d_Curve`) is a Rust trait, and its handles
//! are `Handle<dyn Trait>`. Rust cannot change a `&dyn Transient` into another
//! trait object by itself, so each concrete class registers, in its own
//! descriptor, one safe function per abstract ancestor that does the view change
//! (see docs/geom-design.md).
use super::{StandardTransient, Transient};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    io::{self, Write},
    sync::{Mutex, OnceLock, PoisonError},
};
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
/// One registered class: system and given names, instance size, parent descriptor and
/// the interface casters of its abstract ancestors.
pub struct RuntimeType {
    base: StandardTransient,
    system_name: &'static str,
    name: &'static str,
    size: usize,
    rust_type: TypeId,
    parent: Option<&'static RuntimeType>,
    interfaces: Vec<InterfaceEntry>,
}
crate::impl_transient!(RuntimeType, "Standard_Type", StandardTransient, base);
/// The process-wide registry behind `Standard_Type::Register`.
type Registry = Mutex<HashMap<TypeId, &'static RuntimeType>>;
fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(Registry::default)
}
/// The bytes a C string argument holds: everything before the first NUL.
pub(crate) fn c_prefix(text: &str) -> &str {
    text.split('\0').next().unwrap_or_default()
}
impl RuntimeType {
    /// Returns the existing descriptor of `T`, or registers a new one with a deep copy
    /// of `name` (up to its first NUL). As in C++, a second registration of the same class
    /// returns the first descriptor unchanged, whatever name, size or parent it passes.
    /// `T` can be a trait object (`dyn MyTrait`) that stands for an abstract class.
    /// `RuntimeType::register::<MyClass>("MyClass", size_of::<MyClass>(), Some(parent))`.
    pub fn register<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<&'static RuntimeType>,
    ) -> &'static RuntimeType {
        Self::register_with_interfaces::<T>(name, size, parent, Vec::new())
    }
    /// [`RuntimeType::register`] for a concrete class that also carries the interface
    /// casters of its abstract ancestors; a repeated registration ignores `interfaces`
    /// as it ignores the other arguments;
    /// `RuntimeType::register_with_interfaces::<T>("T", size_of::<T>(), parent, entries)`.
    pub fn register_with_interfaces<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<&'static RuntimeType>,
        interfaces: Vec<InterfaceEntry>,
    ) -> &'static RuntimeType {
        let mut registered = registry().lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = registered.get(&TypeId::of::<T>()) {
            return existing;
        }
        // The descriptor lives until process exit, like the C++ registry's static handles.
        let descriptor: &'static RuntimeType = Box::leak(Box::new(Self {
            base: StandardTransient::default(),
            system_name: std::any::type_name::<T>(),
            name: Box::leak(c_prefix(name).into()),
            size,
            rust_type: TypeId::of::<T>(),
            parent,
            interfaces,
        }));
        registered.insert(TypeId::of::<T>(), descriptor);
        descriptor
    }
    /// The descriptor of a registered class, as `Standard_Type::Instance<T>()`;
    /// `RuntimeType::instance::<StandardTransient>().name() == "Standard_Transient"`.
    pub fn instance<T: RuntimeTyped>() -> &'static RuntimeType {
        T::static_type()
    }
    /// Rust's name of the registered type (C++ reports the mangled `typeid` name);
    /// `descriptor.system_name()`.
    pub fn system_name(&self) -> &'static str {
        self.system_name
    }
    /// Read the native spelling; `descriptor.name()`.
    pub fn name(&self) -> &'static str {
        self.name
    }
    /// Size in bytes of one instance of the Rust type; `descriptor.size()`.
    pub fn size(&self) -> usize {
        self.size
    }
    /// Read the concrete Rust identity associated with this descriptor; `descriptor.rust_type()`.
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
    /// True when this type is `base` or inherits from it, as `SubType(handle)`.
    /// Like C++, the walk stops at the first ancestor smaller than `base`, since a class
    /// is never smaller than its parent; `derived.is_subtype(base)`.
    pub fn is_subtype(&self, base: &Self) -> bool {
        let mut current: Option<&Self> = Some(self);
        while let Some(candidate) = current.filter(|candidate| base.size <= candidate.size) {
            if std::ptr::eq(candidate, base) {
                return true;
            }
            current = candidate.parent;
        }
        false
    }
    /// True when this type or an ancestor has the given name (compared as a C string,
    /// so it ends at a NUL), as `SubType(const char*)`; `derived.is_subtype_name("Standard_Transient")`.
    pub fn is_subtype_name(&self, name: &str) -> bool {
        let name: &str = c_prefix(name);
        let mut current: Option<&Self> = Some(self);
        while let Some(candidate) = current {
            if candidate.name == name {
                return true;
            }
            current = candidate.parent;
        }
        false
    }
    /// Writes the descriptor address and name, as `Print`; `descriptor.print(&mut out)?`.
    pub fn print(&self, stream: &mut impl Write) -> io::Result<()> {
        write!(stream, "{self}")
    }
}
/// `Print` / `operator<<` format: `0x<descriptor address> : <name>`.
impl fmt::Display for RuntimeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:p} : {}", self, self.name)
    }
}
impl fmt::Debug for RuntimeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
/// Static type registration; implement once per class with `impl_transient!`
/// (concrete structs) or `impl_transient_interface!` (abstract classes as traits).
pub trait RuntimeTyped: 'static {
    /// The class name `get_type_name()` returns, without registering the class;
    /// `StandardTransient::TYPE_NAME == "Standard_Transient"`.
    const TYPE_NAME: &'static str;
    /// Return the one registered descriptor for this concrete class, as
    /// `get_type_descriptor()`; `T::static_type()`.
    fn static_type() -> &'static RuntimeType;
    /// Borrow `Self` inside a live object: the embedded subobject of a concrete
    /// class, or the interface view of an abstract class; `T::project_from(object)`.
    fn project_from(object: &dyn Transient) -> Option<&Self>;
}
/// The `project_from` of a concrete class: its embedded subobject in `object`;
/// `project_concrete::<StandardTransient>(object)`.
pub fn project_concrete<T: Any>(object: &dyn Transient) -> Option<&T> {
    object.project(TypeId::of::<T>())?.downcast_ref::<T>()
}
/// Borrow a live object as an interface through the caster of its dynamic
/// descriptor; `project_interface::<dyn MyTrait>(object)`.
pub fn project_interface<I: ?Sized + 'static>(object: &dyn Transient) -> Option<&I> {
    object.dynamic_type().interface_caster::<I>()?.cast(object)
}
