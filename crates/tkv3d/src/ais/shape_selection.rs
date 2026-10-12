//! Native AIS_Shape selection protects low-layer failures with a mode-zero bounding-box owner.
use super::{AisShapeMembers, AisShapeSource};
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers,
    select_mgr::*,
    select3d::Select3DSensitiveBox,
    std_prs::StdPrsToolTriangulatedShape,
    std_select::{StdSelectBRepOwner, StdSelectBrepSelectionTool},
};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::{
    message::{Message, Messenger},
    standard::{Failure, Handle, catch_native_failure},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Convert native selection modes, retaining Shape for unknown integers; `ais_shape_selection_type(4)`.
pub fn ais_shape_selection_type(mode: i32) -> ShapeEnum {
    match mode {
        1 => ShapeEnum::Vertex,
        2 => ShapeEnum::Edge,
        3 => ShapeEnum::Wire,
        4 => ShapeEnum::Face,
        5 => ShapeEnum::Shell,
        6 => ShapeEnum::Solid,
        7 => ShapeEnum::CompSolid,
        8 => ShapeEnum::Compound,
        _ => ShapeEnum::Shape,
    }
}
/// Invoke the actual inherited native body; `ais_shape_compute_selection(object,&selection,0)?`.
pub fn ais_shape_compute_selection<S: AisShapeSource + ?Sized>(
    object: &S,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let messenger = Message::default_messenger();
    ais_shape_compute_selection_with_messenger(object, selection, mode, messenger)
}
/// Inject message delivery while preserving native construction and catch ordering; `ais_shape_compute_selection_with_messenger(object,&selection,0,&messenger)?`.
pub fn ais_shape_compute_selection_with_messenger<S: AisShapeSource + ?Sized>(
    object: &S,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
    messenger: &Messenger,
) -> Result<(), Failure> {
    let source = object.shape_ref();
    if source.read_current_shape(|shape| {
        Ok(shape.is_null()
            || (shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0))
    })? {
        return Ok(());
    }
    let kind = ais_shape_selection_type(mode);
    let shape = source.clone_shape();
    let drawer = object.attributes();
    let deflection = StdPrsToolTriangulatedShape::get_deflection(&shape, &drawer)?;
    let selectable = object.presentable_base().this().cast();
    let result = catch_native_failure(|| {
        StdSelectBrepSelectionTool::load_with_selectable(
            selection,
            &selectable,
            &shape,
            kind,
            deflection,
            drawer.deviation_angle(),
            drawer.is_auto_triangulation(),
            -1,
            9,
            500.,
        )
    });
    if let Err(failure) = result {
        report_shape_selection_failure(messenger, mode, &failure);
        if mode == 0 {
            add_shape_fallback_selection(object, selection, &shape, &selectable)?;
        }
    }
    // OCCT 3d097a0 StdSelect::SetDrawerForBRepOwner has an entirely commented-out body.
    Ok(())
}
fn report_shape_selection_failure(messenger: &Messenger, mode: i32, failure: &Failure) {
    let text = format!(
        "Error: AIS_Shape::ComputeSelection({mode}) has failed ({})",
        failure.message()
    );
    let text = AsciiString::from_bytes(text.as_bytes());
    messenger.send_fail_text(&text);
}
fn add_shape_fallback_selection<S: AisShapeSource + ?Sized>(
    object: &S,
    selection: &Handle<SelectMgrSelection>,
    shape: &rustycad_tkbrep::topods::Shape,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<(), Failure> {
    selection.clear();
    let bounds = super::shape_bounds::shape_presentation_bounds_snapshot(object)?;
    let owner = StdSelectBRepOwner::with_selectable(shape, selectable, 0, false).into_cast();
    let sensitive = Select3DSensitiveBox::new(&owner, &bounds)?;
    selection.add(&Handle::new(sensitive).cast())
}
