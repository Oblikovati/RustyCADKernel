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
pub(super) fn merge_line_wire_chain(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
) -> Result<Shape, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(shape.clone());
    }
    let edges = wire_edges(shape)?;
    if edges.len() < 2 {
        return Ok(shape.clone());
    }
    merge_edge_chain(tool, shape, &edges)
}

fn wire_edges(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|edge| rustycad_tkbrep::topods::edge(&edge))
        .collect()
}

fn merge_edge_chain(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
    edges: &[Edge],
) -> Result<Shape, Failure> {
    let fields = tool.fields.read().clone();
    if !fields.unify_edges || !all_linear(edges, fields.angular_tolerance)? {
        return Ok(shape.clone());
    }
    let Some((ordered, start, end)) = order_chain(edges) else {
        return Ok(shape.clone());
    };
    rebuild_merged_wire(shape, &ordered, start, end)
}

fn all_linear(edges: &[Edge], tolerance: f64) -> Result<bool, Failure> {
    for pair in edges.windows(2) {
        if !pair_is_linear(&pair[0], &pair[1], tolerance)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn pair_is_linear(first: &Edge, second: &Edge, tolerance: f64) -> Result<bool, Failure> {
    let first_curve = BRepAdaptorCurve::with_edge(first)?;
    let second_curve = BRepAdaptorCurve::with_edge(second)?;
    if first_curve.get_type()? != CurveType::Line || second_curve.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let first_direction = first_curve.line()?.position().direction();
    let second_direction = second_curve.line()?.position().direction();
    Ok(first_direction.is_parallel(second_direction, tolerance))
}

fn order_chain(edges: &[Edge]) -> Option<(Vec<Edge>, Vertex, Vertex)> {
    let analysis = ShapeAnalysisEdge::new();
    let first = edges.first()?.clone();
    let mut ordered = vec![first.clone()];
    let mut start = analysis.first_vertex(&first);
    let mut end = analysis.last_vertex(&first);
    extend_chain_front(edges, &analysis, &mut ordered, &mut start);
    extend_chain_back(edges, &analysis, &mut ordered, &mut end);
    if ordered.len() != edges.len() || start.is_same(&end) {
        return None;
    }
    Some((ordered, start, end))
}

fn extend_chain_front(
    edges: &[Edge],
    analysis: &ShapeAnalysisEdge,
    ordered: &mut Vec<Edge>,
    start: &mut Vertex,
) {
    while let Some(edge) = edges
        .iter()
        .find(|edge| !contains_edge(ordered, edge) && analysis.last_vertex(edge).is_same(start))
    {
        ordered.insert(0, edge.clone());
        *start = analysis.first_vertex(edge);
    }
}

fn extend_chain_back(
    edges: &[Edge],
    analysis: &ShapeAnalysisEdge,
    ordered: &mut Vec<Edge>,
    end: &mut Vertex,
) {
    while let Some(edge) = edges
        .iter()
        .find(|edge| !contains_edge(ordered, edge) && analysis.first_vertex(edge).is_same(end))
    {
        ordered.push(edge.clone());
        *end = analysis.last_vertex(edge);
    }
}

fn contains_edge(edges: &[Edge], candidate: &Edge) -> bool {
    edges.iter().any(|edge| edge.is_same(candidate))
}

fn rebuild_merged_wire(
    shape: &Shape,
    edges: &[Edge],
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
    apply_edge_merge(shape, edges, &edge.into_shape())
}

fn apply_edge_merge(shape: &Shape, edges: &[Edge], merged: &Shape) -> Result<Shape, Failure> {
    let context = Handle::new(ShapeBuildReShape::new());
    let parts = edges
        .iter()
        .cloned()
        .map(Edge::into_shape)
        .collect::<Vec<_>>();
    context.merge(&parts, merged)?;
    context.apply(shape, ShapeEnum::Shape)
}
