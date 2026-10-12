//! Native copy constructor transfers material name first, then explicit color, width and transparency.
use super::{AisColoredShape, AisShapeSource};
use crate::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
pub(super) fn copy_colored_shape_appearance<S: AisShapeSource + ?Sized>(
    target: &AisColoredShape,
    source: &S,
) -> Result<(), Failure> {
    if source.has_material() {
        target.set_material(&Graphic3dMaterialAspect::new(source.material()))?;
    }
    if source.has_color() {
        target.set_color(source.color())?;
    }
    if source.has_width() {
        target.set_width(source.width())?;
    }
    if source.is_transparent() {
        target.set_transparency(source.transparency())?;
    }
    Ok(())
}
