//! Built-in filler dispatch; selector result guards end before image writes.
use super::*;
use super::{
    selection_image::*, selection_image_colors::*, selection_image_depth::SelectionImageDepth,
};
use crate::std_select::StdSelectTypeOfSelectionImage;
use rustycad_tkernel::{quantity::ColorRgba, standard::Failure};
use rustycad_tkservice::image::ImagePixMap;
pub(super) struct SelectionImagePixels<'a, S: SelectMgrViewerSelectorSource + ?Sized> {
    base: SelectMgrSelectionImageFillerBase<'a, S>,
    kind: i32,
    colors: SelectionImageColors,
    depth: Option<SelectionImageDepth>,
}
impl<'a, S: SelectMgrViewerSelectorSource + ?Sized> SelectionImagePixels<'a, S> {
    pub fn new(
        image: &'a ImagePixMap,
        selector: &'a S,
        kind: StdSelectTypeOfSelectionImage,
    ) -> Result<Self, Failure> {
        let mut base = SelectMgrSelectionImageFillerBase::new(image, selector);
        let colors = SelectionImageColors::new(kind.0, &mut base)?;
        let depth = (kind.0 < 2).then(|| SelectionImageDepth::new(image, kind.0 == 1));
        Ok(Self {
            base,
            kind: kind.0,
            colors,
            depth,
        })
    }
    fn pixel_color(&mut self, rank: i32) -> Result<ColorRgba, Failure> {
        if rank < 1 || rank > self.base.selector.nb_picked() {
            return Ok(ColorRgba::from_color(black()));
        }
        match self.kind {
            2 => {
                let depth = self.base.selector.picked_data(rank)?.depth as f32;
                ColorRgba::new(depth, depth, depth, 1.)
            }
            3 => Ok(ColorRgba::from_color(detected_object_color(
                self.base.selector,
                rank,
            )?)),
            8 => normal_color(self.base.selector, rank),
            _ => Ok(ColorRgba::from_color(
                self.colors.picked_color(&mut self.base, rank)?,
            )),
        }
    }
}
impl<S: SelectMgrViewerSelectorSource + ?Sized> SelectMgrSelectionImageFiller
    for SelectionImagePixels<'_, S>
{
    fn fill(&mut self, col: i32, row: i32, picked: i32) -> Result<(), Failure> {
        if let Some(depth) = &mut self.depth {
            return depth.fill(self.base.selector, col, row, picked);
        }
        let color = self.pixel_color(picked)?;
        self.base
            .image
            .set_pixel_color_rgba(col, row, &color, false);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Failure> {
        match &self.depth {
            Some(depth) => depth.flush(self.base.image),
            None => Ok(()),
        }
    }
}
fn normal_color<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
    rank: i32,
) -> Result<ColorRgba, Failure> {
    let mut normal = selector.picked_data(rank)?.normal;
    normal.normalize();
    if normal.modulus() > 0. {
        return ColorRgba::new(
            normal.x * 0.5 + 0.5,
            normal.y * 0.5 + 0.5,
            normal.z * 0.5 + 0.5,
            1.,
        );
    }
    Ok(ColorRgba::from_color(black()))
}
