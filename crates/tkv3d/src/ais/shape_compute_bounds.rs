//! Bounded genuine AIS_Shape mode2 prerequisite preserves primitive priority, infinite fallback and final ReCompute.
use super::{AisShapeMembers, AisShapeSource, shape_bounds::shape_presentation_bounds_snapshot};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dBndBox, std_prs::StdPrsWfShape};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkservice::graphic3d::*;
pub(super) fn ais_shape_compute_bounds_mode<S: AisShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
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
        prs.set_infinite_state(true);
    }
    let drawer = object.attributes();
    if object.is_infinite() {
        StdPrsWfShape::add_reference(prs, &source, &drawer)?;
    } else {
        Prs3dBndBox::add_box(prs, &shape_presentation_bounds_snapshot(object)?, &drawer)?;
    }
    prs.re_compute();
    Ok(())
}
fn set_shape_bounds_priority(
    kind: ShapeEnum,
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<(), Failure> {
    let priority = match kind {
        ShapeEnum::Wire => Graphic3dDisplayPriority::Above1,
        ShapeEnum::Edge => Graphic3dDisplayPriority::Above2,
        ShapeEnum::Vertex => Graphic3dDisplayPriority::Highlight,
        _ => return Ok(()),
    };
    prs.set_visual(Graphic3dTypeOfStructure::All);
    prs.set_display_priority(priority)
}
