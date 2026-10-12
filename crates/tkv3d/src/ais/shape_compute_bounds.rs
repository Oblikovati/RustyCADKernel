//! Bounded genuine AIS_Shape mode2 prerequisite preserves primitive priority, infinite fallback and final ReCompute.
use super::{AisShapeMembers, AisShapeSource, shape_bounds::shape_presentation_bounds_snapshot};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dBndBox, std_prs::StdPrsWfShape};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::standard::{Failure, HandleReference};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::graphic3d::*;
pub(super) fn ais_shape_compute_bounds_mode<S: AisShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let source = object.shape_ref();
    let empty = source.read_current_shape(|shape| {
        Ok(shape.is_null()
            || (shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0))
    })?;
    if empty {
        return Ok(());
    }
    set_shape_bounds_priority(source.read_current_shape(|shape| shape.shape_type())?, prs)?;
    if object.is_infinite() {
        prs.observe().set_infinite_state(true);
    }
    render_shape_bounds_mode(object, prs, &source)?;
    prs.observe().re_compute();
    Ok(())
}
fn render_shape_bounds_mode<S: AisShapeSource + ?Sized>(
    object: &S,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
    source: &impl TopoDSShapeReadSource,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    if object.is_infinite() {
        return StdPrsWfShape::add_reference(prs, source, &drawer);
    }
    let bounds = shape_presentation_bounds_snapshot(object)?;
    Prs3dBndBox::add_box(&prs.observe(), &bounds, &drawer)
}
fn set_shape_bounds_priority(
    kind: ShapeEnum,
    prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let priority = match kind {
        ShapeEnum::Wire => Graphic3dDisplayPriority::Above1,
        ShapeEnum::Edge => Graphic3dDisplayPriority::Above2,
        ShapeEnum::Vertex => Graphic3dDisplayPriority::Highlight,
        _ => return Ok(()),
    };
    prs.observe().set_visual(Graphic3dTypeOfStructure::All);
    prs.observe().set_display_priority(priority)
}
