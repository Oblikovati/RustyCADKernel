use super::light_source_tests::{FakeLightPresentation, assigned_transform};
use super::*;
use crate::prs_mgr::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::*;

#[test]
fn concrete_compute_dispatches_derived_transform_hooks() {
    let object = Handle::new(FakeLightPresentation::new(
        Graphic3dTypeOfLightSource::Ambient,
    ));
    prs_mgr_presentable_bind(&object.cast());
    ais_light_source_initialize(&*object).unwrap();
    let viewer = crate::v3d::V3dViewer::new(
        &rustycad_tkservice::null_driver::NullGraphicDriver::new_handle(),
    )
    .unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    ais_light_source_compute(&*object, &prs.cast(), 0).unwrap();
    assert_eq!(hook_calls(&object), (1, 1));
    ais_light_source_compute(&*object, &prs.cast(), 1).unwrap();
    assert_eq!(hook_calls(&object), (1, 1));
}
#[test]
fn failed_light_frame_refresh_clears_retained_local_datum() {
    let object = FakeLightPresentation::new(Graphic3dTypeOfLightSource::Directional);
    object
        .presentable_base()
        .fields
        .write()
        .local_transformation = Handle::new(Datum3::new(assigned_transform()));
    let failure = ais_light_source_update_local_transform(&object).unwrap_err();
    assert!(failure.message().contains("zero"));
    assert!(
        object
            .presentable_base()
            .read()
            .local_transformation
            .is_null()
    );
}

fn hook_calls(object: &FakeLightPresentation) -> (usize, usize) {
    let persistence = object
        .persistence_calls
        .load(std::sync::atomic::Ordering::Relaxed);
    let frame = object
        .frame_calls
        .load(std::sync::atomic::Ordering::Relaxed);
    (persistence, frame)
}

#[test]
fn both_native_bool_adapters_preserve_success_and_failure_types() {
    use super::light_source_object::native_light_bool_result;
    assert!(native_light_bool_result(Ok(true)));
    assert!(!native_light_bool_result(Ok(false)));
    let failure = rustycad_tkernel::standard::Failure::with_message(
        "Standard_ProgramError",
        "aspect=null; expected initialized native light aspect",
    );
    let panic = std::panic::catch_unwind(|| native_light_bool_result(Err(failure))).unwrap_err();
    let typed = panic
        .downcast_ref::<rustycad_tkernel::standard::Failure>()
        .unwrap();
    assert!(typed.message().contains("aspect=null"));
}
