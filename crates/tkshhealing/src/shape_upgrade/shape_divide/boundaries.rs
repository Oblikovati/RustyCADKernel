use super::ShapeDivider;
use crate::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{FaceDivider, ShapeUpgradeFaceDivide, ShapeUpgradeWireDivide, WireDivider},
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::{self, Explorer},
    topods::{self, Face, Shape},
};
use rustycad_tkernel::{
    message::{Gravity, Msg},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn divide_free_boundaries(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
    face: &Handle<ShapeUpgradeFaceDivide>,
) -> Result<(), Failure> {
    let wire = selected_wire(face)?;
    if wire.is_null() {
        return Ok(());
    }
    configure_wire(tool, &wire);
    divide_boundary_kind(tool, source, &wire, ShapeEnum::Wire)?;
    divide_boundary_kind(tool, source, &wire, ShapeEnum::Edge)
}
fn divide_boundary_kind(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
    wire: &Handle<ShapeUpgradeWireDivide>,
    kind: ShapeEnum,
) -> Result<(), Failure> {
    let (avoid, message) = if kind == ShapeEnum::Wire {
        (ShapeEnum::Face, tool.wire_msg())
    } else {
        (ShapeEnum::Wire, tool.edge_msg())
    };
    divide_boundaries(tool, source, wire, kind, avoid, &message)
}

fn selected_wire(
    face: &Handle<ShapeUpgradeFaceDivide>,
) -> Result<Handle<ShapeUpgradeWireDivide>, Failure> {
    if face.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_ShapeDivide face tool is null; expected a face tool for free-boundary processing",
            "",
        ));
    }
    Ok(face.cast::<dyn FaceDivider>().wire_divide_tool())
}
fn configure_wire(tool: &(impl ShapeDivider + ?Sized), wire: &Handle<ShapeUpgradeWireDivide>) {
    let (precision, max, min, mode) = tool
        .shape_divide_base()
        .with(|f| (f.precision, f.max_tolerance, f.min_tolerance, f.edge_mode));
    wire.set_face(&Face::default());
    wire.set_precision(precision);
    wire.set_max_tolerance(max);
    wire.set_min_tolerance(min);
    wire.set_edge_mode(mode);
}
fn divide_boundaries(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
    wire: &Handle<ShapeUpgradeWireDivide>,
    kind: ShapeEnum,
    avoid: ShapeEnum,
    message: &Msg,
) -> Result<(), Failure> {
    let mut explorer = Explorer::new(source, kind, avoid)?;
    while explorer.more() {
        let original = explorer.current().oriented(Orientation::Forward);
        if eligible_boundary(&original, kind)? {
            divide_applied_boundaries(tool, &original, wire, kind, message)?;
        }
        explorer.next()?;
    }
    Ok(())
}
fn eligible_boundary(shape: &Shape, kind: ShapeEnum) -> Result<bool, Failure> {
    if kind != ShapeEnum::Edge {
        return Ok(true);
    }
    let (first, last) = topexp::vertices(&topods::edge(shape)?, false);
    // OCC5624 skips an edge only when both endpoint vertices are absent.
    Ok(!first.is_null() || !last.is_null())
}
fn divide_applied_boundaries(
    tool: &(impl ShapeDivider + ?Sized),
    original: &Shape,
    wire: &Handle<ShapeUpgradeWireDivide>,
    kind: ShapeEnum,
    message: &Msg,
) -> Result<(), Failure> {
    let applied = tool
        .shape_divide_base()
        .context()
        .apply(original, ShapeEnum::Shape)?;
    let mut children = Explorer::new(&applied, kind, ShapeEnum::Shape)?;
    while children.more() {
        let shape = children.current();
        perform_boundary(tool, wire, shape, kind)?;
        record_boundary(tool, wire, shape, message)?;
        children.next()?;
    }
    Ok(())
}
fn perform_boundary(
    tool: &(impl ShapeDivider + ?Sized),
    wire: &Handle<ShapeUpgradeWireDivide>,
    shape: &Shape,
    kind: ShapeEnum,
) -> Result<(), Failure> {
    if kind == ShapeEnum::Wire {
        wire.load_wire(&topods::wire(shape)?);
    } else {
        wire.load_edge(&topods::edge(shape)?)?;
    }
    wire.set_context(&tool.shape_divide_base().context());
    wire.cast::<dyn WireDivider>().perform()
}
fn record_boundary(
    tool: &(impl ShapeDivider + ?Sized),
    wire: &Handle<ShapeUpgradeWireDivide>,
    shape: &Shape,
    message: &Msg,
) -> Result<(), Failure> {
    let base = tool.shape_divide_base();
    if wire.status_flag(ShapeExtendStatus::Fail) {
        base.add_status(ShapeExtendStatus::Fail3);
    }
    if !wire.status_flag(ShapeExtendStatus::Done) {
        return Ok(());
    }
    base.context().replace(shape, &wire.wire().into_shape())?;
    base.send_msg(shape, message, Gravity::Info);
    base.add_status(ShapeExtendStatus::Done1);
    Ok(())
}
