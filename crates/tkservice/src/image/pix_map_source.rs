//! Fallible image builder hooks borrow registered storage through ordinary Rust references.
use super::{ImageAlienPixMap, ImageAllocationRequest, ImagePixMap, ImageSourceError};
use super::{ImageSourceFailureClass, ImageSourceOperation};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped, Transient};
use std::any::TypeId;

/// Mutable image builder dispatch; `source.allocate_image(request)?`.
///
/// This adapter does not publish geometry or certify it. Implementations retain the
/// existing `Transient` Send/Sync bounds and image storage synchronization.
pub trait ImagePixMapSource: Transient {
    /// Allocate the requested image through its derived hooks; `source.allocate_image(request)?`.
    fn allocate_image(&self, request: ImageAllocationRequest) -> Result<(), ImageSourceError> {
        image_pix_map_allocate(self, request)
    }
    /// Clear the image through its derived hook; `source.clear_image()?`.
    fn clear_image(&self) -> Result<(), ImageSourceError> {
        image_pix_map_clear(self)
    }
}
impl RuntimeTyped for dyn ImagePixMapSource {
    const TYPE_NAME: &'static str = "Image_PixMap";
    fn static_type() -> &'static RuntimeType {
        ImagePixMap::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
impl ImagePixMapSource for ImagePixMap {}
impl ImagePixMapSource for ImageAlienPixMap {
    fn clear_image(&self) -> Result<(), ImageSourceError> {
        // Configured USE_FREEIMAGE=OFF: native Alien Clear has only its qualified base release.
        ImageAlienPixMap::clear(self);
        Ok(())
    }
}
/// Borrow registered storage, or return missing-registration context; `image_pix_map_original_base(source)?.width()`.
///
/// The borrow cannot outlive its source:
/// ```compile_fail
/// use rustycad_tkservice::image::{ImagePixMap, image_pix_map_original_base};
/// let base = {
///     let image = ImagePixMap::default();
///     image_pix_map_original_base(&image)
/// };
/// println!("{}", base.unwrap().width());
/// ```
pub fn image_pix_map_original_base<S: ImagePixMapSource + ?Sized>(
    source: &S,
) -> Result<&ImagePixMap, ImageSourceError> {
    source
        .project(TypeId::of::<ImagePixMap>())
        .and_then(|field| field.downcast_ref())
        .ok_or_else(|| {
            image_source_failure(
                source,
                ImageSourceOperation::Project,
                ImageSourceFailureClass::MissingRegisteredBase,
                None,
            )
        })
}
/// Clear registered base storage without reentering a derived hook; `image_pix_map_clear(source)?`.
pub fn image_pix_map_clear<S: ImagePixMapSource + ?Sized>(
    source: &S,
) -> Result<(), ImageSourceError> {
    let base = image_pix_map_original_base(source).map_err(|mut error| {
        error.operation = ImageSourceOperation::Clear;
        error
    })?;
    base.clear();
    Ok(())
}
/// Allocate registered storage after the derived clear hook; `image_pix_map_allocate(source, request)?`.
///
/// Refusal can leave a cleared image. The existing allocator reports no cause;
/// that error carries `cause: None`. Existing concrete methods retain their behavior.
pub fn image_pix_map_allocate<S: ImagePixMapSource + ?Sized>(
    source: &S,
    request: ImageAllocationRequest,
) -> Result<(), ImageSourceError> {
    let base = allocation_image_base(source, request)?;
    source.clear_image().map_err(|mut error| {
        error.request = Some(request);
        error
    })?;
    if base.init_trash_after_clear(request.format, request.dimensions, request.stride) {
        return Ok(());
    }
    Err(image_source_failure(
        source,
        ImageSourceOperation::Allocate,
        ImageSourceFailureClass::AllocationRefused,
        Some(request),
    ))
}
fn allocation_image_base<S: ImagePixMapSource + ?Sized>(
    source: &S,
    request: ImageAllocationRequest,
) -> Result<&ImagePixMap, ImageSourceError> {
    image_pix_map_original_base(source).map_err(|mut error| {
        error.operation = ImageSourceOperation::Allocate;
        error.request = Some(request);
        error
    })
}
fn image_source_failure<S: ImagePixMapSource + ?Sized>(
    source: &S,
    operation: ImageSourceOperation,
    class: ImageSourceFailureClass,
    request: Option<ImageAllocationRequest>,
) -> ImageSourceError {
    ImageSourceError {
        operation,
        source_type: source.dynamic_type().name(),
        class,
        request,
        cause: None,
    }
}
