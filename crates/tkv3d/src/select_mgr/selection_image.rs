//! Borrowed image-filler interface and deterministic native color generation.
use super::selection_image_pixels::SelectionImagePixels;
use super::*;
use crate::std_select::StdSelectTypeOfSelectionImage;
use rustycad_tkernel::{
    ncollection::Map,
    quantity::{Color, ColorSpace},
    standard::Failure,
};
use rustycad_tkmath::math::BullardGenerator;
use rustycad_tkservice::image::ImagePixMap;

/// Fill pixels from the selector's current results; `filler.fill(0, 0, 1)?`.
pub trait SelectMgrSelectionImageFiller {
    /// Write one pixel using a one-based picked rank; `filler.fill(0, 0, 1)?`.
    fn fill(&mut self, col: i32, row: i32, picked: i32) -> Result<(), Failure>;
    /// Finish deferred pixel processing; `filler.flush()?`.
    fn flush(&mut self) -> Result<(), Failure> {
        Ok(())
    }
}
/// Shared borrowed inputs and color state for custom/native fillers.
/// Example: `SelectMgrSelectionImageFillerBase::new(&image, &selector)`.
pub struct SelectMgrSelectionImageFillerBase<'a, S: SelectMgrViewerSelectorSource + ?Sized> {
    /// Image supplied by the caller; no image guard spans a selector callback.
    pub image: &'a ImagePixMap,
    /// Selector whose current results are read at each Fill.
    pub selector: &'a S,
    /// Generator available to custom fillers, matching the native protected state.
    pub generator: BullardGenerator,
    /// Palette uniqueness set available to custom fillers.
    pub unique_colors: Map<Color>,
}
impl<'a, S: SelectMgrViewerSelectorSource + ?Sized> SelectMgrSelectionImageFillerBase<'a, S> {
    /// Borrow the image and selector, with native generator seed one.
    /// Example: `SelectMgrSelectionImageFillerBase::new(&image, &selector)`.
    pub fn new(image: &'a ImagePixMap, selector: &'a S) -> Self {
        Self {
            image,
            selector,
            generator: BullardGenerator::default(),
            unique_colors: Map::default(),
        }
    }
    /// Produce a unique color using native hash/equality semantics.
    /// Example: `let color = base.random_pastel_color()?`.
    pub fn random_pastel_color(&mut self) -> Result<Color, Failure> {
        loop {
            let color = self.next_random_pastel_color()?;
            if self.unique_colors.add(color) {
                return Ok(color);
            }
        }
    }
    /// Consume three draws in the pinned GCC constructor-argument order.
    /// Example: `let color = base.next_random_pastel_color()?`.
    pub fn next_random_pastel_color(&mut self) -> Result<Color, Failure> {
        // OCCT 3d097a0 uses three side-effecting arguments; the pinned native GCC
        // profile evaluates blue before green before red. The oracle checks this.
        let blue = f64::from(self.generator.next_int() % 256) / 255.;
        let green = f64::from(self.generator.next_int() % 256) / 255.;
        let red = f64::from(self.generator.next_int() % 256) / 255.;
        Color::new(red, green, blue, ColorSpace::Srgb)
    }
}
/// Construct the selected native filler; unknown kinds return None.
/// Example: `select_mgr_create_selection_image_filler(&image, &selector, kind)?`.
pub fn select_mgr_create_selection_image_filler<'a, S: SelectMgrViewerSelectorSource + ?Sized>(
    image: &'a ImagePixMap,
    selector: &'a S,
    kind: StdSelectTypeOfSelectionImage,
) -> Result<Option<Box<dyn SelectMgrSelectionImageFiller + 'a>>, Failure> {
    if !(0..=8).contains(&kind.0) {
        return Ok(None);
    }
    Ok(Some(Box::new(SelectionImagePixels::new(
        image, selector, kind,
    )?)))
}
