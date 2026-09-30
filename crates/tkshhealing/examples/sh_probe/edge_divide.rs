//! Native EdgeDivide histories use actual derived midpoint splitting criteria.
use super::{
    common::{Reader, attempt, end, flag, int, real, show},
    healing_common::flags,
    shapes::{TpShapes, tp_same_index, tp_shapes},
    wire_data::{sh_edge, sh_face},
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkshhealing::shape_upgrade::{
    EdgeDivider, EdgeSplitKnots, ShapeUpgradeEdgeDivide, ShapeUpgradeSplitCurve2d,
    ShapeUpgradeSplitCurve3d, SplitCurve,
};
#[path = "../../tests/divide_support/criteria.rs"]
mod criteria;
use criteria::{Midpoint2d, Midpoint3d};
struct EdgeProbeTools {
    spatial: Handle<ShapeUpgradeSplitCurve3d>,
    planar: Handle<ShapeUpgradeSplitCurve2d>,
}
fn selected_tools(reader: &mut Reader<'_>) -> EdgeProbeTools {
    let spatial = match reader.integer() {
        -1 => Handle::default(),
        0 => Handle::new(ShapeUpgradeSplitCurve3d::new()),
        _ => Handle::new(Midpoint3d::default()).cast(),
    };
    let planar = match reader.integer() {
        -1 => Handle::default(),
        0 => Handle::new(ShapeUpgradeSplitCurve2d::new()),
        _ => Handle::new(Midpoint2d::default()).cast(),
    };
    EdgeProbeTools { spatial, planar }
}
/// Emit a criterion history, e.g. `edge_divide_case(&request)`.
pub fn edge_divide_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| edge_divide_request(&mut reader));
    end();
}
fn edge_divide_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let divider = ShapeUpgradeEdgeDivide::new();
    flag(divider.split_curve_3d_tool().is_null());
    flag(divider.split_curve_2d_tool().is_null());
    let mut tools = selected_tools(reader);
    replace_tools(&divider, &tools);
    divider.clear();
    edge_state(
        &divider,
        &shapes,
        &EdgeSplitKnots::default(),
        &EdgeSplitKnots::default(),
    )?;
    for _ in 0..reader.integer() {
        let previous = (divider.knots_3d(), divider.knots_2d());
        attempt(|| edge_operation(&divider, &shapes, &mut tools, reader));
        edge_state(&divider, &shapes, &previous.0, &previous.1)?;
    }
    Ok(())
}
fn replace_tools(divider: &ShapeUpgradeEdgeDivide, tools: &EdgeProbeTools) {
    divider.set_split_curve_3d_tool(&tools.spatial);
    divider.set_split_curve_2d_tool(&tools.planar);
    flag(divider.split_curve_3d_tool() == tools.spatial);
    flag(divider.split_curve_2d_tool() == tools.planar);
}
fn edge_operation(
    divider: &ShapeUpgradeEdgeDivide,
    shapes: &TpShapes,
    tools: &mut EdgeProbeTools,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    match reader.integer() {
        0 => divider.set_face(&sh_face(shapes, reader.integer())?),
        1 => show(divider.compute(&sh_edge(shapes, reader.integer())?), flag),
        2 => divider.clear(),
        3 => set_tool_status(tools, reader),
        4 => change_knots(divider, reader)?,
        5 => {
            *tools = selected_tools(reader);
            replace_tools(divider, tools);
        }
        _ => build_tools(tools, reader.flag()),
    }
    Ok(())
}
fn set_tool_status(tools: &EdgeProbeTools, reader: &mut Reader<'_>) {
    let spatial = reader.flag();
    let status = reader.integer();
    selected_criterion(tools, spatial)
        .split_curve_data()
        .with_mut(|f| f.status = status);
}
fn build_tools(tools: &EdgeProbeTools, segment: bool) {
    show(
        tools.spatial.cast::<dyn SplitCurve>().build(segment),
        |_| {},
    );
    show(tools.planar.cast::<dyn SplitCurve>().build(segment), |_| {});
}
fn selected_criterion(tools: &EdgeProbeTools, spatial: bool) -> Handle<dyn SplitCurve> {
    if spatial {
        return tools.spatial.cast();
    }
    tools.planar.cast()
}
fn change_knots(divider: &ShapeUpgradeEdgeDivide, reader: &mut Reader<'_>) -> Result<(), Failure> {
    let values = if reader.flag() {
        divider.knots_3d()
    } else {
        divider.knots_2d()
    };
    let index = reader.integer() as usize;
    let value = reader.real();
    values.change_sequence(|s| s.set_value(index, value))
}
fn print_knots(knots: &Handle<HSequence<f64>>) {
    flag(knots.is_null());
    let Some(knots) = knots.get() else {
        return;
    };
    let values = knots.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    int(values.len() as i32);
    values.into_iter().for_each(real);
}
fn edge_state(
    divider: &ShapeUpgradeEdgeDivide,
    shapes: &TpShapes,
    previous3: &EdgeSplitKnots,
    previous2: &EdgeSplitKnots,
) -> Result<(), Failure> {
    divider
        .edge_divide_data()
        .with(|f| int(tp_same_index(shapes, &f.face)));
    flag(divider.has_curve_3d());
    flag(divider.has_curve_2d());
    for (current, previous) in [
        (divider.knots_3d(), previous3),
        (divider.knots_2d(), previous2),
    ] {
        flag(current == *previous);
        print_knots(&current);
        print_knots(previous);
    }
    let spatial = divider.split_curve_3d_tool();
    let planar = divider.split_curve_2d_tool();
    print_tool(&spatial.cast(), &divider.knots_3d());
    print_tool(&planar.cast(), &divider.knots_2d());
    print_piece_geometry(&spatial, &planar)
}
fn print_piece_geometry(
    spatial: &Handle<ShapeUpgradeSplitCurve3d>,
    planar: &Handle<ShapeUpgradeSplitCurve2d>,
) -> Result<(), Failure> {
    if !spatial.is_null() {
        super::split_curve_3d::print_curves(&spatial.curves())?;
    }
    if !planar.is_null() {
        super::split_curve_2d::print_curves(&planar.curves())?;
    }
    Ok(())
}
fn print_tool(tool: &Handle<dyn SplitCurve>, knots: &EdgeSplitKnots) {
    flag(tool.is_null());
    if tool.is_null() {
        return;
    }
    flag(tool.split_values() == *knots);
    print_knots(&tool.split_values());
    flags(|status| tool.status_flag(status));
}
