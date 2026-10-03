use super::ShapeUpgradeUnifySameDomain;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Direction3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

/// Merges the native two-line chain without needing face p-curve rewrites.
pub(super) fn merge_two_line_wire_edges(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
) -> Result<Shape, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(shape.clone());
    }
    let edges = wire_edges(shape)?;
    if edges.len() != 2 {
        return Ok(shape.clone());
    }
    merge_edge_pair(tool, shape, &edges[0], &edges[1])
}

fn wire_edges(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|edge| rustycad_tkbrep::topods::edge(&edge))
        .collect()
}

fn merge_edge_pair(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
    first: &Edge,
    second: &Edge,
) -> Result<Shape, Failure> {
    let fields = tool.fields.read().clone();
    if !fields.unify_edges || !both_linear(first, second, fields.angular_tolerance)? {
        return Ok(shape.clone());
    }
    let Some((start, end)) = ordered_endpoints(first, second)? else {
        return Ok(shape.clone());
    };
    rebuild_merged_wire(shape, first, second, start, end)
}

fn both_linear(first: &Edge, second: &Edge, tolerance: f64) -> Result<bool, Failure> {
    let first_curve = BRepAdaptorCurve::with_edge(first)?;
    let second_curve = BRepAdaptorCurve::with_edge(second)?;
    if first_curve.get_type()? != CurveType::Line || second_curve.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let first_direction = first_curve.line()?.position().direction();
    let second_direction = second_curve.line()?.position().direction();
    Ok(first_direction.is_parallel(second_direction, tolerance))
}

fn ordered_endpoints(first: &Edge, second: &Edge) -> Result<Option<(Vertex, Vertex)>, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (first_start, first_end) = edge_vertices(&analysis, first);
    let (second_start, second_end) = edge_vertices(&analysis, second);
    if first_end.is_same(&second_start) {
        return Ok(Some((first_start, second_end)));
    }
    if second_end.is_same(&first_start) {
        return Ok(Some((second_start, first_end)));
    }
    Ok(None)
}

fn edge_vertices(analysis: &ShapeAnalysisEdge, edge: &Edge) -> (Vertex, Vertex) {
    (analysis.first_vertex(edge), analysis.last_vertex(edge))
}

fn rebuild_merged_wire(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    start: Vertex,
    end: Vertex,
) -> Result<Shape, Failure> {
    let start_point = BRepTool::pnt(&start)?;
    let end_point = BRepTool::pnt(&end)?;
    let direction = Direction3::new(
        end_point.x() - start_point.x(),
        end_point.y() - start_point.y(),
        end_point.z() - start_point.z(),
    )?;
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(start_point, direction))).cast();
    let distance = start_point.distance(&end_point);
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line, 0., distance, true, true)?).cast();
    let edge =
        BRepBuilderAPIMakeEdge::from_curve_vertices_params(&trimmed, &start, &end, 0., distance)?
            .edge()?;
    apply_edge_merge(shape, first, second, &edge.into_shape())
}

fn apply_edge_merge(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    merged: &Shape,
) -> Result<Shape, Failure> {
    let context = Handle::new(ShapeBuildReShape::new());
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        merged,
    )?;
    context.apply(shape, ShapeEnum::Shape)
}
