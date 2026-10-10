//! Classes without a native RTTI declaration still need concrete Rust trait casters.
use rustycad_tkernel::standard::{
    Handle, InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, StandardTransient,
    Transient, project_concrete,
};
use std::{
    any::{Any, TypeId},
    sync::OnceLock,
};

trait InheritedReading: Transient {
    fn reading(&self) -> i32;
}
rustycad_tkernel::impl_transient_interface!(
    dyn InheritedReading,
    "InheritedReading",
    StandardTransient
);
trait MissingReading: Transient {}
rustycad_tkernel::impl_transient_interface!(
    dyn MissingReading,
    "MissingReading",
    StandardTransient
);
#[derive(Default)]
struct InheritedReadingObject {
    base: StandardTransient,
}
impl InheritedReading for InheritedReadingObject {
    fn reading(&self) -> i32 {
        42
    }
}
impl RuntimeTyped for InheritedReadingObject {
    const TYPE_NAME: &'static str = <dyn InheritedReading as RuntimeTyped>::TYPE_NAME;
    fn static_type() -> &'static RuntimeType {
        <dyn InheritedReading>::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_concrete(object)
    }
}
impl Transient for InheritedReadingObject {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn rust_interface_descriptor(&self) -> &RuntimeType {
        inherited_reading_casters()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.base.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.base
    }
}
fn inherited_reading_casters() -> &'static RuntimeType {
    static DESCRIPTOR: OnceLock<Handle<RuntimeType>> = OnceLock::new();
    DESCRIPTOR
        .get_or_init(|| {
            RuntimeType::register_owned_with_interfaces::<InheritedReadingObject>(
                "InheritedReadingObject Rust casters",
                std::mem::size_of::<InheritedReadingObject>(),
                Some(InheritedReadingObject::static_type_handle()),
                vec![InterfaceEntry::new(
                    InterfaceCaster::<dyn InheritedReading>::new(|object| {
                        Some(project_concrete::<InheritedReadingObject>(object)?
                            as &dyn InheritedReading)
                    }),
                )],
            )
        })
        .get()
        .unwrap()
}
#[test]
fn inherited_native_type_and_rust_projection_remain_independent() {
    let concrete = Handle::new(InheritedReadingObject::default());
    let view = concrete.cast::<dyn InheritedReading>();
    assert!(!view.is_null());
    assert_eq!(view.reading(), 42);
    assert!(view.is_instance(<dyn InheritedReading>::static_type()));
    assert!(std::ptr::eq(
        view.dynamic_type().unwrap(),
        InheritedReadingObject::static_type()
    ));
    assert!(!view.is_kind_name("InheritedReadingObject Rust casters"));
    assert_eq!(concrete.object_address(), view.object_address());
    assert_eq!(concrete.ref_count(), 2);
}
#[test]
fn inherited_interface_casts_back_and_rejects_unrelated_interfaces() {
    let concrete = Handle::new(InheritedReadingObject::default());
    let view = concrete.cast::<dyn InheritedReading>();
    assert_eq!(
        view.cast::<InheritedReadingObject>().as_ptr(),
        concrete.as_ptr()
    );
    assert!(view.cast::<dyn MissingReading>().is_null());
    assert!(!view.cast::<StandardTransient>().is_null());
    let weak = view.downgrade();
    drop(view);
    assert_eq!(weak.upgrade().object_address(), concrete.object_address());
    drop(concrete);
    assert!(weak.upgrade().is_null());
}
#[test]
fn ordinary_transients_use_their_existing_dynamic_descriptor_for_casts() {
    let object = StandardTransient::default();
    assert!(std::ptr::eq(
        object.rust_interface_descriptor(),
        object.dynamic_type()
    ));
}
