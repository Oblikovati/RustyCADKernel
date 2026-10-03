use super::ShapeFixFixSmallFace;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::vertices;
use rustycad_tkbrep::topods::{Builder, Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

pub(super) fn compute(
    fixer: &ShapeFixFixSmallFace,
    first_edge: &Edge,
    second_edge: &Edge,
    support_face: &Face,
    tolerance: f64,
) -> Result<Edge, Failure> {
    let context = context(fixer)?;
    let (first_start, first_end) = vertices(first_edge, false);
    let (second_start, second_end) = vertices(second_edge, false);
    let Some(endpoints) = match_endpoints(
        &context,
        [&first_start, &first_end, &second_start, &second_end],
        tolerance,
    )?
    else {
        return Ok(Edge::default());
    };
    make_shared_edge(&endpoints, first_edge, support_face)
}

struct SharedEndpoints {
    first: Vertex,
    second: Vertex,
}

fn match_endpoints(
    context: &Handle<ShapeBuildReShape>,
    vertices: [&Vertex; 4],
    tolerance: f64,
) -> Result<Option<SharedEndpoints>, Failure> {
    let [first_start, _, second_start, second_end] = vertices;
    let first_point = BRepTool::pnt(first_start)?;
    let start_distance = first_point.distance(&BRepTool::pnt(second_start)?);
    if close_endpoints(start_distance, first_start, second_start, tolerance)? {
        return match_direct_endpoints(context, vertices, start_distance);
    }
    let end_distance = first_point.distance(&BRepTool::pnt(second_end)?);
    if !close_endpoints(end_distance, first_start, second_end, tolerance)? {
        return Ok(None);
    }
    match_crossed_endpoints(context, vertices, end_distance)
}

fn close_endpoints(
    distance: f64,
    first: &Vertex,
    second: &Vertex,
    tolerance: f64,
) -> Result<bool, Failure> {
    Ok(distance <= BRepTool::vertex_tolerance(first)?
        || distance <= BRepTool::vertex_tolerance(second)?
        || distance <= tolerance)
}

fn match_direct_endpoints(
    context: &Handle<ShapeBuildReShape>,
    vertices: [&Vertex; 4],
    distance: f64,
) -> Result<Option<SharedEndpoints>, Failure> {
    let [first_start, _, second_start, _] = vertices;
    let first = if first_start.is_same(&second_start.clone().into_shape()) {
        first_start.clone()
    } else {
        let position = midpoint(BRepTool::pnt(first_start)?, BRepTool::pnt(second_start)?);
        let tolerance = (distance / 2.0) * 1.0001;
        let shared = make_vertex(position, tolerance)?;
        replace_vertex(context, first_start, &shared)?;
        replace_vertex(context, second_start, &shared)?;
        shared
    };
    let second = direct_far_vertex(context, vertices, &first)?;
    Ok(Some(SharedEndpoints { first, second }))
}

fn direct_far_vertex(
    context: &Handle<ShapeBuildReShape>,
    vertices: [&Vertex; 4],
    first: &Vertex,
) -> Result<Vertex, Failure> {
    let [first_start, first_end, second_start, second_end] = vertices;
    if first_start.is_same(&first_end.clone().into_shape())
        || second_start.is_same(&second_end.clone().into_shape())
    {
        return Ok(first.clone());
    }
    if !first_end.is_same(&second_end.clone().into_shape()) {
        let point = midpoint(BRepTool::pnt(first_end)?, BRepTool::pnt(second_end)?);
        let deviation = BRepTool::pnt(first_end)?.distance(&BRepTool::pnt(second_end)?);
        let second = make_vertex(point, deviation)?;
        replace_far_endpoints(context, first_end, second_end, &second)?;
        return Ok(second);
    }
    let _ = first;
    Ok(first_end.clone())
}

fn match_crossed_endpoints(
    context: &Handle<ShapeBuildReShape>,
    vertices: [&Vertex; 4],
    distance: f64,
) -> Result<Option<SharedEndpoints>, Failure> {
    let [first_start, _, _, second_end] = vertices;
    let point = midpoint(BRepTool::pnt(first_start)?, BRepTool::pnt(second_end)?);
    let first = if first_start.is_same(&second_end.clone().into_shape()) {
        first_start.clone()
    } else {
        let shared = make_vertex(point, (distance / 2.0) * 1.0001)?;
        replace_vertex(context, first_start, &shared)?;
        replace_vertex(context, second_end, &shared)?;
        shared
    };
    let second = crossed_far_vertex(context, vertices, &first)?;
    Ok(Some(SharedEndpoints { first, second }))
}

fn crossed_far_vertex(
    context: &Handle<ShapeBuildReShape>,
    vertices: [&Vertex; 4],
    first: &Vertex,
) -> Result<Vertex, Failure> {
    let [first_start, first_end, second_start, second_end] = vertices;
    if first_start.is_same(&first_end.clone().into_shape())
        || second_start.is_same(&second_end.clone().into_shape())
    {
        return Ok(first.clone());
    }
    if !first_end.is_same(&second_start.clone().into_shape()) {
        let point = midpoint(BRepTool::pnt(first_end)?, BRepTool::pnt(second_start)?);
        let deviation = BRepTool::pnt(first_end)?.distance(&BRepTool::pnt(second_start)?);
        let second = make_vertex(point, deviation)?;
        replace_far_endpoints(context, first_end, second_start, &second)?;
        return Ok(second);
    }
    Ok(first_end.clone())
}

fn replace_far_endpoints(
    context: &Handle<ShapeBuildReShape>,
    old_first: &Vertex,
    old_second: &Vertex,
    replacement: &Vertex,
) -> Result<(), Failure> {
    replace_vertex(context, old_first, replacement)?;
    replace_vertex(context, old_second, replacement)
}

fn replace_vertex(
    context: &Handle<ShapeBuildReShape>,
    old: &Vertex,
    replacement: &Vertex,
) -> Result<(), Failure> {
    let oriented = replacement.oriented(old.orientation());
    ReShape::replace(context.get().unwrap(), &old.clone().into_shape(), &oriented)
}

fn make_vertex(point: Point3, tolerance: f64) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
    Ok(vertex)
}

fn midpoint(first: Point3, second: Point3) -> Point3 {
    Point3::new(
        (first.x() + second.x()) / 2.0,
        (first.y() + second.y()) / 2.0,
        (first.z() + second.z()) / 2.0,
    )
}

fn make_shared_edge(
    endpoints: &SharedEndpoints,
    source_edge: &Edge,
    support_face: &Face,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    let first = BRepTool::vertex_tolerance(&endpoints.first)?;
    let second = BRepTool::vertex_tolerance(&endpoints.second)?;
    let tolerance = native_max(first, second);
    builder.update_vertex_tolerance(&endpoints.first, tolerance)?;
    builder.update_vertex_tolerance(&endpoints.second, tolerance)?;
    builder.same_parameter(&edge, false)?;
    update_shared_edge_curve(&builder, &edge, source_edge, tolerance)?;
    add_shared_edge_vertices(&mut edge, &endpoints.first, &endpoints.second)?;
    add_shared_edge_pcurve(&builder, &edge, source_edge, support_face, tolerance)?;
    Ok(edge)
}

fn update_shared_edge_curve(
    builder: &BRepBuilder,
    edge: &Edge,
    source: &Edge,
    tolerance: f64,
) -> Result<(), Failure> {
    let (curve, first, last) = BRepTool::curve(source)?;
    builder.update_edge_curve(edge, &curve, tolerance)?;
    builder.range(edge, first, last, false)
}

fn add_shared_edge_vertices(
    edge: &mut Edge,
    first: &Vertex,
    second: &Vertex,
) -> Result<(), Failure> {
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &first.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &second.oriented(Orientation::Reversed))?;
    *edge = rustycad_tkbrep::topods::edge(&shape)?;
    Ok(())
}

fn add_shared_edge_pcurve(
    builder: &BRepBuilder,
    edge: &Edge,
    source: &Edge,
    face: &Face,
    tolerance: f64,
) -> Result<(), Failure> {
    let source_curve = BRepTool::curve(source)?;
    let curve = BRepTool::curve_on_face(source, face)?;
    if curve.pcurve.is_null() || !curve.is_stored {
        return Ok(());
    }
    let mut replacement: Handle<dyn Geom2dCurve> = Handle::default();
    if curve.first != source_curve.1 && curve.last != source_curve.2 {
        rustycad_tkgeombase::geom_lib::same_range(
            precision::confusion(),
            &curve.pcurve,
            (curve.first, curve.last),
            (source_curve.1, source_curve.2),
            &mut replacement,
        )?;
    }
    if !replacement.is_null() {
        builder.update_edge_pcurve_on_face(edge, &replacement, face, tolerance)?;
    }
    Ok(())
}

fn native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}

fn context(fixer: &ShapeFixFixSmallFace) -> Result<Handle<ShapeBuildReShape>, Failure> {
    let context = fixer.context();
    if context.get().is_some() {
        return Ok(context);
    }
    Err(Failure::new(
        "ShapeFix_FixSmallFace requires Init(shape) before creating a strip edge",
    ))
}
