//! Deterministic shape and mesh fixtures for the PopulateRegression parity cases.
use super::support;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::*,
    brep_graph_inc::populate,
    topexp::Explorer,
    topods::{self, Builder, Compound, Face, Shape, ShapeIterator, Vertex},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::{Point3, Transform3},
    poly::{PolygonOnTriangulation, Triangle, Triangulation},
    toploc::Location,
};

pub(super) fn translation(x: f64, y: f64, z: f64) -> Location {
    let mut transform = Transform3::default();
    transform.set_translation(rustycad_tkmath::gp::Vector3::new(x, y, z));
    Location::new(transform)
}

pub(super) fn subshapes(shape: &Shape, kind: ShapeEnum) -> Vec<Shape> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut result = Vec::new();
    while explorer.more() {
        result.push(explorer.current().clone());
        explorer.next().unwrap();
    }
    result
}

pub(super) fn face_vertices(face: &Face) -> Vec<Vertex> {
    let mut result: Vec<Vertex> = Vec::new();
    for vertex in subshapes(&face.clone().into_shape(), ShapeEnum::Vertex) {
        let vertex = topods::Vertex::unchecked(vertex);
        if result.iter().all(|known| !known.is_same(&vertex)) {
            result.push(vertex);
        }
    }
    result
}

pub(super) fn node_for_vertex(vertices: &[Vertex], vertex: &Vertex) -> i32 {
    vertices
        .iter()
        .position(|known| known.is_same(vertex))
        .map(|index| index as i32 + 1)
        .expect("face edge endpoint has a face vertex")
}

// Pinned OCCT BRepMesh_IncrementalMesh at the source deflections produces four corner
// nodes and two triangles per box face. The recorded node ordering and diagonals below
// match those native records; edge polygons reference the same vertex nodes.
pub(super) fn attach_box_mesh(shape: &Shape) {
    let builder = BRepBuilder::default();
    for face_shape in subshapes(shape, ShapeEnum::Face) {
        let face = topods::Face::unchecked(face_shape);
        let mut vertices = face_vertices(&face);
        assert_eq!(vertices.len(), 4);
        let points: Vec<Point3> = vertices
            .iter()
            .map(|vertex| BRepTool::pnt(vertex).unwrap())
            .collect();
        let axis = (1..=3)
            .find(|axis| {
                points
                    .iter()
                    .all(|point| point.coord(*axis).unwrap() == points[0].coord(*axis).unwrap())
            })
            .expect("each box face is planar and axis aligned");
        let (first_axis, second_axis) = match axis {
            1 => (2, 3),
            2 => (1, 3),
            _ => (2, 1),
        };
        vertices.sort_by(|left, right| {
            let left = BRepTool::pnt(left).unwrap();
            let right = BRepTool::pnt(right).unwrap();
            left.coord(first_axis)
                .unwrap()
                .total_cmp(&right.coord(first_axis).unwrap())
                .then_with(|| {
                    left.coord(second_axis)
                        .unwrap()
                        .total_cmp(&right.coord(second_axis).unwrap())
                })
        });
        let triangulation = Handle::new(Triangulation::with_sizes(4, 2, false, false).unwrap());
        for (index, vertex) in vertices.iter().enumerate() {
            triangulation
                .set_node(index as i32 + 1, BRepTool::pnt(vertex).unwrap())
                .unwrap();
        }
        let triangles = if axis == 1 {
            [Triangle::new(2, 1, 3), Triangle::new(2, 3, 4)]
        } else {
            [Triangle::new(4, 2, 1), Triangle::new(4, 1, 3)]
        };
        triangulation.set_triangle(1, triangles[0]).unwrap();
        triangulation.set_triangle(2, triangles[1]).unwrap();
        triangulation.update_cached_min_max();
        builder
            .update_face_triangulation(&face, &triangulation, true)
            .unwrap();
        for edge_shape in subshapes(&face.clone().into_shape(), ShapeEnum::Edge) {
            let edge = topods::Edge::unchecked(edge_shape);
            let endpoints: Vec<Vertex> =
                ShapeIterator::new(&edge.clone().into_shape(), false, false)
                    .filter(|child| child.shape_type().ok() == Some(ShapeEnum::Vertex))
                    .map(topods::Vertex::unchecked)
                    .collect();
            if endpoints.len() != 2 {
                continue;
            }
            let first = node_for_vertex(&vertices, &endpoints[0]);
            let last = node_for_vertex(&vertices, &endpoints[1]);
            let polygon = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
            polygon.set_node(1, first).unwrap();
            polygon.set_node(2, last).unwrap();
            builder
                .update_edge_polygon_on_triangulation(
                    &edge,
                    &polygon,
                    &triangulation,
                    &Location::default(),
                )
                .unwrap();
        }
    }
}

pub(super) fn box_shape(x: f64, y: f64, z: f64, mesh: bool) -> Shape {
    let shape = support::make_box(x, y, z).into_shape();
    if mesh {
        attach_box_mesh(&shape);
    }
    shape
}

pub(super) fn nested_moved_box(mesh: bool) -> (Shape, Shape, Shape) {
    let solid = box_shape(2.0, 3.0, 4.0, mesh);
    let moved_solid = solid.moved(&translation(5.0, 0.0, 0.0), false).unwrap();
    let mut inner = Compound::default();
    Builder.make_compound(&mut inner);
    Builder.add(&mut inner, &moved_solid).unwrap();
    let moved_inner = inner
        .into_shape()
        .moved(&translation(0.0, 7.0, 0.0), false)
        .unwrap();
    let mut root = Compound::default();
    Builder.make_compound(&mut root);
    Builder.add(&mut root, &moved_inner).unwrap();
    let moved_root = root
        .into_shape()
        .moved(&translation(0.0, 0.0, 11.0), false)
        .unwrap();
    let solid_source = subshapes(&moved_root, ShapeEnum::Solid).remove(0);
    let face_source = subshapes(&moved_root, ShapeEnum::Face).remove(0);
    (moved_root, solid_source, face_source)
}

pub(super) fn nested_locations(graph: &BRepGraph, root: NodeId) -> (Location, Location) {
    let root_compound = CompoundId::new(root.index);
    let root_refs = &graph
        .topo()
        .compounds()
        .relations(root_compound)
        .child_ref_ids;
    let inner_ref = root_refs[0];
    let inner_node = graph.refs().children().entry(inner_ref).child_node_id;
    let inner_refs = &graph
        .topo()
        .compounds()
        .relations(CompoundId::new(inner_node.index))
        .child_ref_ids;
    (
        graph.refs().generic().local_location(inner_ref.into()),
        graph.refs().generic().local_location(inner_refs[0].into()),
    )
}

pub(super) fn assert_translation(location: &Location, expected: [f64; 3]) {
    assert_eq!(
        location.transformation().translation_part().coordinates(),
        &expected
    );
}

pub(super) fn populate_shape(shape: &Shape) -> BRepGraph {
    let mut graph = BRepGraph::new();
    let status = populate::perform(&mut graph, shape, false, &populate::Options).unwrap();
    assert_ne!(status, populate::BuildStatus::Failed);
    graph
}

pub(super) fn assert_graph_box_bounds(graph: &BRepGraph, min: Point3, max: Point3) {
    let mut points = Vec::new();
    for index in 0..graph.topo().vertices().nb() {
        points.push(
            graph
                .topo()
                .vertices()
                .definition(VertexId::new(index))
                .point,
        );
    }
    for (axis, low, high) in [
        (1, min.x(), max.x()),
        (2, min.y(), max.y()),
        (3, min.z(), max.z()),
    ] {
        let values: Vec<f64> = points
            .iter()
            .map(|point| point.coord(axis).unwrap())
            .collect();
        assert!((values.iter().copied().fold(f64::INFINITY, f64::min) - low).abs() < 1e-6);
        assert!((values.iter().copied().fold(f64::NEG_INFINITY, f64::max) - high).abs() < 1e-6);
    }
}
