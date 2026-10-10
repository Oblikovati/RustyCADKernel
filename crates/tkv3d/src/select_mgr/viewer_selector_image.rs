//! Render a selection result image through the same public pixel picking path.
use super::*;
use crate::{std_select::StdSelectTypeOfSelectionImage, v3d::V3dView};
use rustycad_tkernel::standard::{Failure, exceptions::program_error};
use rustycad_tkservice::image::ImagePixMap;
/// Selection-image queries; `selector.to_pix_map(&image, &view, kind, 1)?`.
pub trait SelectMgrViewerSelectorImageMembers: SelectMgrViewerSelectorSource {
    /// Pick every pixel in row-major order and then flush the selected image filler.
    /// Example: `selector.to_pix_map(&image, &view, kind, 1)?`.
    fn to_pix_map(
        &self,
        image: &ImagePixMap,
        view: &V3dView,
        kind: StdSelectTypeOfSelectionImage,
        picked: i32,
    ) -> Result<bool, Failure> {
        if image.is_empty() {
            return Err(program_error(
                "ViewerSelector::ToPixMap: image=empty; expected initialized image",
            ));
        }
        let Some(mut filler) = select_mgr_create_selection_image_filler(image, self, kind)? else {
            return Ok(false);
        };
        fill_selection_pixels(self, image, view, picked, filler.as_mut())?;
        filler.flush()?;
        Ok(true)
    }
}
impl<S: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorImageMembers for S {}

fn fill_selection_pixels<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
    image: &ImagePixMap,
    view: &V3dView,
    picked: i32,
    filler: &mut dyn SelectMgrSelectionImageFiller,
) -> Result<(), Failure> {
    let size = [image.width() as i32, image.height() as i32];
    for row in 0..size[1] {
        for col in 0..size[0] {
            selector.pick_point(col, row, view)?;
            filler.fill(col, row, picked)?;
        }
    }
    Ok(())
}
