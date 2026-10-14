//! Structured failures for copying a completed capture into a mutable image builder.
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkservice::{
    graphic3d::Graphic3dBufferType,
    image::{ImageAllocationRequest, ImageFormat, ImageSourceError},
};

/// Writer step that rejected the request; `CaptureImageOperation::ValidateCapture`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureImageOperation {
    /// Check immutable capture dimensions and storage before destination callbacks.
    ValidateCapture,
    /// Check the requested buffer and addressable row/slice layout.
    ValidateRequest,
    /// Project and allocate the destination through its typed source interface.
    AllocateDestination,
    /// Check the returned mutable image layout under its write guard.
    ValidateDestination,
    /// Copy checked logical rows using the existing capture conversion.
    CopyRows,
}

/// Actual destination metadata at rejection; `layout.row_bytes` reports its stride.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureImageLayout {
    /// Returned pixel format.
    pub format: ImageFormat,
    /// Returned width, height and depth.
    pub dimensions: Vec3<usize>,
    /// Returned bytes per pixel.
    pub pixel_bytes: usize,
    /// Returned bytes per row.
    pub row_bytes: usize,
    /// Returned bytes per slice.
    pub slice_bytes: usize,
    /// Actual backing storage length.
    pub storage_bytes: usize,
    /// Row order: one for top-down or `usize::MAX` for bottom-up.
    pub row_order: usize,
}

/// Typed rejection and its observed values; source type names are not unique image identities.
#[derive(Clone, Debug, PartialEq)]
pub enum CaptureImageFailure {
    /// Empty, overflowing or incomplete capture storage is never admitted as success.
    MalformedCapture {
        /// Declared width and height.
        extent: [u32; 2],
        /// Actual color byte count.
        rgba_bytes: usize,
        /// Actual depth element count.
        depth_values: usize,
    },
    /// The requested buffer has no supported capture conversion.
    UnsupportedBuffer,
    /// Requested row/slice storage cannot be represented by an addressable allocation.
    UnrepresentableLayout {
        /// Rejected destination allocation request.
        request: ImageAllocationRequest,
    },
    /// Original typed source error, preserved without synthesizing a cause.
    ImageSource {
        /// Requested destination allocation.
        request: ImageAllocationRequest,
        /// Exact error supplied by projection or allocation.
        cause: Box<ImageSourceError>,
    },
    /// The source returned an incompatible or incomplete destination layout.
    InvalidDestination {
        /// Allocation requested from the source.
        request: ImageAllocationRequest,
        /// Observed metadata under the image write guard.
        actual: Box<CaptureImageLayout>,
        /// Dynamic source type, not a unique or persistent identity.
        source_type: &'static str,
    },
    /// The existing conversion or row-copy routine refused; it supplies no underlying cause.
    CopyRefused {
        /// Allocation associated with the attempted copy.
        request: ImageAllocationRequest,
        /// Dynamic source type, not a unique or persistent identity.
        source_type: &'static str,
    },
}

/// Context for a failed capture-to-image write; inspect `error.operation` and `error.failure`.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureImageError {
    /// Failing operation step.
    pub operation: CaptureImageOperation,
    /// Requested capture buffer.
    pub buffer: Graphic3dBufferType,
    /// Observed rejection with its original cause when available.
    pub failure: CaptureImageFailure,
}
impl CaptureImageError {
    pub(crate) fn new(
        operation: CaptureImageOperation,
        buffer: Graphic3dBufferType,
        failure: CaptureImageFailure,
    ) -> Self {
        Self {
            operation,
            buffer,
            failure,
        }
    }
}
impl std::fmt::Display for CaptureImageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "capture image operation={:?} buffer={:?} failure={:?}; expected a positive complete capture, supported buffer and writable matching destination rows",
            self.operation, self.buffer, self.failure
        )
    }
}
impl std::error::Error for CaptureImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.failure {
            CaptureImageFailure::ImageSource { cause, .. } => Some(cause.as_ref()),
            _ => None,
        }
    }
}
