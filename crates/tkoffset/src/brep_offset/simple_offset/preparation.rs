//! The ordered geometry-map construction used by the modification constructor.
use super::{OffsetEdgeGeometry, OffsetVertexGeometry, SimpleOffsetFaceData};
use crate::brep_offset::{
    OffsetFaceSurface, build_simple_offset_edge, build_simple_offset_face_data,
    build_simple_offset_vertices,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn offset_faces(
    shape: &Shape,
    offset: f64,
    tolerance: f64,
) -> Result<DataMap<Face, SimpleOffsetFaceData>, Failure> {
    let mut faces = DataMap::default();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while explorer.more() {
        let face = Face::unchecked(explorer.current().clone());
        let geometry = build_simple_offset_face_data(&face, offset, tolerance)?;
        faces.bind(face, geometry);
        explorer.next()?;
    }
    Ok(faces)
}

pub(super) fn offset_edges(
    shape: &Shape,
    faces: &DataMap<Face, SimpleOffsetFaceData>,
) -> Result<DataMap<Edge, OffsetEdgeGeometry>, Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    let mut edges = DataMap::default();
    for index in 1..=ancestors.extent() {
        let edge = Edge::unchecked(ancestors.find_key(index)?.clone());
        let adjacent = offset_ancestors(ancestors.find_from_index(index)?, faces)?;
        if let Some(geometry) = build_simple_offset_edge(&edge, &adjacent)? {
            edges.bind(edge, geometry);
        }
    }
    Ok(edges)
}

fn offset_ancestors(
    ancestors: &List<Shape>,
    faces: &DataMap<Face, SimpleOffsetFaceData>,
) -> Result<Vec<OffsetFaceSurface>, Failure> {
    if ancestors
        .iter()
        .next()
        .is_some_and(|shape| !faces.is_bound(&Face::unchecked(shape.clone())))
    {
        return Ok(Vec::new());
    }
    let mut adjacent: Vec<OffsetFaceSurface> = Vec::new();
    for shape in ancestors.iter() {
        let face = Face::unchecked(shape.clone());
        if !faces.is_bound(&face) {
            continue;
        }
        adjacent.push(OffsetFaceSurface {
            surface: faces.find(&face)?.offset_surface.clone(),
            face,
        });
    }
    Ok(adjacent)
}

pub(super) fn offset_vertices(
    shape: &Shape,
    edges: &DataMap<Edge, OffsetEdgeGeometry>,
) -> Result<DataMap<Vertex, OffsetVertexGeometry>, Failure> {
    let edge_offsets: Vec<(Edge, Option<OffsetEdgeGeometry>)> = edges
        .items()
        .map(|(edge, geometry)| (edge.clone(), Some(geometry.clone())))
        .collect();
    let computed = build_simple_offset_vertices(shape, &edge_offsets)?;
    let mut vertices = DataMap::default();
    for result in computed {
        vertices.bind(result.vertex, result.geometry);
    }
    Ok(vertices)
}
