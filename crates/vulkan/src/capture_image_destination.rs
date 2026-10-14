//! Completed captures use the existing conversion and row-copy path with a typed image builder.
use crate::FrameCapture;
use crate::capture_image_error::{CaptureImageError, CaptureImageFailure, CaptureImageOperation};
use crate::capture_image_layout::validate_capture_image_layout;
use crate::view_capture::{
    capture_bytes, capture_image_format, checked_capture_pixels, copy_capture_rows,
};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{
        ImageAllocationRequest, ImageFormat, ImagePixMap, ImagePixMapSource,
        image_pix_map_original_base,
    },
};

impl FrameCapture {
    /// Write a completed capture to a mutable image builder; `capture.write_image(&image, kind, 0)?`.
    ///
    /// Empty or malformed captures fail before any destination callback. The writer
    /// supports RGBA, RGB, Red and Depth, and requests at least `stride` bytes per row.
    /// Allocation errors may leave the destination changed by its own callback.
    /// This copies an existing capture; it does not redraw or certify rendered content.
    pub fn write_image(
        &self,
        destination: &dyn ImagePixMapSource,
        kind: Graphic3dBufferType,
        stride: usize,
    ) -> Result<(), CaptureImageError> {
        let request = capture_image_request(self, kind, stride)?;
        let image = allocate_capture_destination(destination, kind, request)?;
        let source_type = destination.dynamic_type().name();
        copy_allocated_capture(self, image, kind, request, source_type)
    }
}

fn capture_image_request(
    capture: &FrameCapture,
    kind: Graphic3dBufferType,
    stride: usize,
) -> Result<ImageAllocationRequest, CaptureImageError> {
    validate_capture_storage(capture, kind)?;
    validate_capture_request(
        ImageAllocationRequest {
            format: capture_requested_format(kind)?,
            dimensions: Vec3::new(capture.width as usize, capture.height as usize, 1),
            stride,
        },
        kind,
    )
}

fn capture_requested_format(kind: Graphic3dBufferType) -> Result<ImageFormat, CaptureImageError> {
    capture_image_format(&kind).ok_or_else(|| {
        CaptureImageError::new(
            CaptureImageOperation::ValidateRequest,
            kind,
            CaptureImageFailure::UnsupportedBuffer,
        )
    })
}

fn validate_capture_storage(
    capture: &FrameCapture,
    kind: Graphic3dBufferType,
) -> Result<(), CaptureImageError> {
    if checked_capture_pixels(capture).is_some() {
        return Ok(());
    }
    Err(CaptureImageError::new(
        CaptureImageOperation::ValidateCapture,
        kind,
        CaptureImageFailure::MalformedCapture {
            extent: [capture.width, capture.height],
            rgba_bytes: capture.rgba.len(),
            depth_values: capture.depth.len(),
        },
    ))
}

fn allocate_capture_destination(
    destination: &dyn ImagePixMapSource,
    kind: Graphic3dBufferType,
    request: ImageAllocationRequest,
) -> Result<&ImagePixMap, CaptureImageError> {
    let source_error = |cause| {
        CaptureImageError::new(
            CaptureImageOperation::AllocateDestination,
            kind,
            CaptureImageFailure::ImageSource {
                request,
                cause: Box::new(cause),
            },
        )
    };
    let image = image_pix_map_original_base(destination).map_err(source_error)?;
    destination.allocate_image(request).map_err(source_error)?;
    Ok(image)
}

fn copy_allocated_capture(
    capture: &FrameCapture,
    image: &ImagePixMap,
    kind: Graphic3dBufferType,
    request: ImageAllocationRequest,
    source_type: &'static str,
) -> Result<(), CaptureImageError> {
    let mut storage = image.change_data();
    let row_bytes = validate_capture_image_layout(&storage, kind, request, source_type)?;
    let copied = capture_bytes(&kind, storage.format, capture)
        .is_some_and(|bytes| copy_capture_rows(&mut storage.data, &bytes, row_bytes));
    if copied {
        return Ok(());
    }
    Err(capture_copy_refused(kind, request, source_type))
}
fn capture_copy_refused(
    kind: Graphic3dBufferType,
    request: ImageAllocationRequest,
    source_type: &'static str,
) -> CaptureImageError {
    CaptureImageError::new(
        CaptureImageOperation::CopyRows,
        kind,
        CaptureImageFailure::CopyRefused {
            request,
            source_type,
        },
    )
}

fn validate_capture_request(
    request: ImageAllocationRequest,
    kind: Graphic3dBufferType,
) -> Result<ImageAllocationRequest, CaptureImageError> {
    let row_bytes = request
        .dimensions
        .x
        .checked_mul(ImagePixMap::size_pixel_bytes_of(request.format));
    let storage_bytes =
        row_bytes.and_then(|row| row.max(request.stride).checked_mul(request.dimensions.y));
    if storage_bytes.is_some_and(|bytes| bytes <= isize::MAX as usize) {
        return Ok(request);
    }
    Err(CaptureImageError::new(
        CaptureImageOperation::ValidateRequest,
        kind,
        CaptureImageFailure::UnrepresentableLayout { request },
    ))
}

#[cfg(test)]
#[path = "capture_image_copy_tests.rs"]
mod capture_image_copy_tests;
