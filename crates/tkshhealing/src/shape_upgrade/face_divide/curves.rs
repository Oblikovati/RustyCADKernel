//! Context-aware boundary wire subdivision after surface composition.
use super::{FaceDivider, ShapeExtendStatus};
use crate::shape_upgrade::{ShapeUpgradeWireDivide, WireDivider};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Face, ShapeIterator},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn split_curves(tool: &(impl FaceDivider + ?Sized)) -> Result<bool, Failure> {
    let wire_tool = tool.wire_divide_tool();
    if wire_tool.is_null() {
        return Ok(false);
    }
    wire_tool.set_max_tolerance(tool.face_divide_base().max_tolerance());
    let result = tool.face_divide_data().with(|fields| fields.result.clone());
    let mut faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let Some(face) = applied_face(tool, faces.current())? else {
            return Ok(false);
        };
        split_face_wires(tool, &wire_tool, &face)?;
        faces.next()?;
    }
    apply_final_context(tool)?;
    Ok(tool.face_divide_data().with(|fields| {
        crate::shape_extend::ShapeExtend::decode_status(fields.status, ShapeExtendStatus::Done)
    }))
}

fn split_face_wires(
    tool: &(impl FaceDivider + ?Sized),
    wire_tool: &Handle<ShapeUpgradeWireDivide>,
    face: &Face,
) -> Result<(), Failure> {
    wire_tool.set_face(face);
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        if child.shape_type()? != ShapeEnum::Wire {
            continue;
        }
        let wire = topods::wire(&child)?;
        wire_tool.load_wire(&wire);
        wire_tool.set_context(&tool.face_divide_base().context());
        let dispatch = wire_tool.cast::<dyn WireDivider>();
        dispatch.perform()?;
        record_wire_result(tool, wire_tool, &child)?;
    }
    Ok(())
}

fn record_wire_result(
    tool: &(impl FaceDivider + ?Sized),
    wire_tool: &ShapeUpgradeWireDivide,
    original: &rustycad_tkbrep::topods::Shape,
) -> Result<(), Failure> {
    if wire_tool.status_flag(ShapeExtendStatus::Fail) {
        tool.face_divide_data().add_status(ShapeExtendStatus::Fail1);
    }
    if wire_tool.status_flag(ShapeExtendStatus::Done) {
        tool.face_divide_data().add_status(ShapeExtendStatus::Done1);
        tool.face_divide_base()
            .context()
            .replace(original, &wire_tool.wire().into_shape())?;
    }
    Ok(())
}

fn apply_final_context(tool: &(impl FaceDivider + ?Sized)) -> Result<(), Failure> {
    let result = tool.face_divide_data().with(|fields| fields.result.clone());
    let applied = tool
        .face_divide_base()
        .context()
        .apply(&result, ShapeEnum::Shape)?;
    tool.face_divide_data()
        .with_mut(|fields| fields.result = applied);
    Ok(())
}

fn applied_face(
    tool: &(impl FaceDivider + ?Sized),
    original: &rustycad_tkbrep::topods::Shape,
) -> Result<Option<Face>, Failure> {
    let applied = tool
        .face_divide_base()
        .context()
        .apply(original, ShapeEnum::Shape)?;
    if applied.is_null() || applied.shape_type()? != ShapeEnum::Face {
        tool.face_divide_data().add_status(ShapeExtendStatus::Fail3);
        return Ok(None);
    }
    Ok(Some(topods::face(&applied)?))
}
