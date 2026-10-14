//! Named mutable image builder with independently observable hooks.
use super::{ImageAllocationRequest, ImageFormat, ImagePixMap, ImagePixMapSource};
use super::{ImageSourceError, ImageSourceFailureClass, ImageSourceOperation};
use super::{image_pix_map_allocate, image_pix_map_clear};
use rustycad_tkernel::standard::Failure;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub(super) struct ImageAllocationProbe {
    pub base: ImagePixMap,
    pub alternate: ImagePixMap,
    pub clears: AtomicUsize,
    pub allocations: AtomicUsize,
    pub decline_allocation: AtomicBool,
    pub fail_clear: AtomicBool,
    pub clear_failure: Failure,
}
rustycad_tkernel::impl_transient!(ImageAllocationProbe, "Martensite_ImageAllocationProbe",
    ImagePixMap, base, interfaces:[dyn ImagePixMapSource]);
impl ImagePixMapSource for ImageAllocationProbe {
    fn clear_image(&self) -> Result<(), ImageSourceError> {
        self.clears.fetch_add(1, Ordering::Relaxed);
        if self.fail_clear.load(Ordering::Relaxed) {
            return Err(ImageSourceError {
                operation: ImageSourceOperation::Clear,
                source_type: "Martensite_ImageAllocationProbe",
                class: ImageSourceFailureClass::CallbackFailed,
                request: None,
                cause: Some(Box::new(self.clear_failure.clone())),
            });
        }
        image_pix_map_clear(self)
    }
    fn allocate_image(&self, request: ImageAllocationRequest) -> Result<(), ImageSourceError> {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        if self.decline_allocation.load(Ordering::Relaxed) {
            return Err(ImageSourceError {
                operation: ImageSourceOperation::Allocate,
                source_type: "Martensite_ImageAllocationProbe",
                class: ImageSourceFailureClass::AllocationRefused,
                request: Some(request),
                cause: None,
            });
        }
        image_pix_map_allocate(self, request)
    }
}

pub(super) fn image_allocation_probe() -> ImageAllocationProbe {
    let base = ImagePixMap::default();
    let alternate = ImagePixMap::default();
    assert!(base.init_zero(ImageFormat::Gray, 4, 3, 0, 17));
    assert!(alternate.init_zero(ImageFormat::Gray, 2, 2, 0, 131));
    ImageAllocationProbe {
        base,
        alternate,
        clears: AtomicUsize::new(0),
        allocations: AtomicUsize::new(0),
        decline_allocation: AtomicBool::new(false),
        fail_clear: AtomicBool::new(false),
        clear_failure: Failure::with_type(
            "Image_ClearFailure",
            "clear callback rejected image=probe",
            "owned-stack",
        ),
    }
}

pub(super) struct MissingOriginalImage {
    pub base: rustycad_tkernel::standard::StandardTransient,
}
rustycad_tkernel::impl_transient!(MissingOriginalImage, "Martensite_MissingOriginalImage",
    rustycad_tkernel::standard::StandardTransient, base, interfaces:[dyn ImagePixMapSource]);
impl ImagePixMapSource for MissingOriginalImage {}
