//! Private native grid structures inherit the Graphic3d descriptor without new RTTI.
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::v3d::V3dComputedStructure;

#[test]
fn computed_structure_inherits_native_descriptor_and_parent() {
    let descriptor = V3dComputedStructure::static_type();
    assert_eq!(descriptor.name(), "Graphic3d_Structure");
    assert_eq!(descriptor.parent().unwrap().name(), "Standard_Transient");
    assert!(std::ptr::eq(
        descriptor,
        <dyn Graphic3dStructureSource>::static_type()
    ));
}

#[test]
fn inherited_descriptor_preserves_concrete_and_interface_casts() {
    let driver = NullGraphicDriver::new_handle();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let structure = V3dComputedStructure::create(&manager, &Handle::default()).unwrap();
    assert!(structure.is_instance(<dyn Graphic3dStructureSource>::static_type()));
    let concrete = structure.cast::<V3dComputedStructure>();
    assert!(!concrete.is_null());
    assert_eq!(concrete.cast::<dyn Graphic3dStructureSource>(), structure);
    structure.compute();
}
