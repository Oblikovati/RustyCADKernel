//! Invalid immutable inputs never call a destination; invalid returned layouts never receive rows.
use crate::capture_image_destination_fixture::{
    DestinationMode, MissingCaptureImage, capture_destination_probe, counting_capture_destination,
};
use crate::capture_owned_fixture::owned_frame_capture;
use crate::{CaptureImageError, CaptureImageFailure, CaptureImageOperation, FrameCapture};
use rustycad_tkernel::{ncollection::Vec3, standard::Failure};
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{ImageAllocationRequest, ImageFormat, ImageSourceError, ImageSourceOperation},
};
use std::sync::atomic::Ordering;

fn malformed_owned_captures() -> [FrameCapture; 6] {
    let mut captures = std::array::from_fn(|_| owned_frame_capture());
    captures[0].width = 0;
    captures[0].height = 0;
    captures[0].rgba.clear();
    captures[0].depth.clear();
    captures[1].rgba.pop();
    captures[2].depth.pop();
    captures[3].rgba.push(99);
    captures[4].width = u32::MAX;
    captures[4].height = u32::MAX;
    captures[5].height = 0;
    captures
}

#[test]
fn malformed_and_empty_captures_do_not_invoke_any_destination_callback() {
    for capture in malformed_owned_captures() {
        let destination = counting_capture_destination();
        let original = destination.inner.base.data().clone();
        let error = capture
            .write_image(&destination, Graphic3dBufferType::Rgba, 0)
            .unwrap_err();
        assert_eq!(error.operation, CaptureImageOperation::ValidateCapture);
        assert_eq!(
            error.failure,
            CaptureImageFailure::MalformedCapture {
                extent: [capture.width, capture.height],
                rgba_bytes: capture.rgba.len(),
                depth_values: capture.depth.len(),
            }
        );
        assert_eq!(destination.callbacks.load(Ordering::Relaxed), 0);
        assert_eq!(*destination.inner.base.data(), original);
    }
}

#[test]
fn unsupported_buffers_fail_before_projection_or_allocation() {
    for kind in [
        Graphic3dBufferType::ShadowMap,
        Graphic3dBufferType::RgbRayTraceHdrLeft,
    ] {
        let destination = counting_capture_destination();
        let error = owned_frame_capture()
            .write_image(&destination, kind, 0)
            .unwrap_err();
        assert_eq!(error.operation, CaptureImageOperation::ValidateRequest);
        assert_eq!(error.failure, CaptureImageFailure::UnsupportedBuffer);
        assert_eq!(error.buffer, kind);
        assert_eq!(destination.callbacks.load(Ordering::Relaxed), 0);
        assert_eq!(destination.inner.base.data().data.data(), &[91]);
    }
}

#[test]
fn unrepresentable_stride_fails_before_any_destination_callback() {
    for stride in [usize::MAX, isize::MAX as usize] {
        let destination = counting_capture_destination();
        let error = owned_frame_capture()
            .write_image(&destination, Graphic3dBufferType::Rgba, stride)
            .unwrap_err();
        assert_eq!(error.operation, CaptureImageOperation::ValidateRequest);
        assert_eq!(
            error.failure,
            CaptureImageFailure::UnrepresentableLayout {
                request: ImageAllocationRequest {
                    stride,
                    ..rgba_capture_request()
                },
            }
        );
        assert_eq!(destination.callbacks.load(Ordering::Relaxed), 0);
        assert_eq!(destination.inner.base.data().data.data(), &[91]);
    }
}

fn rgba_capture_request() -> ImageAllocationRequest {
    ImageAllocationRequest {
        format: ImageFormat::Rgba,
        dimensions: Vec3::new(2, 2, 1),
        stride: 0,
    }
}

fn assert_original_source_error(mode: DestinationMode) {
    let destination = capture_destination_probe(mode);
    let error = owned_frame_capture()
        .write_image(&destination, Graphic3dBufferType::Rgba, 0)
        .unwrap_err();
    let mut expected = destination.failure.clone();
    if matches!(mode, DestinationMode::RefuseClear) {
        expected.request = Some(rgba_capture_request());
    }
    assert_source_error_chain(&error, &expected);
    assert_eq!(error.operation, CaptureImageOperation::AllocateDestination);
    assert_eq!(destination.base.data().data.data(), &[91]);
}

fn assert_source_error_chain(error: &CaptureImageError, expected: &ImageSourceError) {
    let original = std::error::Error::source(error)
        .unwrap()
        .downcast_ref::<ImageSourceError>()
        .unwrap();
    assert_eq!(original, expected);
    let cause = std::error::Error::source(original)
        .unwrap()
        .downcast_ref::<Failure>()
        .unwrap();
    assert_eq!(Some(cause), expected.cause.as_deref());
}

#[test]
fn allocation_and_clear_errors_preserve_exact_typed_source_context_and_cause() {
    assert_original_source_error(DestinationMode::RefuseAllocation);
    assert_original_source_error(DestinationMode::RefuseClear);
}

#[test]
fn missing_registered_image_keeps_the_original_projection_error() {
    let destination = MissingCaptureImage {
        base: Default::default(),
    };
    let error = owned_frame_capture()
        .write_image(&destination, Graphic3dBufferType::Rgba, 0)
        .unwrap_err();
    let source = std::error::Error::source(&error)
        .unwrap()
        .downcast_ref::<ImageSourceError>()
        .unwrap();
    assert_eq!(source.operation, ImageSourceOperation::Project);
    assert_eq!(source.source_type, "Martensite_MissingCaptureImage");
    assert!(source.cause.is_none());
    assert!(
        error
            .to_string()
            .contains("expected a positive complete capture")
    );
}

const INVALID_DESTINATION_MODES: [DestinationMode; 8] = [
    DestinationMode::WrongFormat,
    DestinationMode::WrongWidth,
    DestinationMode::WrongDepth,
    DestinationMode::WrongPixelBytes,
    DestinationMode::WrongStride,
    DestinationMode::WrongSliceBytes,
    DestinationMode::WrongRowOrder,
    DestinationMode::MissingRows,
];

fn assert_rejected_destination(mode: DestinationMode) {
    let destination = capture_destination_probe(mode);
    let error = owned_frame_capture()
        .write_image(&destination, Graphic3dBufferType::Rgba, 0)
        .unwrap_err();
    assert_eq!(error.operation, CaptureImageOperation::ValidateDestination);
    assert_rejected_layout_context(error, destination.base.format());
    let storage = destination.base.data();
    assert!(storage.data.data().iter().all(|byte| *byte == 167));
}

fn assert_rejected_layout_context(error: CaptureImageError, format: ImageFormat) {
    let CaptureImageFailure::InvalidDestination {
        request,
        source_type,
        actual,
    } = error.failure
    else {
        panic!("expected InvalidDestination, got {error:?}");
    };
    assert_eq!(request, rgba_capture_request());
    assert_eq!(source_type, "Martensite_CaptureDestinationProbe");
    assert_eq!(actual.format, format);
}

#[test]
fn invalid_derived_layouts_are_rejected_before_copying_any_rows() {
    for mode in INVALID_DESTINATION_MODES {
        assert_rejected_destination(mode);
    }
}
