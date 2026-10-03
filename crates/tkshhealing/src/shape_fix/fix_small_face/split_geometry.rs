use super::ShapeFixFixSmallFace;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{Explorer, vertices};
use rustycad_tkbrep::topods::{Builder, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) struct SplitBoundary {
    pub vertex: Vertex,
    pub split_vertex: Vertex,
    pub source_edge: Edge,
    pub first_edge: Edge,
    pub second_edge: Edge,
}

pub(super) fn find_and_split_boundary(
    fixer: &ShapeFixFixSmallFace,
    face: &Face,
    vertex: &Vertex,
) -> Result<Option<SplitBoundary>, Failure> {
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let edge = rustycad_tkbrep::topods::edge(edges.current())?;
        if let Some(boundary) = split_candidate_edge(fixer, &edge, vertex)? {
            return Ok(Some(boundary));
        }
        edges.next()?;
    }
    Ok(None)
}

fn split_candidate_edge(
    fixer: &ShapeFixFixSmallFace,
    edge: &Edge,
    vertex: &Vertex,
) -> Result<Option<SplitBoundary>, Failure> {
    let (first_vertex, last_vertex) = vertices(edge, false);
    if vertex.is_same(&first_vertex.into_shape()) || vertex.is_same(&last_vertex.into_shape()) {
        return Ok(None);
    }
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.get().is_none() {
        return Ok(None);
    }
    let Some(projection) = split_projection(vertex, &curve, (first, last))? else {
        return Ok(None);
    };
    make_boundary(fixer, edge, vertex, curve, (first, last), projection)
}

fn split_projection(
    vertex: &Vertex,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
) -> Result<Option<(f64, rustycad_tkmath::gp::Point3, f64)>, Failure> {
    let point = BRepTool::pnt(vertex)?;
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    let projection = crate::shape_analysis::ShapeAnalysisCurve.project(
        curve,
        point,
        tolerance * 10.0,
        range,
        false,
    )?;
    Ok((projection.0 != 0.0 && projection.0 <= tolerance).then_some(projection))
}

fn make_boundary(
    fixer: &ShapeFixFixSmallFace,
    edge: &Edge,
    vertex: &Vertex,
    curve: Handle<dyn GeomCurve>,
    range: (f64, f64),
    projection: (f64, rustycad_tkmath::gp::Point3, f64),
) -> Result<Option<SplitBoundary>, Failure> {
    let boundary = create_boundary_vertex_edges(edge, vertex, &curve, range, projection)?;
    replace_source_edge(fixer, edge, &boundary.first_edge, &boundary.second_edge)?;
    Ok(Some(boundary))
}

fn create_boundary_vertex_edges(
    edge: &Edge,
    source_vertex: &Vertex,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    projection: (f64, rustycad_tkmath::gp::Point3, f64),
) -> Result<SplitBoundary, Failure> {
    let endpoints = vertices(edge, false);
    let split_vertex = projected_vertex(projection.1)?;
    let parameters = (range.0, projection.2, range.1);
    let (first_edge, second_edge) =
        split_boundary_edges(curve, endpoints, &split_vertex, parameters)?;
    Ok(SplitBoundary {
        vertex: source_vertex.clone(),
        split_vertex,
        source_edge: edge.clone(),
        first_edge,
        second_edge,
    })
}

fn projected_vertex(point: rustycad_tkmath::gp::Point3) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
    Ok(vertex)
}

fn split_boundary_edges(
    curve: &Handle<dyn GeomCurve>,
    endpoints: (Vertex, Vertex),
    split_vertex: &Vertex,
    parameters: (f64, f64, f64),
) -> Result<(Edge, Edge), Failure> {
    let first = split_edge_piece(
        curve,
        endpoints.0,
        split_vertex.clone(),
        (parameters.0, parameters.1),
    )?;
    let second = split_edge_piece(
        curve,
        split_vertex.clone(),
        endpoints.1,
        (parameters.1, parameters.2),
    )?;
    Ok((first, second))
}

fn split_edge_piece(
    curve: &Handle<dyn GeomCurve>,
    first_vertex: Vertex,
    last_vertex: Vertex,
    range: (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, curve, precision::confusion())?;
    builder.range(&edge, range.0, range.1, true)?;
    let mut shape = edge.clone().into_shape();
    Builder.add(&mut shape, &first_vertex.oriented(Orientation::Forward))?;
    Builder.add(&mut shape, &last_vertex.oriented(Orientation::Reversed))?;
    rustycad_tkbrep::topods::edge(&shape)
}

fn replace_source_edge(
    fixer: &ShapeFixFixSmallFace,
    source: &Edge,
    first: &Edge,
    second: &Edge,
) -> Result<(), Failure> {
    let replacement = replacement_wire(source, first, second)?;
    let context = fixer.context();
    let context = context
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_FixSmallFace requires Init before edge splitting"))?;
    ReShape::replace(context, &source.clone().into_shape(), &replacement)
}

fn replacement_wire(source: &Edge, first: &Edge, second: &Edge) -> Result<Shape, Failure> {
    let mut replacement = Wire::default();
    Builder.make_wire(&mut replacement);
    let mut wire_shape = replacement.into_shape();
    let orientation = source.orientation();
    let (first_orientation, second_orientation) = if orientation == Orientation::Forward {
        (Orientation::Forward, Orientation::Forward)
    } else {
        (Orientation::Reversed, Orientation::Reversed)
    };
    Builder.add(&mut wire_shape, &first.oriented(first_orientation))?;
    Builder.add(&mut wire_shape, &second.oriented(second_orientation))?;
    Ok(wire_shape)
}
