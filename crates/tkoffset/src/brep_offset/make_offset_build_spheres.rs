//! Vertex spheres closing three or more offset edge patches.
use super::{BRepOffsetAnalyse, BRepOffsetOffset, make_offset_build_adjacent::SharedSupportJoin};
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) struct SphericalVertexJoin {
    pub(super) source_vertex: Vertex,
    pub(super) face: Face,
}

pub(super) fn vertex_sphere_joins(
    faces: &[Face],
    joins: &[SharedSupportJoin],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
) -> Result<Vec<SphericalVertexJoin>, Failure> {
    let mut result = Vec::new();
    for face in faces {
        append_face_vertex_spheres(face, joins, analysis, offset, &mut result)?;
    }
    Ok(result)
}

fn append_face_vertex_spheres(
    face: &Face,
    joins: &[SharedSupportJoin],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
    result: &mut Vec<SphericalVertexJoin>,
) -> Result<(), Failure> {
    let shape = face.clone().into_shape();
    for shape in Explorer::new(&shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vertex = Vertex::unchecked(shape);
        if result
            .iter()
            .any(|join| join.source_vertex.is_same(&vertex.clone().into_shape()))
        {
            continue;
        }
        append_vertex_sphere(&vertex, joins, analysis, offset, result)?;
    }
    Ok(())
}

fn append_vertex_sphere(
    vertex: &Vertex,
    joins: &[SharedSupportJoin],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
    result: &mut Vec<SphericalVertexJoin>,
) -> Result<(), Failure> {
    let edges = convex_edges_at_vertex(vertex, analysis, offset)?;
    let ancestors = analysis.ancestors(&vertex.clone().into_shape())?;
    if edges.len() < 3 || edges.len() != ancestors.len() {
        return Ok(());
    }
    let connectors = connector_edges(vertex, &edges, joins)?;
    let image = BRepOffsetOffset::from_vertex(vertex, &connectors, offset)?;
    result.push(SphericalVertexJoin {
        source_vertex: vertex.clone(),
        face: image.face().clone(),
    });
    Ok(())
}

fn convex_edges_at_vertex(
    vertex: &Vertex,
    analysis: &BRepOffsetAnalyse,
    offset: f64,
) -> Result<Vec<Shape>, Failure> {
    let kind = offset_concavity(offset);
    let mut edges = Vec::new();
    analysis.edges_on_vertex(vertex, kind, &mut edges)?;
    Ok(edges)
}

fn offset_concavity(offset: f64) -> TypeOfConcavity {
    if offset < 0.0 {
        return TypeOfConcavity::Concave;
    }
    TypeOfConcavity::Convex
}

fn connector_edges(
    vertex: &Vertex,
    source_edges: &[Shape],
    joins: &[SharedSupportJoin],
) -> Result<Vec<Edge>, Failure> {
    source_edges
        .iter()
        .map(|source| connector_edge(vertex, source, joins))
        .collect()
}

fn connector_edge(
    vertex: &Vertex,
    source: &Shape,
    joins: &[SharedSupportJoin],
) -> Result<Edge, Failure> {
    let join = joins
        .iter()
        .find(|join| join.source_edge.is_same(source))
        .ok_or_else(|| unsupported_vertex_join(vertex, source))?;
    let image = join
        .endpoint_connectors
        .iter()
        .find(|(source_vertex, _)| source_vertex.is_same(&vertex.clone().into_shape()))
        .map(|(_, edge)| edge.clone())
        .ok_or_else(|| unsupported_vertex_join(vertex, source))?;
    Ok(Edge::unchecked(image.reversed()))
}

fn unsupported_vertex_join(vertex: &Vertex, edge: &Shape) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        &format!(
            "source edge {:?} has no endpoint connector for vertex {:?}; expected a generated offset rail connector",
            edge.shape_type(),
            vertex.clone().into_shape().shape_type()
        ),
        "BRepOffset_MakeOffset::BuildOffsetByArc",
    )
}
