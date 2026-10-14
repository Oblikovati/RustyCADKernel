//! Defensive internal refusal paths keep their observed context and do not invent causes.
use super::*;
use crate::capture_owned_fixture::owned_frame_capture;
use rustycad_tkservice::image::ImagePixMapData;

#[test]
fn unsupported_internal_conversion_reports_copy_refusal_without_mutating_rows() {
    let capture = owned_frame_capture();
    let kind = Graphic3dBufferType::ShadowMap;
    let request = capture_image_request(&capture, Graphic3dBufferType::Rgba, 0).unwrap();
    let image = ImagePixMap::default();
    assert!(image.init_zero(ImageFormat::Rgba, 2, 2, 0, 91));
    let error = copy_allocated_capture(&capture, &image, kind, request, "owned-image").unwrap_err();
    assert_eq!(error.operation, CaptureImageOperation::CopyRows);
    assert_eq!(
        error.failure,
        CaptureImageFailure::CopyRefused {
            request,
            source_type: "owned-image"
        }
    );
    assert!(std::error::Error::source(&error).is_none());
    assert_eq!(image.data().data.data(), &[91; 16]);
}

#[test]
fn shared_row_copy_reports_missing_rows() {
    let mut storage = ImagePixMapData::default();
    assert!(!copy_capture_rows(&mut storage, &[1, 2, 3, 4], 4));
    assert!(storage.data().is_empty());
}

#[test]
fn overflowing_returned_layout_request_is_rejected_without_indexing_rows() {
    let image = ImagePixMap::default();
    let mut request =
        capture_image_request(&owned_frame_capture(), Graphic3dBufferType::Rgba, 0).unwrap();
    request.dimensions.x = usize::MAX;
    let error = validate_capture_image_layout(
        &image.data(),
        Graphic3dBufferType::Rgba,
        request,
        "owned-image",
    )
    .unwrap_err();
    assert_eq!(error.operation, CaptureImageOperation::ValidateDestination);
    assert!(matches!(
        error.failure,
        CaptureImageFailure::InvalidDestination { .. }
    ));
}
