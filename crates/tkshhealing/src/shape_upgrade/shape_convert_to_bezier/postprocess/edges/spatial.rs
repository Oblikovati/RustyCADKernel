use super::super::super::ShapeUpgradeShapeConvertToBezier;
use crate::{shape_analysis::ShapeAnalysisEdge, shape_build::ShapeBuildEdge};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::Edge,
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};

pub(super) fn process_edge(
    tool: &ShapeUpgradeShapeConvertToBezier,
    edge: &Edge,
) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (curve, first, last) = analysis.curve_3d(edge, false)?;
    let bezier_handle = curve.cast::<GeomBezierCurve>();
    let Some(bezier) = bezier_handle.get() else {
        return same_range_for_pcurve_mode(tool, edge, &curve);
    };
    normalize_spatial_range(edge, &curve, bezier, first, last)?;
    snap_spatial_ends(&analysis, edge, bezier, first, last)?;
    Ok(())
}

fn normalize_spatial_range(
    edge: &Edge,
    source: &Handle<dyn GeomCurve>,
    bezier: &GeomBezierCurve,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    if first == 0. && last == 1. {
        return Ok(());
    }
    if first > precision::p_confusion() || last < 1. - precision::p_confusion() {
        bezier.segment(first, last)?;
        BRepBuilder::default().update_edge_curve(edge, source, 0.)?;
    }
    ShapeBuildEdge.set_range_3d(edge, 0., 1.)
}

fn snap_spatial_ends(
    analysis: &ShapeAnalysisEdge,
    edge: &Edge,
    bezier: &GeomBezierCurve,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let first_vertex = analysis.first_vertex(edge);
    let last_vertex = analysis.last_vertex(edge);
    let first_point = bezier.value(first)?;
    let last_point = bezier.value(last)?;
    let first_target = BRepTool::pnt(&first_vertex)?;
    let last_target = BRepTool::pnt(&last_vertex)?;
    snap_first(bezier, first_point, first_target)?;
    snap_last(bezier, last_point, last_target)
}

fn snap_first(
    curve: &GeomBezierCurve,
    current: rustycad_tkmath::gp::Point3,
    target: rustycad_tkmath::gp::Point3,
) -> Result<(), Failure> {
    if current.distance(&target) > precision::confusion() {
        curve.set_pole(1, target)?;
    }
    Ok(())
}

fn snap_last(
    curve: &GeomBezierCurve,
    current: rustycad_tkmath::gp::Point3,
    target: rustycad_tkmath::gp::Point3,
) -> Result<(), Failure> {
    if current.distance(&target) > precision::confusion() {
        curve.set_pole(curve.nb_poles(), target)?;
    }
    Ok(())
}

fn same_range_for_pcurve_mode(
    tool: &ShapeUpgradeShapeConvertToBezier,
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
) -> Result<(), Failure> {
    if !curve.is_null() && tool.get_2d_conversion() {
        BRepBuilder::default().same_range(edge, true)?;
    }
    Ok(())
}
