//! Destination metadata and rows are checked under the same guard used by the copier.
use crate::{CaptureImageError, CaptureImageFailure, CaptureImageLayout, CaptureImageOperation};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{IMAGE_BOTTOM_UP, ImageAllocationRequest, ImagePixMap, ImagePixMapState},
};

pub(crate) fn validate_capture_image_layout(
    storage: &ImagePixMapState,
    kind: Graphic3dBufferType,
    request: ImageAllocationRequest,
    source_type: &'static str,
) -> Result<usize, CaptureImageError> {
    let row_bytes = request
        .dimensions
        .x
        .checked_mul(ImagePixMap::size_pixel_bytes_of(request.format));
    row_bytes
        .filter(|row_bytes| {
            capture_image_metadata_matches(storage, request, *row_bytes)
                && capture_image_rows_match(storage, request.dimensions.y, *row_bytes)
        })
        .ok_or_else(|| capture_image_layout_rejection(storage, kind, request, source_type))
}
fn capture_image_rows_match(storage: &ImagePixMapState, height: usize, row_bytes: usize) -> bool {
    (0..height).all(|row| {
        storage
            .data
            .row(row)
            .is_some_and(|span| span.len() >= row_bytes)
    })
}
fn capture_image_layout_rejection(
    storage: &ImagePixMapState,
    kind: Graphic3dBufferType,
    request: ImageAllocationRequest,
    source_type: &'static str,
) -> CaptureImageError {
    CaptureImageError::new(
        CaptureImageOperation::ValidateDestination,
        kind,
        CaptureImageFailure::InvalidDestination {
            request,
            actual: Box::new(observed_capture_image_layout(storage)),
            source_type,
        },
    )
}

fn capture_image_metadata_matches(
    storage: &ImagePixMapState,
    request: ImageAllocationRequest,
    row_bytes: usize,
) -> bool {
    let layout = &storage.data;
    storage.format == request.format
        && Vec3::new(layout.size_x, layout.size_y, layout.size_z) == request.dimensions
        && layout.size_bpp == ImagePixMap::size_pixel_bytes_of(request.format)
        && layout.size_row_bytes >= row_bytes.max(request.stride)
        && layout.size_row_bytes.checked_mul(layout.size_y) == Some(layout.size_slice_bytes)
        && matches!(layout.top_to_down, 1 | IMAGE_BOTTOM_UP)
}

fn observed_capture_image_layout(storage: &ImagePixMapState) -> CaptureImageLayout {
    let layout = &storage.data;
    CaptureImageLayout {
        format: storage.format,
        dimensions: Vec3::new(layout.size_x, layout.size_y, layout.size_z),
        pixel_bytes: layout.size_bpp,
        row_bytes: layout.size_row_bytes,
        slice_bytes: layout.size_slice_bytes,
        storage_bytes: layout.size(),
        row_order: layout.top_to_down,
    }
}
