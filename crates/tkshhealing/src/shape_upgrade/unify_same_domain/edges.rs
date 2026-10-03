use super::{ShapeUpgradeUnifySameDomain, face_wires};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

/// Merges linear chains whose p-curves need no curved-support transformation.
pub(super) fn merge_line_wire_chain(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
) -> Result<Shape, Failure> {
    let fields = tool.fields.read().clone();
    if !fields.unify_edges {
        return Ok(shape.clone());
    }
    match shape.shape_type()? {
        ShapeEnum::Wire => merge_wire_lines(shape, fields.angular_tolerance),
        ShapeEnum::Face => merge_planar_face_lines(shape, fields.angular_tolerance),
        _ => Ok(shape.clone()),
    }
}

fn merge_wire_lines(shape: &Shape, angular_tolerance: f64) -> Result<Shape, Failure> {
    let mut result = shape.clone();
    let limit = wire_edges(shape)?.len();
    for _ in 0..limit {
        let edges = wire_edges(&result)?;
        let Some((first, second, start, end)) = first_linear_pair(&edges, angular_tolerance)?
        else {
            break;
        };
        result = replace_pair(&result, &first, &second, start, end)?;
    }
    Ok(result)
}

fn merge_planar_face_lines(shape: &Shape, angular_tolerance: f64) -> Result<Shape, Failure> {
    let face = rustycad_tkbrep::topods::face(shape)?;
    if BRepAdaptorSurface::with_face(&face, false)?.get_type()? != SurfaceType::Plane {
        return Ok(shape.clone());
    }
    let wires = face_wires(&face)?;
    if wires.len() != 1 {
        return Ok(shape.clone());
    }
    let before = wire_edges(&wires[0].clone().into_shape())?.len();
    let merged = merge_wire_lines(&wires[0].clone().into_shape(), angular_tolerance)?;
    if wire_edges(&merged)?.len() == before {
        return Ok(shape.clone());
    }
    rebuild_planar_face(&face, &rustycad_tkbrep::topods::wire(&merged)?)
}

fn first_linear_pair(
    edges: &[Edge],
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            if let Some(pair) = connected_linear_pair(&edges[i], &edges[j], tolerance)? {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

fn connected_linear_pair(
    first: &Edge,
    second: &Edge,
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    if !both_linear(first, second, tolerance)? {
        return Ok(None);
    }
    let analysis = ShapeAnalysisEdge::new();
    let (first_start, first_end) = edge_vertices(&analysis, first);
    let (second_start, second_end) = edge_vertices(&analysis, second);
    if first_end.is_same(&second_start) {
        return Ok(Some((
            first.clone(),
            second.clone(),
            first_start,
            second_end,
        )));
    }
    if second_end.is_same(&first_start) {
        return Ok(Some((
            second.clone(),
            first.clone(),
            second_start,
            first_end,
        )));
    }
    Ok(None)
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

fn edge_vertices(analysis: &ShapeAnalysisEdge, edge: &Edge) -> (Vertex, Vertex) {
    (analysis.first_vertex(edge), analysis.last_vertex(edge))
}

fn replace_pair(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    start: Vertex,
    end: Vertex,
) -> Result<Shape, Failure> {
    let edge = make_line_edge(start, end)?;
    let context = Handle::new(ShapeBuildReShape::new());
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        &edge.into_shape(),
    )?;
    context.apply(shape, ShapeEnum::Shape)
}

fn make_line_edge(start: Vertex, end: Vertex) -> Result<Edge, Failure> {
    let first_point = BRepTool::pnt(&start)?;
    let last_point = BRepTool::pnt(&end)?;
    let line = trimmed_line(first_point, last_point)?;
    let distance = first_point.distance(&last_point);
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(&line, &start, &end, 0., distance)?.edge()
}

fn trimmed_line(start: Point3, end: Point3) -> Result<Handle<dyn GeomCurve>, Failure> {
    let direction = Direction3::new(
        end.x() - start.x(),
        end.y() - start.y(),
        end.z() - start.z(),
    )?;
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::new(Axis1::new(start, direction))).cast();
    let distance = start.distance(&end);
    Ok(Handle::new(GeomTrimmedCurve::new(&line, 0., distance, true, true)?).cast())
}

fn wire_edges(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?
        .map(|edge| rustycad_tkbrep::topods::edge(&edge))
        .collect()
}

fn rebuild_planar_face(face: &Face, wire: &Wire) -> Result<Shape, Failure> {
    let surface = BRepTool::surface(face)?;
    let mut rebuilt = BRepBuilderAPIMakeFace::from_surface_wire(&surface, wire, true)?
        .face()?
        .into_shape();
    rebuilt.set_orientation(face.orientation());
    Ok(rebuilt)
}
