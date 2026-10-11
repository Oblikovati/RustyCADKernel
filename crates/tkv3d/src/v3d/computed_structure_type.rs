//! OCCT 3d097a0 private grid/trihedron classes override Compute, but do not declare RTTI.
use super::V3dComputedStructure;
use rustycad_tkernel::standard::{
    Handle, InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, StandardTransient,
    Transient, project_concrete,
};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::{
    any::{Any, TypeId},
    sync::OnceLock,
};

impl RuntimeTyped for V3dComputedStructure {
    const TYPE_NAME: &'static str = <dyn Graphic3dStructureSource as RuntimeTyped>::TYPE_NAME;
    fn static_type() -> &'static RuntimeType {
        <dyn Graphic3dStructureSource>::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        project_concrete(object)
    }
}
impl Transient for V3dComputedStructure {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn rust_interface_descriptor(&self) -> &RuntimeType {
        computed_structure_casters()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.transient.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.transient
    }
}

fn computed_structure_casters() -> &'static RuntimeType {
    static DESCRIPTOR: OnceLock<Handle<RuntimeType>> = OnceLock::new();
    DESCRIPTOR
        .get_or_init(register_computed_structure_casters)
        .get()
        .expect("computed structure caster descriptor")
}
fn register_computed_structure_casters() -> Handle<RuntimeType> {
    // Rust needs a concrete caster even though native DynamicType stays inherited.
    RuntimeType::register_owned_with_interfaces::<V3dComputedStructure>(
        "Martensite computed structure Rust casters",
        std::mem::size_of::<V3dComputedStructure>(),
        Some(V3dComputedStructure::static_type_handle()),
        vec![InterfaceEntry::new(InterfaceCaster::<
            dyn Graphic3dStructureSource,
        >::new(|object| {
            Some(project_concrete::<V3dComputedStructure>(object)? as &dyn Graphic3dStructureSource)
        }))],
    )
}
