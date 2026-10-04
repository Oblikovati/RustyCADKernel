use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Protects kept shapes and shared junctions before the global edge pass.
pub(crate) fn protected_shapes(shape: &Shape, keep: &[Shape]) -> Result<Vec<Shape>, Failure> {
    let mut protected = keep.to_vec();
    let shared = shared_junctions(shape)?;
    for vertex in shared {
        if !protected.iter().any(|known| known.is_same(&vertex)) {
            protected.push(vertex);
        }
    }
    Ok(protected)
}

fn shared_junctions(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    let edges = unique_edges(shape)?;
    let analysis = ShapeAnalysisEdge::new();
    let mut vertices = Vec::<(Shape, Vec<Edge>)>::new();
    for edge in edges {
        add_incident_edge(&mut vertices, analysis.first_vertex(&edge), &edge);
        add_incident_edge(&mut vertices, analysis.last_vertex(&edge), &edge);
    }
    Ok(vertices
        .into_iter()
        .filter(|(_, incident)| incident.len() > 2)
        .map(|(vertex, _)| vertex)
        .collect())
}

pub(super) fn unique_edges(shape: &Shape) -> Result<Vec<Edge>, Failure> {
    let mut unique = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = rustycad_tkbrep::topods::edge(&candidate)?;
        if !unique.iter().any(|known: &Edge| known.is_same(&edge)) {
            unique.push(edge);
        }
    }
    Ok(unique)
}

fn add_incident_edge(vertices: &mut Vec<(Shape, Vec<Edge>)>, vertex: Vertex, edge: &Edge) {
    if vertex.is_null() {
        return;
    }
    let shape = vertex.into_shape();
    let Some((_, incident)) = vertices.iter_mut().find(|(known, _)| known.is_same(&shape)) else {
        vertices.push((shape, vec![edge.clone()]));
        return;
    };
    if !incident.iter().any(|known| known.is_same(edge)) {
        incident.push(edge.clone());
    }
}
