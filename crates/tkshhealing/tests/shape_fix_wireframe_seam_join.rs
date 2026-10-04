//! Native paired seam-pcurve join regression for `ShapeFix_Wireframe::MergeSmallEdges`.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{Builder, Edge, Face, Shape, Vertex, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{ncollection::List, precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomCylindricalSurface, GeomLine},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge,
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;

#[test]
fn native_seam_pair_join_preserves_both_pcurves() {
    let (face, first, middle, last) = seam_fixture();
    let fixer = ShapeFixWireframe::from_shape(&face.clone().into_shape());
    fixer.set_precision(0.1);
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    let mut small = shape_map(&middle.clone().into_shape());
    let mut multiple = MapOfShape::default();
    let (mut edge_faces, mut face_small) = seam_maps(&face, [&first, &middle, &last]);
    let changed = fixer
        .merge_small_edges(
            &mut small,
            &mut edge_faces,
            &mut face_small,
            &mut multiple,
            false,
            -1.0,
        )
        .unwrap();
    assert_eq!(seam_row(&fixer, &face, changed), expected_row());
}

#[test]
fn joined_edges_remap_old_endpoint_vertices_like_native() {
    assert_eq!(endpoint_context_row(), endpoint_expected_row());
}

fn endpoint_context_row() -> Vec<i32> {
    let wire = endpoint_wire();
    let shape = wire.clone().into_shape();
    let edges = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect::<Vec<_>>();
    let small_edge = rustycad_tkbrep::topods::edge(&edges[0]).unwrap();
    let analysis = ShapeAnalysisEdge::new();
    let source_vertices = (
        analysis.first_vertex(&small_edge),
        analysis.last_vertex(&small_edge),
    );
    let fixer = ShapeFixWireframe::from_shape(&shape);
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    let mut small = shape_map(&small_edge.clone().into_shape());
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    let mut multiple = MapOfShape::default();
    let changed = fixer
        .merge_small_edges(
            &mut small,
            &mut edge_faces,
            &mut face_small,
            &mut multiple,
            false,
            -1.0,
        )
        .unwrap();
    endpoint_observations(&fixer, &shape, source_vertices, changed)
}

fn endpoint_wire() -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.01, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

fn endpoint_observations(
    fixer: &ShapeFixWireframe,
    shape: &Shape,
    source_vertices: (Vertex, Vertex),
    changed: bool,
) -> Vec<i32> {
    let context = fixer.context();
    let result = ReShape::apply(&*context, shape, ShapeEnum::Shape).unwrap();
    let mapped = (
        ReShape::apply(
            &*context,
            &source_vertices.0.clone().into_shape(),
            ShapeEnum::Shape,
        )
        .unwrap(),
        ReShape::apply(
            &*context,
            &source_vertices.1.clone().into_shape(),
            ShapeEnum::Shape,
        )
        .unwrap(),
    );
    let result_edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect::<Vec<_>>();
    let vertex_occurrences = result_edges.iter().map(edge_vertex_count).sum::<usize>();
    vec![
        i32::from(changed),
        result_edges.len() as i32,
        vertex_occurrences as i32,
        i32::from(!mapped.0.is_same(&source_vertices.0.clone().into_shape())),
        i32::from(!mapped.1.is_same(&source_vertices.1.clone().into_shape())),
        i32::from(edges_include_vertex(&result_edges, &mapped.0)),
        i32::from(edges_include_vertex(&result_edges, &mapped.1)),
    ]
}

fn edge_vertex_count(edge: &rustycad_tkbrep::topods::Shape) -> usize {
    Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn edges_include_vertex(edges: &[Shape], vertex: &Shape) -> bool {
    edges.iter().any(|edge| {
        Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)
            .unwrap()
            .any(|candidate| candidate.is_same(vertex))
    })
}

fn seam_fixture() -> (Face, Edge, Edge, Edge) {
    let builder = BRepBuilder::default();
    let surface = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap());
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, &surface.cast(), precision::confusion())
        .unwrap();
    let first = seam_edge(&builder, &face, 0.0, 1.0);
    let middle = seam_edge(&builder, &face, 1.0, 1.001);
    let last = seam_edge(&builder, &face, 0.0, 1.001);
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &first).unwrap();
    Builder.add(&mut wire, &middle).unwrap();
    Builder
        .add(&mut wire, &last.oriented(Orientation::Reversed))
        .unwrap();
    Builder.add(&mut face, &wire).unwrap();
    (face, first, middle, last)
}

fn seam_edge(builder: &BRepBuilder, face: &Face, first: f64, last: f64) -> Edge {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::Z,
    ))
    .cast();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve, precision::confusion())
        .unwrap();
    add_seam_endpoints(builder, &mut edge, first, last);
    builder.range(&edge, first, last, true).unwrap();
    let left = pcurve(0.0, first);
    let right = pcurve(std::f64::consts::TAU, first);
    builder
        .update_edge_pcurves_on_face(&edge, (&left, &right), face, 0.0)
        .unwrap();
    builder.range_on_face(&edge, face, first, last).unwrap();
    edge
}

fn add_seam_endpoints(builder: &BRepBuilder, edge: &mut Edge, first: f64, last: f64) {
    let first_vertex = vertex_at(builder, first);
    let last_vertex = vertex_at(builder, last);
    Builder
        .add(edge, &first_vertex.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(edge, &last_vertex.oriented(Orientation::Reversed))
        .unwrap();
    builder
        .update_vertex_on_edge(&first_vertex, first, edge, precision::confusion())
        .unwrap();
    builder
        .update_vertex_on_edge(&last_vertex, last, edge, precision::confusion())
        .unwrap();
}

fn vertex_at(builder: &BRepBuilder, z: f64) -> Vertex {
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(
            &mut vertex,
            Point3::new(1.0, 0.0, z),
            precision::confusion(),
        )
        .unwrap();
    vertex
}

fn pcurve(u: f64, first: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(u, first), Direction2::Y)).cast()
}

fn seam_maps(
    face: &Face,
    edges: [&Edge; 3],
) -> (ShapeFixWireframeShapeLists, ShapeFixWireframeShapeLists) {
    let face = face.clone().into_shape();
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    for edge in edges {
        edge_faces.bind(edge.clone().into_shape(), List::from_iter([face.clone()]));
    }
    let mut face_small = ShapeFixWireframeShapeLists::default();
    face_small.bind(face, List::from_iter([edges[1].clone().into_shape()]));
    (edge_faces, face_small)
}

fn seam_row(fixer: &ShapeFixWireframe, face: &Face, changed: bool) -> Vec<i32> {
    let result = ReShape::apply(
        &*fixer.context(),
        &face.clone().into_shape(),
        ShapeEnum::Shape,
    )
    .unwrap();
    let face = rustycad_tkbrep::topods::face(&result).unwrap();
    let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect::<Vec<_>>();
    let analysis = ShapeAnalysisEdge::new();
    let seams = edges
        .iter()
        .filter_map(|edge| rustycad_tkbrep::topods::edge(edge).ok())
        .filter(|edge| analysis.is_seam_on_face(edge, &face).unwrap())
        .count();
    vec![
        i32::from(changed),
        edges.len() as i32,
        seams as i32,
        i32::from(fixer.status_small_edges(ShapeExtendStatus::Done)),
        i32::from(fixer.status_small_edges(ShapeExtendStatus::Fail1)),
    ]
}

fn shape_map(shape: &Shape) -> MapOfShape {
    let mut map = MapOfShape::default();
    map.add(shape.clone());
    map
}

fn expected_row() -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_seam_join.expected")
        .lines()
        .find(|line| line.starts_with("seam_pair_join "))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}

fn endpoint_expected_row() -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_nonmanifold_vertices.expected")
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
