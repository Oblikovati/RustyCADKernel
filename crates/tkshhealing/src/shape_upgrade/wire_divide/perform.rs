//! Context application, edge dispatch and final wire replacement preserve native ordering.
use super::*;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Shape, ShapeIterator},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) struct WireSupport {
    pub face: Face,
    pub surface: Handle<dyn GeomSurface>,
    pub location: Location,
    pub context: Handle<ShapeBuildReShape>,
    pub criterion: Handle<ShapeUpgradeEdgeDivide>,
}
pub(super) fn perform<T: WireDivider + ?Sized>(tool: &T) -> Result<(), Failure> {
    tool.wire_divide_data().with_mut(|fields| fields.status = 0);
    let (wire, face) = tool
        .wire_divide_data()
        .with(|fields| (fields.wire.clone(), fields.face.clone()));
    let mut result = Wire::default();
    BRepBuilder::default().make_wire(&mut result);
    let mut support = prepare_support(tool, face)?;
    for child in ShapeIterator::new(&wire, false, true) {
        state::require(&support.context, "reshape context")?;
        let applied = support.context.apply(&child, ShapeEnum::Shape)?;
        divide_child(tool, &mut support, &applied, &mut result)?;
    }
    finish_wire(tool, &wire, &mut result, &support)
}
fn prepare_support<T: WireDivider + ?Sized>(tool: &T, face: Face) -> Result<WireSupport, Failure> {
    let (surface, location) = if face.is_null() {
        (Handle::default(), Location::default())
    } else {
        BRepTool::surface_with_location(&face)?
    };
    let (criterion, mode) = tool
        .wire_divide_data()
        .with(|fields| (fields.edge_divide.clone(), fields.edge_mode));
    state::require(&criterion, "edge criterion")?;
    criterion.set_face(&face);
    configure_criterion(tool, &criterion, &face, mode);
    Ok(WireSupport {
        face,
        surface,
        location,
        context: tool.wire_divide_base().context(),
        criterion,
    })
}
fn configure_criterion<T: WireDivider + ?Sized>(
    tool: &T,
    criterion: &Handle<ShapeUpgradeEdgeDivide>,
    face: &Face,
    mode: i32,
) {
    let split3 = match mode {
        0 => face.is_null(),
        1 => !face.is_null(),
        _ => true,
    };
    // Native inactive modes retain the edge criterion's previously configured 3D splitter.
    if split3 {
        criterion.set_split_curve_3d_tool(&tool.split_curve_3d_tool());
    }
    criterion.set_split_curve_2d_tool(&tool.split_curve_2d_tool());
}
fn divide_child<T: WireDivider + ?Sized>(
    tool: &T,
    support: &mut WireSupport,
    shape: &Shape,
    result: &mut Wire,
) -> Result<(), Failure> {
    for found in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&found)?;
        if matches!(
            edge.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            BRepBuilder::default().add(result, &edge)?;
            continue;
        }
        if !support.criterion.cast::<dyn EdgeDivider>().compute(&edge)? {
            BRepBuilder::default().add(result, &edge)?;
            continue;
        }
        divide_edge(tool, support, &edge, result)?;
    }
    Ok(())
}
fn divide_edge<T: WireDivider + ?Sized>(
    tool: &T,
    support: &mut WireSupport,
    edge: &Edge,
    result: &mut Wire,
) -> Result<(), Failure> {
    let curves = segments::build(tool, edge, &support.face, &support.criterion)?;
    let vertices = vertices::EdgeVertices::prepare(tool, support, edge, &curves)?;
    if !curves.is_done()? {
        return BRepBuilder::default().add(result, edge);
    }
    tool.wire_divide_data()
        .with_mut(|fields| fields.status |= 1);
    pieces::divide(tool, support, edge, &curves, vertices, result)
}
fn finish_wire<T: WireDivider + ?Sized>(
    tool: &T,
    original: &Wire,
    result: &mut Wire,
    support: &WireSupport,
) -> Result<(), Failure> {
    let done = tool
        .wire_divide_data()
        .with(|fields| ShapeExtend::decode_status(fields.status, ShapeExtendStatus::Done));
    if !done {
        return Ok(());
    }
    result.set_closed(BRepTool::is_closed(result)?)?;
    state::require(&support.context, "reshape context")?;
    let applied = support
        .context
        .apply(result, ShapeEnum::Shape)?
        .oriented(original.orientation());
    let wire = topods::wire(&applied)?;
    tool.wire_divide_data()
        .with_mut(|fields| fields.wire = wire);
    Ok(())
}
