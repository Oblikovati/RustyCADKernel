use super::ShapeDivider;
use crate::{
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{FaceDivider, ShapeUpgradeFaceDivide},
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Shape},
};
use rustycad_tkernel::{
    message::{Gravity, Msg},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn perform(
    tool: &(impl ShapeDivider + ?Sized),
    new_context: bool,
) -> Result<bool, Failure> {
    let base = tool.shape_divide_base();
    base.with_mut(|f| f.status = 0);
    let source = base.with(|f| f.shape.clone());
    if source.is_null() {
        base.add_status(ShapeExtendStatus::Fail1);
        return Ok(false);
    }
    if new_context || base.context().is_null() {
        base.set_context(&Handle::new(ShapeBuildReShape::new()));
    }
    if source.shape_type()? == ShapeEnum::Compound {
        return super::compound::perform_compound(tool, &source);
    }
    divide_noncompound(tool, &source)
}
fn divide_noncompound(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
) -> Result<bool, Failure> {
    let base = tool.shape_divide_base();
    let face_tool = tool.split_face_tool();
    if !face_tool.is_null() {
        divide_faces(tool, source, &face_tool)?;
    }
    super::boundaries::divide_free_boundaries(tool, source, &face_tool)?;
    let result = base.context().apply(source, ShapeEnum::Shape)?;
    let changed = !result.is_same(source);
    base.with_mut(|f| f.result = result);
    Ok(changed)
}
fn divide_faces(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
    face_tool: &Handle<ShapeUpgradeFaceDivide>,
) -> Result<(), Failure> {
    configure_face(tool, face_tool);
    let message = tool.face_msg();
    let mut faces = Explorer::new(source, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let original = faces.current().oriented(Orientation::Forward);
        let applied = tool
            .shape_divide_base()
            .context()
            .apply(&original, ShapeEnum::Shape)?;
        divide_applied_faces(tool, face_tool, &applied, &message)?;
        faces.next()?;
    }
    Ok(())
}
fn configure_face(tool: &(impl ShapeDivider + ?Sized), face: &Handle<ShapeUpgradeFaceDivide>) {
    let (precision, max, min, segment, edge_mode) = tool.shape_divide_base().with(|f| {
        (
            f.precision,
            f.max_tolerance,
            f.min_tolerance,
            f.segment_mode,
            f.edge_mode,
        )
    });
    face.set_precision(precision);
    face.set_max_tolerance(max);
    face.set_surface_segment_mode(segment);
    let wire = face.cast::<dyn FaceDivider>().wire_divide_tool();
    if !wire.is_null() {
        wire.set_min_tolerance(min);
        wire.set_edge_mode(edge_mode);
    }
}
fn divide_applied_faces(
    tool: &(impl ShapeDivider + ?Sized),
    face_tool: &Handle<ShapeUpgradeFaceDivide>,
    applied: &Shape,
    message: &Msg,
) -> Result<(), Failure> {
    let mut faces = Explorer::new(applied, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        // Native catches Standard_Failure around each face and advances to the next.
        if divide_face(tool, face_tool, faces.current(), message).is_err() {
            tool.shape_divide_base()
                .add_status(ShapeExtendStatus::Fail2);
        }
        faces.next()?;
    }
    Ok(())
}
fn divide_face(
    tool: &(impl ShapeDivider + ?Sized),
    face_tool: &Handle<ShapeUpgradeFaceDivide>,
    source: &Shape,
    message: &Msg,
) -> Result<(), Failure> {
    let base = tool.shape_divide_base();
    face_tool.init(&topods::face(source)?);
    face_tool.set_context(&base.context());
    face_tool.cast::<dyn FaceDivider>().perform(0.)?;
    record_face(tool, face_tool, source, message)
}
fn record_face(
    tool: &(impl ShapeDivider + ?Sized),
    face_tool: &Handle<ShapeUpgradeFaceDivide>,
    source: &Shape,
    message: &Msg,
) -> Result<(), Failure> {
    let base = tool.shape_divide_base();
    if face_tool.status_flag(ShapeExtendStatus::Fail) {
        base.add_status(ShapeExtendStatus::Fail2);
    }
    if !face_tool.status_flag(ShapeExtendStatus::Done) {
        return Ok(());
    }
    base.context().replace(source, &face_tool.result())?;
    base.send_msg(source, message, Gravity::Info);
    for flag in [ShapeExtendStatus::Done1, ShapeExtendStatus::Done2] {
        if face_tool.status_flag(flag) {
            base.add_status(flag);
        }
    }
    Ok(())
}
