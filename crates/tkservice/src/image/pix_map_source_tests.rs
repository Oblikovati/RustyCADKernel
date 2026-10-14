//! Expected sizes, byte values and hook counts are owned algebraic assertions.
use super::pix_map_source_fixture::{MissingOriginalImage, image_allocation_probe};
use super::{
    ImageAlienPixMap, ImageAllocationRequest, ImageFormat, ImagePixMap, ImagePixMapSource,
};
use super::{ImageSourceError, ImageSourceFailureClass, ImageSourceOperation};
use super::{image_pix_map_allocate, image_pix_map_clear, image_pix_map_original_base};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use std::sync::atomic::Ordering;

const RGB_REQUEST: ImageAllocationRequest = ImageAllocationRequest {
    format: ImageFormat::Rgb,
    dimensions: Vec3::new(5, 4, 1),
    stride: 16,
};

#[test]
fn qualified_clear_targets_registered_base_and_preserves_other_image() {
    let image = image_allocation_probe();
    image_pix_map_clear(&image).unwrap();
    assert!(image.base.is_empty());
    assert_eq!(image.alternate.data().data.data(), &[131; 4]);
    assert_eq!(image.clears.load(Ordering::Relaxed), 0);
}

#[test]
fn dynamic_allocation_calls_clear_once_on_original_storage() {
    let image = image_allocation_probe();
    let source: &dyn ImagePixMapSource = &image;
    source.allocate_image(RGB_REQUEST).unwrap();
    assert_eq!(image.base.size_xyz(), Vec3::new(5, 4, 1));
    assert_eq!(image.base.data().data.data(), &[0; 64]);
    assert_eq!(image.alternate.data().data.data(), &[131; 4]);
    assert_eq!(image.allocations.load(Ordering::Relaxed), 1);
    assert_eq!(image.clears.load(Ordering::Relaxed), 1);
}

#[test]
fn qualified_volume_allocation_dispatches_clear_without_recursive_allocation() {
    let image = image_allocation_probe();
    let request = ImageAllocationRequest {
        dimensions: Vec3::new(5, 4, 3),
        ..RGB_REQUEST
    };
    image_pix_map_allocate(&image, request).unwrap();
    assert_eq!(image.base.size_xyz(), Vec3::new(5, 4, 3));
    assert_eq!(image.base.data().data.data(), &[0; 192]);
    assert_eq!(image.alternate.data().data.data(), &[131; 4]);
    assert_eq!(image.allocations.load(Ordering::Relaxed), 0);
    assert_eq!(image.clears.load(Ordering::Relaxed), 1);
}

#[test]
fn projection_borrows_original_field_without_acquiring_ownership() {
    let image = image_allocation_probe();
    let source: &dyn ImagePixMapSource = &image;
    let base = image_pix_map_original_base(source).unwrap();
    assert!(std::ptr::eq(base, &image.base));
    assert_eq!(base.width(), 4);
    assert_eq!(image.transient_base().ref_count(), 0);
}

#[test]
fn declined_dynamic_allocation_preserves_pixels_and_reports_request() {
    let image = image_allocation_probe();
    image.decline_allocation.store(true, Ordering::Relaxed);
    let source: &dyn ImagePixMapSource = &image;
    let error = source.allocate_image(RGB_REQUEST).unwrap_err();
    assert_eq!(error.class, ImageSourceFailureClass::AllocationRefused);
    assert_eq!(error.request, Some(RGB_REQUEST));
    assert!(error.cause.is_none());
    assert_eq!(image.base.data().data.data(), &[17; 12]);
    assert_eq!(image.allocations.load(Ordering::Relaxed), 1);
    assert_eq!(image.clears.load(Ordering::Relaxed), 0);
}

#[test]
fn missing_image_registration_reports_typed_dynamic_source_context() {
    let source = MissingOriginalImage {
        base: Default::default(),
    };
    let argument: &dyn ImagePixMapSource = &source;
    let error = image_pix_map_original_base(argument).err().unwrap();
    assert_eq!(error.operation, ImageSourceOperation::Project);
    assert_eq!(error.class, ImageSourceFailureClass::MissingRegisteredBase);
    assert_eq!(error.source_type, "Martensite_MissingOriginalImage");
    assert_eq!(error.request, None);
    assert!(error.to_string().contains("expected registered image"));
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn missing_base_clear_and_allocate_report_operation_and_request() {
    let source = MissingOriginalImage {
        base: Default::default(),
    };
    let clear_error = source.clear_image().unwrap_err();
    assert_eq!(clear_error.operation, ImageSourceOperation::Clear);
    let allocate_error = source.allocate_image(RGB_REQUEST).unwrap_err();
    assert_eq!(allocate_error.operation, ImageSourceOperation::Allocate);
    assert_eq!(allocate_error.request, Some(RGB_REQUEST));
    assert_eq!(
        allocate_error.class,
        ImageSourceFailureClass::MissingRegisteredBase
    );
}

#[test]
fn callback_clear_failure_keeps_original_cause_and_allocation_context() {
    let image = image_allocation_probe();
    image.fail_clear.store(true, Ordering::Relaxed);
    let error = image.allocate_image(RGB_REQUEST).unwrap_err();
    assert_eq!(error.operation, ImageSourceOperation::Clear);
    assert_eq!(error.class, ImageSourceFailureClass::CallbackFailed);
    assert_eq!(error.request, Some(RGB_REQUEST));
    assert_eq!(error.cause.as_deref(), Some(&image.clear_failure));
    let cause = std::error::Error::source(&error).unwrap();
    assert_eq!(cause.downcast_ref::<Failure>(), Some(&image.clear_failure));
    assert_eq!(image.base.data().data.data(), &[17; 12]);
}

#[test]
fn concrete_allocator_refusal_reports_unknown_cause_without_changing_zero_depth() {
    let image = ImagePixMap::default();
    let request = ImageAllocationRequest {
        dimensions: Vec3::new(5, 4, 0),
        ..RGB_REQUEST
    };
    let error = image.allocate_image(request).unwrap_err();
    assert_eq!(error.operation, ImageSourceOperation::Allocate);
    assert_eq!(error.class, ImageSourceFailureClass::AllocationRefused);
    assert_eq!(error.request, Some(request));
    assert!(error.cause.is_none());
    assert!(image.is_empty());
    assert_eq!(image.format(), ImageFormat::Rgb);
}

#[test]
fn concrete_image_handle_projects_the_same_image_descriptor_and_storage() {
    let image = Handle::new(ImagePixMap::default());
    let source = image.cast::<dyn ImagePixMapSource>();
    assert_eq!(source.object_address(), image.object_address());
    assert!(std::ptr::eq(
        image_pix_map_original_base(&*source).unwrap(),
        &*image
    ));
    assert!(std::ptr::eq(
        <dyn ImagePixMapSource>::static_type(),
        ImagePixMap::static_type()
    ));
    assert_eq!(image.ref_count(), 2);
    source.allocate_image(RGB_REQUEST).unwrap();
    assert_eq!(image.data().data.data(), &[0; 64]);
    source.clear_image().unwrap();
    assert!(image.is_empty());
}

#[test]
fn alien_image_source_keeps_dynamic_identity_and_registered_parent_storage() {
    let image = Handle::new(ImageAlienPixMap::default());
    let source = image.cast::<dyn ImagePixMapSource>();
    assert_eq!(
        Transient::dynamic_type(&*source).name(),
        "Image_AlienPixMap"
    );
    assert!(std::ptr::eq(
        image_pix_map_original_base(&*source).unwrap(),
        &**image
    ));
    source.allocate_image(RGB_REQUEST).unwrap();
    assert_eq!(image.data().data.data(), &[0; 64]);
    source.clear_image().unwrap();
    assert!(image.is_empty());
}

fn require_image_send_sync<T: Send + Sync + ?Sized>() {}

#[test]
fn image_interface_and_context_types_retain_send_sync_bounds() {
    require_image_send_sync::<dyn ImagePixMapSource>();
    require_image_send_sync::<ImageAllocationRequest>();
    require_image_send_sync::<ImageSourceError>();
    require_image_send_sync::<ImageSourceFailureClass>();
    require_image_send_sync::<ImageSourceOperation>();
}
