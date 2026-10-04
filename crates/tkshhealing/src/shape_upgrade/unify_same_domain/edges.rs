use super::ShapeUpgradeUnifySameDomain;
mod bspline;
mod containers;
mod global_context;
mod global_merge;
mod pcurve_circle;
mod pcurve_concat;
mod pcurve_union;
mod safe_input;
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildReShape;
pub(super) use global_context::protected_shapes;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_lib::build_pcurve_for_edges_on_plane;
use safe_input::{record_safe_vertex, safe_merge_vertex};

#[derive(Clone, Copy)]
pub(super) struct EdgeMergeOptions<'a> {
    angular_tolerance: f64,
    concat_bspline_edges: bool,
    keep_shapes: &'a [Shape],
    safe_input_mode: bool,
    context: &'a Handle<ShapeBuildReShape>,
}

/// Merges supported wire and planar-face chains, restoring missing planar p-curves.
pub(super) fn merge_wire_edge_chains(
    tool: &ShapeUpgradeUnifySameDomain,
    shape: &Shape,
) -> Result<Shape, Failure> {
    let fields = tool.fields.read().clone();
    if !fields.unify_edges {
        return Ok(shape.clone());
    }
    let context = Handle::new(ShapeBuildReShape::new());
    let protected = protected_shapes(shape, &fields.keep_shapes)?;
    let options = EdgeMergeOptions {
        angular_tolerance: fields.angular_tolerance,
        concat_bspline_edges: fields._concat_bspline_edges,
        keep_shapes: &protected,
        safe_input_mode: fields.safe_input_mode,
        context: &context,
    };
    match shape.shape_type()? {
        ShapeEnum::Wire => merge_wire_chains(shape, true, &options),
        _ => containers::merge_nested_children(shape, &options),
    }
}

pub(super) fn has_unported_spline_candidate(
    shape: &Shape,
    angular_tolerance: f64,
    concat_bspline_edges: bool,
    keep_shapes: &[Shape],
) -> Result<bool, Failure> {
    if !concat_bspline_edges {
        return Ok(false);
    }
    bspline::has_candidate(shape, angular_tolerance, keep_shapes)
}

fn merge_wire_chains(
    shape: &Shape,
    unify_circles: bool,
    options: &EdgeMergeOptions<'_>,
) -> Result<Shape, Failure> {
    let mut result = shape.clone();
    let limit = wire_edges(shape)?.len();
    for _ in 0..limit {
        let edges = wire_edges(&result)?;
        if let Some((first, second, start, end)) =
            first_linear_pair(&edges, options.angular_tolerance, options.keep_shapes)?
        {
            result = replace_line_pair(
                &result,
                options.context,
                &first,
                &second,
                start,
                end,
                options.safe_input_mode,
            )?;
            continue;
        }
        if unify_circles
            && let Some((first, second, start, end)) =
                first_circle_pair(&edges, options.angular_tolerance, options.keep_shapes)?
        {
            result = replace_circle_pair(
                &result,
                options.context,
                &first,
                &second,
                start,
                end,
                options.safe_input_mode,
            )?;
            continue;
        }
        if options.concat_bspline_edges
            && let Some(joined) = bspline::merge_first_pair(&result, &edges, options)?
        {
            result = joined;
            continue;
        }
        break;
    }
    Ok(result)
}

pub(super) fn rebuild_planar_face_pcurves(shape: &Shape) -> Result<Shape, Failure> {
    let face = rustycad_tkbrep::topods::face(shape)?;
    let edges = wire_edges(shape)?;
    let edge_shapes = edges
        .iter()
        .map(|edge| edge.clone().into_shape())
        .collect::<Vec<_>>();
    // The replacement edge has no stored p-curve; planar projection restores its UV curve.
    build_pcurve_for_edges_on_plane(edge_shapes.iter(), &face)?;
    Ok(shape.clone())
}

fn first_linear_pair(
    edges: &[Edge],
    tolerance: f64,
    keep_shapes: &[Shape],
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            if let Some(pair) = connected_linear_pair(&edges[i], &edges[j], tolerance)?
                && !keeps_join_vertex(&pair.0, keep_shapes)
            {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

pub(super) fn keeps_join_vertex(first: &Edge, keep_shapes: &[Shape]) -> bool {
    let analysis = ShapeAnalysisEdge::new();
    let join = analysis.last_vertex(first);
    keep_shapes
        .iter()
        .any(|kept| kept.is_same(&join.clone().into_shape()))
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
    if connected_vertex_occurrences(&first_end, &second_start) {
        return Ok(Some((
            first.clone(),
            second.clone(),
            first_start,
            second_end,
        )));
    }
    if connected_vertex_occurrences(&second_end, &first_start) {
        return Ok(Some((
            second.clone(),
            first.clone(),
            second_start,
            first_end,
        )));
    }
    Ok(None)
}

pub(super) fn connected_vertex_occurrences(first: &Vertex, second: &Vertex) -> bool {
    first.is_same(second)
        && matches!(
            (first.orientation(), second.orientation()),
            (Orientation::Forward, Orientation::Reversed)
                | (Orientation::Reversed, Orientation::Forward)
        )
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
    context: &Handle<ShapeBuildReShape>,
    first: &Edge,
    second: &Edge,
    start: Vertex,
    end: Vertex,
    safe_input_mode: bool,
) -> Result<Shape, Failure> {
    let original_start = start.clone();
    let original_end = end.clone();
    let start = safe_merge_vertex(&start, safe_input_mode)?;
    let end = safe_merge_vertex(&end, safe_input_mode)?;
    let edge = make_line_edge(start.clone(), end.clone())?;
    pcurve_union::union_pcurves(shape, first, second, &edge)?;
    record_safe_vertex(context, &original_start, &start, safe_input_mode)?;
    record_safe_vertex(context, &original_end, &end, safe_input_mode)?;
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        &edge.into_shape(),
    )?;
    context.apply(shape, ShapeEnum::Shape)
}

fn first_circle_pair(
    edges: &[Edge],
    tolerance: f64,
    keep_shapes: &[Shape],
) -> Result<Option<(Edge, Edge, Vertex, Vertex)>, Failure> {
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            if let Some(pair) = connected_circle_pair(&edges[i], &edges[j], tolerance)?
                && !keeps_join_vertex(&pair.0, keep_shapes)
            {
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
    if connected_vertex_occurrences(&first_end, &second_start) {
        if !bspline::aligned_join(first, second, tolerance)? {
            return Ok(None);
        }
        if second_end.is_same(&first_start) {
            return Ok(Some((
                first.clone(),
                second.clone(),
                first_end.clone(),
                first_end,
            )));
        }
        return Ok(Some((
            first.clone(),
            second.clone(),
            first_start,
            second_end,
        )));
    }
    if connected_vertex_occurrences(&second_end, &first_start) {
        if !bspline::aligned_join(second, first, tolerance)? {
            return Ok(None);
        }
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

fn replace_circle_pair(
    shape: &Shape,
    context: &Handle<ShapeBuildReShape>,
    first: &Edge,
    second: &Edge,
    start: Vertex,
    end: Vertex,
    safe_input_mode: bool,
) -> Result<Shape, Failure> {
    let closed = start.is_same(&end);
    let original_start = start.clone();
    let original_end = end.clone();
    let start = safe_merge_vertex(&start, safe_input_mode)?;
    let end = if closed {
        start.clone()
    } else {
        safe_merge_vertex(&end, safe_input_mode)?
    };
    let edge = if closed {
        make_closed_circle_edge(first, &start)?
    } else {
        make_merged_circle_edge(first, &original_start, &original_end, &start, &end)?
    };
    record_safe_vertex(context, &original_start, &start, safe_input_mode)?;
    if !closed {
        record_safe_vertex(context, &original_end, &end, safe_input_mode)?;
    }
    context.merge(
        &[first.clone().into_shape(), second.clone().into_shape()],
        &edge.into_shape(),
    )?;
    context.apply(shape, ShapeEnum::Shape)
}

fn make_closed_circle_edge(first: &Edge, vertex: &Vertex) -> Result<Edge, Failure> {
    let source = BRepAdaptorCurve::with_edge(first)?.circle()?;
    let point = BRepTool::pnt(vertex)?;
    let radial = Vector3::between(source.location(), point);
    let x_axis = Direction3::new(radial.x(), radial.y(), radial.z())?;
    let position =
        Axis2::with_x_direction(source.location(), source.position().direction(), x_axis)?;
    let circle = Circle3::new(position, source.radius())?;
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(circle)).cast();
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(
        &curve,
        vertex,
        vertex,
        0.,
        std::f64::consts::TAU,
    )?
    .edge()
}

fn make_merged_circle_edge(
    first: &Edge,
    source_start: &Vertex,
    source_end: &Vertex,
    start: &Vertex,
    end: &Vertex,
) -> Result<Edge, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(first)?;
    let source = adaptor.circle()?;
    let first_point = BRepTool::pnt(source_start)?;
    let last_point = BRepTool::pnt(source_end)?;
    let reference = circle_reference_point(first, source_start, &adaptor)?;
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
