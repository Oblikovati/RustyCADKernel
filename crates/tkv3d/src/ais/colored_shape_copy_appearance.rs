//! Native copy constructor transfers material name first, then explicit color, width and transparency.
use super::{AisColoredShape, AisShapeSource};
use crate::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::standard::{Failure, HandleReference};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
pub(super) fn copy_colored_shape_appearance(
    target: &AisColoredShape,
    source: &dyn HandleReference<dyn AisShapeSource>,
) -> Result<(), Failure> {
    // OCCT 3d097a0 AIS_ColoredShape.cxx75–101 rereads the original slot after virtual getters.
    copy_source_material(target, source)?;
    copy_source_color(target, source)?;
    copy_source_width(target, source)?;
    copy_source_transparency(target, source)
}
fn copy_source_material(
    target: &AisColoredShape,
    source: &dyn HandleReference<dyn AisShapeSource>,
) -> Result<(), Failure> {
    if !source.observe().has_material() {
        return Ok(());
    }
    target.set_material(&Graphic3dMaterialAspect::new(source.observe().material()))
}
fn copy_source_color(
    target: &AisColoredShape,
    source: &dyn HandleReference<dyn AisShapeSource>,
) -> Result<(), Failure> {
    if !source.observe().has_color() {
        return Ok(());
    }
    target.set_color(source.observe().color())
}
fn copy_source_width(
    target: &AisColoredShape,
    source: &dyn HandleReference<dyn AisShapeSource>,
) -> Result<(), Failure> {
    if !source.observe().has_width() {
        return Ok(());
    }
    target.set_width(source.observe().width())
}
fn copy_source_transparency(
    target: &AisColoredShape,
    source: &dyn HandleReference<dyn AisShapeSource>,
) -> Result<(), Failure> {
    if !source.observe().is_transparent() {
        return Ok(());
    }
    target.set_transparency(source.observe().transparency())
}
