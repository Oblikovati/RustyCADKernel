//! Discovery and virtual Compute calls preserve native mutation order.
use super::super::SplitCurve;
use super::*;
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::BRepTool;
pub(super) fn spatial_knots<T: EdgeDivider + ?Sized>(
    divider: &T,
    edge: &Edge,
) -> Result<Handle<ShapeUpgradeSplitCurve3d>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    divider
        .edge_divide_data()
        .with_mut(|fields| fields.has_curve_3d = !curve.is_null());
    let tool = divider.split_curve_3d_tool();
    if curve.is_null() {
        return Ok(tool);
    }
    require_tool(&tool.cast())?;
    tool.init_curve_range(&curve, first, last)?;
    let criterion = tool.cast::<dyn SplitCurve>();
    criterion.compute()?;
    divider
        .edge_divide_data()
        .with_mut(|fields| fields.knots_3d = criterion.split_values());
    Ok(tool)
}
pub(super) fn planar_knots<T: EdgeDivider + ?Sized>(
    divider: &T,
    edge: &Edge,
) -> Result<Handle<ShapeUpgradeSplitCurve2d>, Failure> {
    let (curve, first, last) = edge_pcurve(divider, edge)?;
    divider
        .edge_divide_data()
        .with_mut(|fields| fields.has_curve_2d = !curve.is_null());
    let tool = divider.split_curve_2d_tool();
    if curve.is_null() {
        return Ok(tool);
    }
    require_tool(&tool.cast())?;
    tool.init_curve_range(&curve, first, last)?;
    let criterion = tool.cast::<dyn SplitCurve>();
    criterion.compute()?;
    divider
        .edge_divide_data()
        .with_mut(|fields| fields.knots_2d = criterion.split_values());
    Ok(tool)
}
pub(super) fn curve_done(tool: &Handle<dyn SplitCurve>) -> Result<bool, Failure> {
    require_tool(tool)?;
    Ok(tool.status_flag(ShapeExtendStatus::Done))
}
fn require_tool(tool: &Handle<dyn SplitCurve>) -> Result<(), Failure> {
    if tool.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeUpgrade_EdgeDivide: null split tool; expected initialized curve criterion",
            "",
        ));
    }
    Ok(())
}

fn edge_pcurve<T: EdgeDivider + ?Sized>(
    divider: &T,
    edge: &Edge,
) -> Result<crate::shape_analysis::PCurveRange, Failure> {
    let face = divider
        .edge_divide_data()
        .with(|fields| fields.face.clone());
    if face.is_null() {
        return Ok((Handle::default(), 0., 0.));
    }
    ShapeAnalysisEdge::new().pcurve_on_face(edge, &face, false)
}
