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
use super::{Handle, StandardTransient, Transient};
use std::{
    any::{Any, TypeId},
    fmt,
    io::{self, Write},
};
mod registry;
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
    system_name: String,
    name: String,
    size: usize,
    rust_type: TypeId,
    parent: Option<Handle<RuntimeType>>,
    interfaces: Vec<InterfaceEntry>,
}
crate::impl_transient!(RuntimeType, "Standard_Type", StandardTransient, base);
/// The bytes a C string argument holds: everything before the first NUL.
pub(crate) fn c_prefix(text: &str) -> &str {
    text.split('\0').next().unwrap_or_default()
}
impl RuntimeType {
    /// Return `T`'s borrowed descriptor, or register a deep copy of `name` through its first NUL.
    /// Repeated calls keep the first metadata. Storage is pinned for this compatibility API
    /// because its return type promises a process-long borrow; use `register_owned` when the
    /// descriptor must unbind at final handle drop. Example: `RuntimeType::register::<T>("T", 8, None)`.
    /// `T` can be a trait object (`dyn MyTrait`) that stands for an abstract class.
    /// `RuntimeType::register::<MyClass>("MyClass", size_of::<MyClass>(), Some(parent))`.
    pub fn register<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<&'static RuntimeType>,
    ) -> &'static RuntimeType {
        Self::register_legacy_borrowed::<T>(name, size, parent, Vec::new())
    }
    /// Keep one class-style owner for a descriptor registered by a generic Rust class;
    /// `RuntimeType::retain_class_descriptor::<MyGeneric<u32>>(descriptor)`.
    pub fn retain_class_descriptor<T: ?Sized + 'static>(
        descriptor: &'static RuntimeType,
    ) -> &'static RuntimeType {
        registry::retain_class_descriptor::<T>(descriptor)
    }
    /// Register or reuse an owned descriptor; final owner drop unbinds it unless a borrowed
    /// compatibility call has pinned storage. Example: `RuntimeType::register_owned::<T>("T", 8, None)`.
    /// `RuntimeType::register_owned::<T>("T", size_of::<T>(), Some(parent_handle))`.
    pub fn register_owned<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<Handle<RuntimeType>>,
    ) -> Handle<RuntimeType> {
        Self::register_owned_with_interfaces::<T>(name, size, parent, Vec::new())
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
        Self::register_legacy_borrowed::<T>(name, size, parent, interfaces)
    }
    /// `register_owned` with interface casts for abstract class projections;
    /// `RuntimeType::register_owned_with_interfaces::<T>("T", size_of::<T>(), None, Vec::new())`.
    pub fn register_owned_with_interfaces<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<Handle<RuntimeType>>,
        interfaces: Vec<InterfaceEntry>,
    ) -> Handle<RuntimeType> {
        registry::register_owned::<T>(name, size, parent, interfaces)
    }
    /// Pin an owned registration for `static_type()` compatibility and return its static borrow;
    /// `RuntimeType::retain_class_descriptor_handle::<MyGeneric<u32>>(descriptor)`.
    pub fn retain_class_descriptor_handle<T: ?Sized + 'static>(
        descriptor: Handle<RuntimeType>,
    ) -> &'static RuntimeType {
        registry::retain_class_handle::<T>(descriptor)
    }
    pub(super) fn new<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<Handle<RuntimeType>>,
        interfaces: Vec<InterfaceEntry>,
    ) -> Self {
        Self {
            base: StandardTransient::default(),
            system_name: std::any::type_name::<T>().to_owned(),
            name: c_prefix(name).to_owned(),
            size,
            rust_type: TypeId::of::<T>(),
            parent,
            interfaces,
        }
    }
    fn register_legacy_borrowed<T: ?Sized + 'static>(
        name: &str,
        size: usize,
        parent: Option<&'static RuntimeType>,
        interfaces: Vec<InterfaceEntry>,
    ) -> &'static RuntimeType {
        registry::register_legacy_borrowed::<T>(name, size, parent, interfaces)
    }
    /// The descriptor of a registered class, as `Standard_Type::Instance<T>()`;
    /// `RuntimeType::instance::<StandardTransient>().name() == "Standard_Transient"`.
    pub fn instance<T: RuntimeTyped>() -> &'static RuntimeType {
        T::static_type()
    }
    /// Rust's name of the registered type (C++ reports the mangled `typeid` name);
    /// `descriptor.system_name()`.
    pub fn system_name(&self) -> &str {
        &self.system_name
    }
    /// Read the native spelling; `descriptor.name()`.
    pub fn name(&self) -> &str {
        &self.name
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
    pub fn parent(&self) -> Option<&Self> {
        self.parent.as_ref().and_then(Handle::get)
    }
    /// Borrow the native parent handle; `descriptor.parent_handle().map(Handle::get)`.
    pub fn parent_handle(&self) -> Option<&Handle<RuntimeType>> {
        self.parent.as_ref()
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
            current = candidate.parent();
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
            current = candidate.parent();
        }
        false
    }
    /// Writes the descriptor address and name, as `Print`; `descriptor.print(&mut out)?`.
    pub fn print(&self, stream: &mut impl Write) -> io::Result<()> {
        write!(stream, "{self}")
    }
}
impl Drop for RuntimeType {
    fn drop(&mut self) {
        registry::remove_owned(self.rust_type, std::ptr::from_ref(self).cast());
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
    /// Own the class descriptor, incrementing its native-style count;
    /// `MyClass::static_type_handle()`.
    fn static_type_handle() -> Handle<RuntimeType> {
        Handle::from_static(Self::static_type())
    }
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
