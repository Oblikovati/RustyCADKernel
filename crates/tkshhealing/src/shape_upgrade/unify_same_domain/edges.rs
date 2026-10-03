use super::{ShapeUpgradeUnifySameDomain, face_wires};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};

/// Merges line and circle chains whose p-curves need no support transformation.
pub(super) fn merge_wire_edge_chains(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
) -> Result<Shape, Failure> {
    let fields = tool.fields.read().clone();
    if !fields.unify_edges {
        return Ok(shape.clone());
    }
    match shape.shape_type()? {
        ShapeEnum::Wire => merge_wire_chains(shape, fields.angular_tolerance, true),
        ShapeEnum::Face => merge_planar_face_lines(shape, fields.angular_tolerance),
        _ => Ok(shape.clone()),
    }
}

fn merge_wire_chains(
    shape: &Shape,
    angular_tolerance: f64,
    unify_circles: bool,
) -> Result<Shape, Failure> {
    let mut result = shape.clone();
    let limit = wire_edges(shape)?.len();
    for _ in 0..limit {
        let edges = wire_edges(&result)?;
        if let Some((first, second, start, end)) = first_linear_pair(&edges, angular_tolerance)? {
            result = replace_line_pair(&result, &first, &second, start, end)?;
            continue;
        }
        if !unify_circles {
            break;
        }
        let Some((first, second, start, end)) = first_circle_pair(&edges, angular_tolerance)?
        else {
            break;
        };
        result = replace_circle_pair(&result, &first, &second, start, end)?;
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
    let merged = merge_wire_chains(&wires[0].clone().into_shape(), angular_tolerance, false)?;
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

fn replace_line_pair(
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

fn first_circle_pair(
    edges: &[Edge],
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            if let Some(pair) = connected_circle_pair(&edges[i], &edges[j], tolerance)? {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

fn connected_circle_pair(
    first: &Edge,
    second: &Edge,
    tolerance: f64,
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    let Some((first_circle, second_circle)) = circle_pair(first, second)? else {
        return Ok(None);
    };
    if first_circle.location().distance(&second_circle.location()) > precision::confusion() {
        return Ok(None);
    }
    let analysis = ShapeAnalysisEdge::new();
    let (first_start, first_end) = edge_vertices(&analysis, first);
    let (second_start, second_end) = edge_vertices(&analysis, second);
    if first_end.is_same(&second_start) {
        ensure_same_circle_support(&first_circle, &second_circle, tolerance)?;
        return Ok(Some((
            first.clone(),
            second.clone(),
            first_start,
            second_end,
        )));
    }
    if second_end.is_same(&first_start) {
        ensure_same_circle_support(&first_circle, &second_circle, tolerance)?;
        return Ok(Some((
            second.clone(),
            first.clone(),
            second_start,
            first_end,
        )));
    }
    Ok(None)
}

fn circle_pair(first: &Edge, second: &Edge) -> Result<Option<(Circle3, Circle3)>, Failure> {
    let first_adaptor = BRepAdaptorCurve::with_edge(first)?;
    let second_adaptor = BRepAdaptorCurve::with_edge(second)?;
    if first_adaptor.get_type()? != CurveType::Circle
        || second_adaptor.get_type()? != CurveType::Circle
    {
        return Ok(None);
    }
    Ok(Some((first_adaptor.circle()?, second_adaptor.circle()?)))
}

fn ensure_same_circle_support(
    first: &Circle3,
    second: &Circle3,
    tolerance: f64,
) -> Result<(), Failure> {
    let same_radius = (first.radius() - second.radius()).abs() <= tolerance;
    let same_plane = first
        .position()
        .direction()
        .is_parallel(second.position().direction(), tolerance);
    if same_radius && same_plane {
        return Ok(());
    }
    Err(Failure::with_type(
        "Standard_NotImplemented",
        &format!(
            "UnifySameDomain circle supports have radii {} and {}; expected equal radii and parallel axes",
            first.radius(),
            second.radius()
        ),
        "",
    ))
}

fn replace_circle_pair(
    shape: &Shape,
    first: &Edge,
    second: &Edge,
    start: Vertex,
    end: Vertex,
) -> Result<Shape, Failure> {
    let edge = make_merged_circle_edge(first, &start, &end)?;
    let context = Handle::new(ShapeBuildReShape::new());
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        &edge.into_shape(),
    )?;
    context.apply(shape, ShapeEnum::Shape)
}

fn make_merged_circle_edge(first: &Edge, start: &Vertex, end: &Vertex) -> Result<Edge, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(first)?;
    let source = adaptor.circle()?;
    let first_point = BRepTool::pnt(start)?;
    let last_point = BRepTool::pnt(end)?;
    let reference = circle_reference_point(first, start, &adaptor)?;
    let circle = merged_circle(source, first_point, reference)?;
    let angle = merged_circle_angle(circle.location(), first_point, last_point, reference)?;
    let curve = trimmed_circle(circle, angle)?;
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(&curve, start, end, 0., angle)?.edge()
}

fn merged_circle(source: Circle3, first: Point3, last: Point3) -> Result<Circle3, Failure> {
    let first_radius = Vector3::between(source.location(), first);
    let last_radius = Vector3::between(source.location(), last);
    let normal = circle_normal(first_radius, last_radius)?;
    let x_axis = Direction3::new(first_radius.x(), first_radius.y(), first_radius.z())?;
    Circle3::new(
        Axis2::with_x_direction(source.location(), normal, x_axis)?,
        source.radius(),
    )
}

fn circle_normal(first_radius: Vector3, last_radius: Vector3) -> Result<Direction3, Failure> {
    let cross = first_radius.crossed(last_radius);
    Direction3::new(cross.x(), cross.y(), cross.z())
}

fn trimmed_circle(circle: Circle3, angle: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    let basis: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
    Ok(Handle::new(GeomTrimmedCurve::new(&basis, 0., angle, true, true)?).cast())
}

fn circle_reference_point(
    edge: &Edge,
    start: &Vertex,
    adaptor: &BRepAdaptorCurve,
) -> Result<Point3, Failure> {
    let first = BRepTool::parameter(start, edge)?;
    let last_vertex = ShapeAnalysisEdge::new().last_vertex(edge);
    let mut last = BRepTool::parameter(&last_vertex, edge)?;
    while (last - first).abs() > 7. * std::f64::consts::PI / 8. {
        last = (first + last) / 2.;
    }
    adaptor.value(last)
}

fn merged_circle_angle(
    origin: Point3,
    first: Point3,
    last: Point3,
    reference: Point3,
) -> Result<f64, Failure> {
    let first_radius = Vector3::between(origin, first);
    let last_radius = Vector3::between(origin, last);
    let reference_radius = Vector3::between(origin, reference);
    let normal = circle_normal(first_radius, reference_radius)?;
    let first_direction = Direction3::new(first_radius.x(), first_radius.y(), first_radius.z())?;
    let last_direction = Direction3::new(last_radius.x(), last_radius.y(), last_radius.z())?;
    let mut angle = first_direction.angle_with_ref(last_direction, normal);
    if angle < 0.0 {
        angle += std::f64::consts::TAU;
    }
    Ok(angle)
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
