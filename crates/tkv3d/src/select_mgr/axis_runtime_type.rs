//! Concrete selectors without native RTTI keep Rust handle casters separate.
use super::{SelectMgrAxisIntersector, SelectMgrBaseIntersector};
use rustycad_tkernel::standard::{
    Handle, InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, StandardTransient,
    Transient, project_concrete,
};
use std::{
    any::{Any, TypeId},
    sync::OnceLock,
};
macro_rules! inherited_intersector_type {
    ($concrete:ty, $parent:ty, [$($interface:ty),+]) => {
        impl RuntimeTyped for $concrete {
            const TYPE_NAME: &'static str = <$parent as RuntimeTyped>::TYPE_NAME;
            fn static_type() -> &'static RuntimeType { <$parent>::static_type() }
            fn project_from(object: &dyn Transient) -> Option<&Self> { project_concrete(object) }
        }
        impl Transient for $concrete {
            fn dynamic_type(&self) -> &'static RuntimeType { Self::static_type() }
            fn rust_interface_descriptor(&self) -> &RuntimeType {
                static DESCRIPTOR: OnceLock<Handle<RuntimeType>> = OnceLock::new();
                DESCRIPTOR.get_or_init(|| RuntimeType::register_owned_with_interfaces::<$concrete>(
                    concat!(stringify!($concrete), " Rust casters"), std::mem::size_of::<$concrete>(),
                    Some(Self::static_type_handle()),
                    vec![$(InterfaceEntry::new(InterfaceCaster::<$interface>::new(|object| {
                        Some(project_concrete::<$concrete>(object)? as &$interface)
                    }))),+],
                )).get().expect("inherited intersector caster descriptor")
            }
            fn project(&self, requested: TypeId) -> Option<&dyn Any> {
                if requested == TypeId::of::<Self>() { return Some(self); }
                self.transient.project(requested)
            }
            fn transient_base(&self) -> &StandardTransient { &self.transient }
        }
    };
}
inherited_intersector_type!(
    SelectMgrAxisIntersector,
    dyn SelectMgrBaseIntersector,
    [dyn SelectMgrBaseIntersector]
);
inherited_intersector_type!(super::SelectMgrRectangularFrustum, dyn super::SelectMgrBaseFrustum,
    [dyn super::SelectMgrBaseFrustum, dyn SelectMgrBaseIntersector]);

inherited_intersector_type!(super::SelectMgrTriangularFrustumSet, dyn super::SelectMgrBaseFrustum,
    [dyn super::SelectMgrBaseFrustum, dyn SelectMgrBaseIntersector]);
