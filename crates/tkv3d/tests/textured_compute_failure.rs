//! Native shading catch keeps emitted geometry and only catches typed Standard_Failure transport.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[allow(dead_code)]
#[path = "support/textured_face_fixture.rs"]
mod face_fixture;
#[path = "support/textured_compute_fault_factory.rs"]
mod fault_factory;
use fault_factory::{TexturedComputeFault, TexturedComputeFaultFactory};
use rustycad_tkernel::{message::Messenger, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};
use std::sync::Arc;

#[test]
fn textured_compute_failure_keeps_already_emitted_geometry_and_fresh_aspect() {
    for fault in [
        TexturedComputeFault::CheckedConstructor,
        TexturedComputeFault::NativeConstructor,
    ] {
        let (object, prs) = textured_fault_presentation(fault);
        object.compute(&Handle::default(), &prs.cast(), 3).unwrap();
        assert_eq!(prs.groups().len(), 1);
        assert!(!object.textured_shape_base().fields.read().aspect.is_null());
        let aspect = prs.groups().first().unwrap().aspects();
        let original = object.attributes().shading_aspect().aspect();
        assert_eq!(aspect.object_address(), original.object_address());
        let native = include_str!("fixtures/ais_textured_shape/native-compute-failure.txt");
        assert_eq!(native_textured_fault_row(&object, &prs), native.trim());
    }
}

#[test]
fn textured_compute_bool_failure_transport_preserves_assigned_texture_and_original_group() {
    let (object, prs) = textured_fault_presentation(TexturedComputeFault::NativeAvailability);
    object.compute(&Handle::default(), &prs.cast(), 3).unwrap();
    // This is an adapter transport regression; the native directory oracle proves constructor failure separately.
    assert_eq!(
        native_textured_fault_row(&object, &prs),
        "MARTENSITE_COMPUTE_FAILURE 1 1 0 1"
    );
}
fn native_textured_fault_row(object: &AisTexturedShape, prs: &Graphic3dStructure) -> String {
    let fields = object.textured_shape_base().fields.read();
    let group = prs.groups().first().unwrap().aspects();
    let original = object.attributes().shading_aspect().aspect();
    format!(
        "MARTENSITE_COMPUTE_FAILURE {} {} {} {}",
        prs.groups().len(),
        i32::from(!fields.aspect.is_null()),
        i32::from(fields.texture.is_null()),
        i32::from(group.object_address() == original.object_address())
    )
}

#[test]
fn textured_compute_unrelated_panic_escapes_native_failure_catch() {
    let (object, prs) = textured_fault_presentation(TexturedComputeFault::UnrelatedAvailability);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        object.compute(&Handle::default(), &prs.cast(), 3)
    }));
    assert_eq!(*result.unwrap_err().downcast::<u32>().unwrap(), 73);
    assert_eq!(prs.groups().len(), 1);
}

fn textured_fault_presentation(
    fault: TexturedComputeFault,
) -> (Handle<AisTexturedShape>, Handle<Graphic3dStructure>) {
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let face = face_fixture::textured_face_fixture(1).unwrap();
    let object = AisTexturedShape::with_services(
        &face,
        Arc::new(Messenger::default()),
        Arc::new(TexturedComputeFaultFactory::new(fault)),
    );
    let drawer = object.attributes();
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.u_iso_aspect().set_number(0);
    drawer.v_iso_aspect().set_number(0);
    (object, prs)
}
