//! Transient base with native atomic counter ordering and safe base projections.
use super::{
    Failure, Handle, RuntimeType, RuntimeTyped, exceptions, project_concrete,
    runtime_type::c_prefix,
};
use std::{
    any::{Any, TypeId},
    sync::{
        OnceLock,
        atomic::{AtomicI32, Ordering, fence},
    },
};
/// Runtime object interface retained by every owning handle.
pub trait Transient: Any + Send + Sync {
    /// Dispatch the most-derived OCCT descriptor; `object.dynamic_type()`.
    fn dynamic_type(&self) -> &'static RuntimeType;
    /// Own the dynamic class descriptor; `object.dynamic_type_handle()`.
    fn dynamic_type_handle(&self) -> Handle<RuntimeType> {
        Handle::from_static(self.dynamic_type())
    }
    /// Project an actual base subobject, without pointer reinterpreting; `object.project(TypeId::of::<Base>())`.
    fn project(&self, requested: TypeId) -> Option<&dyn Any>;
    /// Borrow the embedded reference-count base; `object.transient_base().ref_count()`.
    fn transient_base(&self) -> &StandardTransient;
    /// True when the dynamic type is exactly `exact` (descriptor identity, as the C++
    /// handle comparison); `object.is_instance(StandardTransient::static_type())`.
    fn is_instance(&self, exact: &RuntimeType) -> bool {
        std::ptr::eq(self.dynamic_type(), exact)
    }
    /// True when the dynamic type's name equals `name` compared as a C string (it ends
    /// at a NUL); `object.is_instance_name("Standard_Transient")`.
    fn is_instance_name(&self, name: &str) -> bool {
        self.dynamic_type().name() == c_prefix(name)
    }
    /// True when the dynamic type is `base` or inherits from it; `object.is_kind(base)`.
    fn is_kind(&self, base: &RuntimeType) -> bool {
        self.dynamic_type().is_subtype(base)
    }
    /// True when the dynamic type or an ancestor is named `name`; `object.is_kind_name("Standard_Transient")`.
    fn is_kind_name(&self, name: &str) -> bool {
        self.dynamic_type().is_subtype_name(name)
    }
}
/// Root object with native atomic reference-count behavior; `StandardTransient::default()`.
#[repr(C)]
#[derive(Default)]
pub struct StandardTransient {
    references: AtomicI32,
}
impl StandardTransient {
    /// Read the intrusive count; a standalone object has zero, `object.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        self.references.load(Ordering::Relaxed)
    }
    /// Atomically acquire one native-style reference; `object.increment_ref_counter()`.
    pub fn increment_ref_counter(&self) {
        self.references.fetch_add(1, Ordering::Relaxed);
    }
    /// Release one reference using native release/acquire ordering; `object.decrement_ref_counter()`.
    pub fn decrement_ref_counter(&self) -> i32 {
        let remaining: i32 = self
            .references
            .fetch_sub(1, Ordering::Release)
            .wrapping_sub(1);
        if remaining == 0 {
            fence(Ordering::Acquire);
        }
        remaining
    }
    /// Reject an object no handle owns, as `This()` does; the message is OCCT's own
    /// (the offending value is the reference count 0); `object.this()?`.
    pub fn this(&self) -> Result<&Self, Failure> {
        if self.ref_count() == 0 {
            return Err(exceptions::program_error(
                "Attempt to create handle to object created in stack, not yet constructed, or destroyed",
            ));
        }
        Ok(self)
    }
}
impl Clone for StandardTransient {
    fn clone(&self) -> Self {
        Self::default()
    }
    fn clone_from(&mut self, _: &Self) { /* Native assignment preserves the counter. */
    }
}
impl RuntimeTyped for StandardTransient {
    const TYPE_NAME: &'static str = "Standard_Transient";
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<Handle<RuntimeType>> = OnceLock::new();
        DESCRIPTOR
            .get_or_init(|| {
                RuntimeType::register_owned::<Self>(
                    Self::TYPE_NAME,
                    std::mem::size_of::<Self>(),
                    None,
                )
            })
            .get()
            .expect("registered descriptor")
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_concrete::<Self>(object)
    }
}
impl Transient for StandardTransient {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        (requested == TypeId::of::<Self>()).then_some(self)
    }
    fn transient_base(&self) -> &StandardTransient {
        self
    }
}
/// Implement safe single inheritance through an actual embedded parent field, as
/// `DEFINE_STANDARD_RTTIEXT` + `IMPLEMENT_STANDARD_RTTIEXT` do.
///
/// `$parent` names the descriptor parent; `$field` is the embedded subobject that
/// base projections and the reference counter delegate to. The optional
/// `interfaces` list registers one caster per abstract ancestor trait, so that
/// `Handle<dyn Trait>` can view the object (see docs/geom-design.md).
/// Example: `impl_transient!(Derived, "Derived", Base, parent)` or
/// `impl_transient!(Circle, "Circle", dyn Conic, parent, interfaces: [dyn Curve, dyn Conic])`.
#[macro_export]
macro_rules! impl_transient {
    ($class:ty, $name:literal, $parent:ty, $field:ident) => {
        $crate::impl_transient!($class, $name, $parent, $field, interfaces: []);
    };
    ($class:ty, $name:literal, $parent:ty, $field:ident, interfaces: [$($interface:ty),* $(,)?]) => {
        impl $crate::standard::RuntimeTyped for $class {
            const TYPE_NAME: &'static str = $name;
            fn static_type() -> &'static $crate::standard::RuntimeType {
                static DESCRIPTOR: std::sync::OnceLock<$crate::standard::Handle<$crate::standard::RuntimeType>> =
                    std::sync::OnceLock::new();
                DESCRIPTOR.get_or_init(|| {
                    $crate::standard::RuntimeType::register_owned_with_interfaces::<Self>(
                            $name,
                            std::mem::size_of::<Self>(),
                            Some(<$parent as $crate::standard::RuntimeTyped>::static_type_handle()),
                            vec![$($crate::standard::InterfaceEntry::new(
                                $crate::standard::InterfaceCaster::<$interface>::new(|object| {
                                    let concrete: &$class =
                                        $crate::standard::project_concrete::<$class>(object)?;
                                    Some(concrete as &$interface)
                                }),
                            )),*],
                        )
                }).get().expect("registered descriptor")
            }
            fn project_from(object: &dyn $crate::standard::Transient) -> Option<&Self> {
                $crate::standard::project_concrete::<Self>(object)
            }
        }
        impl $crate::standard::Transient for $class {
            fn dynamic_type(&self) -> &'static $crate::standard::RuntimeType {
                <Self as $crate::standard::RuntimeTyped>::static_type()
            }
            fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
                if requested == std::any::TypeId::of::<Self>() {
                    return Some(self);
                }
                $crate::standard::Transient::project(&self.$field, requested)
            }
            fn transient_base(&self) -> &$crate::standard::StandardTransient {
                $crate::standard::Transient::transient_base(&self.$field)
            }
        }
    };
}
/// Register an abstract class, written as a trait, in the runtime type system.
///
/// The trait must have `Transient` as a supertrait. Handles of the trait object
/// (`Handle<dyn Trait>`) then see every concrete class that lists the trait in
/// the `interfaces` of its `impl_transient!`. A trait object has no size, so the
/// descriptor takes its parent's size: `is_subtype` stops at the first ancestor
/// smaller than the base, and an abstract C++ class is never smaller than its parent.
/// Example: `impl_transient_interface!(dyn Curve, "Geom2d_Curve", dyn Geometry)`.
#[macro_export]
macro_rules! impl_transient_interface {
    ($interface:ty, $name:literal, $parent:ty) => {
        impl $crate::standard::RuntimeTyped for $interface {
            const TYPE_NAME: &'static str = $name;
            fn static_type() -> &'static $crate::standard::RuntimeType {
                static DESCRIPTOR: std::sync::OnceLock<
                    $crate::standard::Handle<$crate::standard::RuntimeType>,
                > = std::sync::OnceLock::new();
                DESCRIPTOR
                    .get_or_init(|| {
                        let parent =
                            <$parent as $crate::standard::RuntimeTyped>::static_type_handle();
                        let size = parent.get().expect("parent descriptor").size();
                        $crate::standard::RuntimeType::register_owned::<Self>(
                            $name,
                            size,
                            Some(parent),
                        )
                    })
                    .get()
                    .expect("registered descriptor")
            }
            fn project_from(object: &dyn $crate::standard::Transient) -> Option<&Self> {
                $crate::standard::project_interface::<Self>(object)
            }
        }
    };
}
