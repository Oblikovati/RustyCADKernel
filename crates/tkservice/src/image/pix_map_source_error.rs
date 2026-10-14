//! Request and failure context for mutable image builder dispatch.
use super::ImageFormat;
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::Failure;

/// Image allocation inputs; use depth one for a 2D image.
/// Example: `ImageAllocationRequest { format: ImageFormat::Gray, dimensions: Vec3::new(2, 3, 1), stride: 0 }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageAllocationRequest {
    /// Pixel representation requested from the builder.
    pub format: ImageFormat,
    /// Width, height and depth; zero dimensions retain concrete allocation refusal.
    pub dimensions: Vec3<usize>,
    /// Requested row stride; the existing allocator can raise it to the pixel width.
    pub stride: usize,
}

/// Builder step that failed; `ImageSourceOperation::Allocate`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageSourceOperation {
    /// Borrow registered image storage.
    Project,
    /// Clear the mutable image.
    Clear,
    /// Allocate the requested image.
    Allocate,
}

/// Failure category independent of its optional underlying cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageSourceFailureClass {
    /// The source does not expose a registered image base.
    MissingRegisteredBase,
    /// The allocator or derived allocation hook refused the request.
    AllocationRefused,
    /// A derived callback returned a failure.
    CallbackFailed,
}

/// Context for a failed image builder step; `error.request` retains allocation inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageSourceError {
    /// Failing step, including a clear callback during allocation.
    pub operation: ImageSourceOperation,
    /// Dynamic type of the affected source; this is not a unique image identity.
    pub source_type: &'static str,
    /// Stable failure category.
    pub class: ImageSourceFailureClass,
    /// Allocation inputs, when failure occurred during allocation.
    pub request: Option<ImageAllocationRequest>,
    /// Original callback failure when available; `None` means no cause was supplied.
    pub cause: Option<Box<Failure>>,
}
impl std::fmt::Display for ImageSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "image source={} operation={:?} request={:?} class={:?} cause={:?}; expected registered image and successful builder callback",
            self.source_type, self.operation, self.request, self.class, self.cause
        )
    }
}
impl std::error::Error for ImageSourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &dyn std::error::Error)
    }
}
