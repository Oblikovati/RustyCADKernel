//! Named destination callbacks expose returned layouts and typed failures without GPU work.
use rustycad_tkernel::standard::{Failure, RuntimeType, StandardTransient, Transient};
use rustycad_tkservice::image::{
    ImageAllocationRequest, ImageFormat, ImagePixMap, ImagePixMapSource, ImagePixMapState,
    ImageSourceError, ImageSourceFailureClass, ImageSourceOperation, image_pix_map_allocate,
    image_pix_map_clear,
};
use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy)]
pub(super) enum DestinationMode {
    Valid,
    RefuseAllocation,
    RefuseClear,
    WrongFormat,
    WrongWidth,
    WrongDepth,
    WrongPixelBytes,
    WrongStride,
    WrongSliceBytes,
    WrongRowOrder,
    MissingRows,
}
pub(super) struct CaptureDestinationProbe {
    pub base: ImagePixMap,
    pub mode: DestinationMode,
    pub allocations: AtomicUsize,
    pub clears: AtomicUsize,
    pub failure: ImageSourceError,
}
rustycad_tkernel::impl_transient!(CaptureDestinationProbe, "Martensite_CaptureDestinationProbe",
    ImagePixMap, base, interfaces: [dyn ImagePixMapSource]);
impl ImagePixMapSource for CaptureDestinationProbe {
    fn allocate_image(&self, request: ImageAllocationRequest) -> Result<(), ImageSourceError> {
        self.allocations.fetch_add(1, Ordering::Relaxed);
        if matches!(self.mode, DestinationMode::RefuseAllocation) {
            return Err(self.failure.clone());
        }
        image_pix_map_allocate(self, request)?;
        let mut storage = self.base.change_data();
        storage.data.change_data().fill(167);
        alter_returned_capture_layout(&mut storage, self.mode);
        Ok(())
    }
    fn clear_image(&self) -> Result<(), ImageSourceError> {
        self.clears.fetch_add(1, Ordering::Relaxed);
        if matches!(self.mode, DestinationMode::RefuseClear) {
            return Err(self.failure.clone());
        }
        image_pix_map_clear(self)
    }
}
fn alter_returned_capture_layout(storage: &mut ImagePixMapState, mode: DestinationMode) {
    match mode {
        DestinationMode::WrongFormat => storage.format = ImageFormat::Gray,
        DestinationMode::WrongWidth => storage.data.size_x += 1,
        DestinationMode::WrongDepth => storage.data.size_z = 2,
        DestinationMode::WrongPixelBytes => storage.data.size_bpp = 0,
        DestinationMode::WrongStride => storage.data.size_row_bytes = 1,
        DestinationMode::WrongSliceBytes => storage.data.size_slice_bytes = 0,
        DestinationMode::WrongRowOrder => storage.data.top_to_down = 0,
        DestinationMode::MissingRows => {
            storage.data.take_bytes();
        }
        _ => {}
    }
}
pub(super) fn capture_destination_probe(mode: DestinationMode) -> CaptureDestinationProbe {
    let base = ImagePixMap::default();
    assert!(base.init_zero(ImageFormat::Gray, 1, 1, 0, 91));
    CaptureDestinationProbe {
        base,
        mode,
        allocations: AtomicUsize::new(0),
        clears: AtomicUsize::new(0),
        failure: capture_destination_failure(mode),
    }
}
fn capture_destination_failure(mode: DestinationMode) -> ImageSourceError {
    ImageSourceError {
        operation: if matches!(mode, DestinationMode::RefuseClear) {
            ImageSourceOperation::Clear
        } else {
            ImageSourceOperation::Allocate
        },
        source_type: "Martensite_CaptureDestinationProbe",
        class: ImageSourceFailureClass::CallbackFailed,
        request: None,
        cause: Some(Box::new(Failure::with_type(
            "Image_CallbackFailure",
            "image=probe refused callback",
            "owned-stack",
        ))),
    }
}

pub(super) struct CountingImageDestination {
    pub inner: CaptureDestinationProbe,
    pub callbacks: AtomicUsize,
}
impl Transient for CountingImageDestination {
    fn dynamic_type(&self) -> &'static RuntimeType {
        self.callbacks.fetch_add(1, Ordering::Relaxed);
        self.inner.dynamic_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        self.callbacks.fetch_add(1, Ordering::Relaxed);
        self.inner.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        self.callbacks.fetch_add(1, Ordering::Relaxed);
        self.inner.transient_base()
    }
}
impl ImagePixMapSource for CountingImageDestination {
    fn allocate_image(&self, request: ImageAllocationRequest) -> Result<(), ImageSourceError> {
        self.callbacks.fetch_add(1, Ordering::Relaxed);
        self.inner.allocate_image(request)
    }
}
pub(super) fn counting_capture_destination() -> CountingImageDestination {
    CountingImageDestination {
        inner: capture_destination_probe(DestinationMode::Valid),
        callbacks: AtomicUsize::new(0),
    }
}
pub(super) struct MissingCaptureImage {
    pub base: StandardTransient,
}
rustycad_tkernel::impl_transient!(MissingCaptureImage, "Martensite_MissingCaptureImage",
    StandardTransient, base, interfaces: [dyn ImagePixMapSource]);
impl ImagePixMapSource for MissingCaptureImage {}
