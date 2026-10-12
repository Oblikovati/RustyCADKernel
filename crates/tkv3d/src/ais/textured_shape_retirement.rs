//! Actual source mutations within a lifetime-bounded destructor reborrow; OCCT 3d097a0.
use super::{textured_shape_source::AisTexturedShapeSource, textured_shape_sources::*};
use rustycad_tkernel::{standard::*, tcollection::AsciiString};
use rustycad_tkservice::image::ImagePixMap;
/// Native base source operations with a genuine destructor borrow; `scope.set_texture_pix_map(&next)?`.
/// This adapter borrows the original source and real pointer slot; it has no separate object identity.
pub struct AisTexturedShapeRetirement<'a, S: AisTexturedShapeSource + ?Sized> {
    source: &'a S,
    slot: IntrusiveRetirementBorrow<'a, ImagePixMap>,
}
/// Lend actual source retirement transport; `ais_textured_shape_with_retirement(&owner,self,|scope|scope.set_texture_file_name(&file))?`.
/// The callback cannot return the scope or its pointee references; virtual override routing stays open.
pub fn ais_textured_shape_with_retirement<S: AisTexturedShapeSource + ?Sized, R>(
    source: &S,
    retiring: &dyn Transient,
    inspect: impl FnOnce(AisTexturedShapeRetirement<'_, S>) -> R,
) -> Result<R, Failure> {
    source
        .textured_shape_base()
        .texture_pix_map
        .with_retiring_reference(retiring, |slot| {
            inspect(AisTexturedShapeRetirement { source, slot })
        })
}
impl<S: AisTexturedShapeSource + ?Sized> AisTexturedShapeRetirement<'_, S> {
    /// Preserve native Nullify-before-filename order; `scope.set_texture_file_name(&file)?`.
    pub fn set_texture_file_name(&self, file: &AsciiString) -> Result<(), Failure> {
        self.slot.clear()?;
        assign_textured_file_after_retirement(self.source, file)
    }
    /// Preserve native filename-reset-before-Assign order; `scope.set_texture_pix_map(&next)?`.
    pub fn set_texture_pix_map(
        &self,
        pixels: &dyn HandleReference<ImagePixMap>,
    ) -> Result<(), Failure> {
        reset_textured_file_for_pixmap(self.source);
        self.slot.assign(pixels)
    }
}
