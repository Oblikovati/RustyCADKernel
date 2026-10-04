//! Native repeated-edge, face-map and singleton drop regressions for `MergeSmallEdges`.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Builder, Compound, Edge, Face, Shape, Vertex, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPICopy, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

#[test]
fn multiedge_neighbor_is_skipped_like_native() {
    assert_eq!(multiedge_neighbor_row(), expected("multiple_neighbor_skip"));
}

#[test]
fn repeated_topological_edge_is_skipped_like_native() {
    assert_eq!(repeated_edge_row(), expected("repeated_edge_guard"));
}

#[test]
fn separate_face_groups_follow_shape_order_and_update_maps() {
    assert_eq!(multiple_face_groups_row(), expected("multiple_face_groups"));
}

#[test]
fn singleton_face_edge_drop_keeps_native_stale_face_map() {
    assert_eq!(face_drop_single_row(), expected("face_drop_single"));
}

#[test]
fn internal_vertex_context_remap_matches_native() {
    assert_eq!(internal_vertex_row(), expected("internal_vertex_transfer"));
}

fn internal_vertex_row() -> Vec<f64> {
    let (shape, source_edge, internal) = wire_with_internal_vertex();
    let fixer = configured_wireframe(&shape);
    fixer.set_precision(0.1);
    let mut small = shape_map(&source_edge.into_shape());
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    let mut multiple = MapOfShape::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        false,
    );
    let applied = applied_shape(&fixer, &shape);
    let replaced = ReShape::apply(
        &*fixer.context(),
        &internal.clone().into_shape(),
        ShapeEnum::Shape,
    )
    .unwrap();
    let edges = edge_shapes(&applied);
    let vertex_count = Explorer::new(&applied, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .count();
    let internal_count = edges.iter().map(count_internal_vertices).sum::<usize>();
    let attached = edges.iter().any(|edge| contains_vertex(edge, &replaced));
    vec![
        f64::from(changed),
        edges.len() as f64,
        internal_count as f64,
        vertex_count as f64,
        f64::from(attached),
        f64::from(!replaced.is_same(&internal.clone().into_shape())),
        done_status(&fixer),
    ]
}

fn multiedge_neighbor_row() -> Vec<f64> {
    let shape = tiny_wire().into_shape();
    let fixer = configured_wireframe(&shape);
    let edges = edge_shapes(&shape);
    let mut small = shape_map(&edges[0]);
    let mut multiple = shape_map(&edges[3]);
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        false,
    );
    let result = applied_shape(&fixer, &shape);
    vec![
        f64::from(changed),
        small.extent() as f64,
        multiple.extent() as f64,
        edge_count(&result),
        done_status(&fixer),
    ]
}

fn repeated_edge_row() -> Vec<f64> {
    let (wire, edge) = repeated_edge_wire();
    let shape = wire.into_shape();
    let fixer = configured_wireframe(&shape);
    let mut small = shape_map(&edge.into_shape());
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    let mut multiple = MapOfShape::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        false,
    );
    let result = applied_shape(&fixer, &shape);
    vec![
        f64::from(changed),
        small.extent() as f64,
        edge_count(&result),
        done_status(&fixer),
    ]
}

fn multiple_face_groups_row() -> Vec<f64> {
    let (shape, first, second) = two_face_compound();
    let fixer = configured_wireframe(&shape);
    let (mut small, mut edge_faces, mut face_small) = face_edge_maps(&[&first, &second]);
    let mut multiple = MapOfShape::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        false,
    );
    let result = applied_shape(&fixer, &shape);
    vec![
        f64::from(changed),
        small.extent() as f64,
        edge_faces.extent() as f64,
        face_small.extent() as f64,
        edge_count(&result),
        done_status(&fixer),
    ]
}

fn face_drop_single_row() -> Vec<f64> {
    let face = plane_face(&single_tiny_wire());
    let shape = face.clone().into_shape();
    let fixer = configured_wireframe(&shape);
    fixer.set_precision(0.1);
    let (mut small, mut edge_faces, mut face_small) = face_edge_maps(&[&face]);
    let mut multiple = MapOfShape::default();
    let changed = merge(
        &fixer,
        &mut small,
        &mut edge_faces,
        &mut face_small,
        &mut multiple,
        true,
    );
    let result = applied_shape(&fixer, &shape);
    vec![
        f64::from(changed),
        small.extent() as f64,
        edge_faces.extent() as f64,
        face_small.extent() as f64,
        f64::from(result.is_null()),
        edge_count(&result),
        f64::from(fixer.status_small_edges(ShapeExtendStatus::Done2)),
    ]
}

fn configured_wireframe(shape: &Shape) -> ShapeFixWireframe {
    let fixer = ShapeFixWireframe::from_shape(shape);
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    fixer
}

fn merge(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    edge_faces: &mut ShapeFixWireframeShapeLists,
    face_small: &mut ShapeFixWireframeShapeLists,
    multiple: &mut MapOfShape,
    drop_mode: bool,
) -> bool {
    fixer
        .merge_small_edges(small, edge_faces, face_small, multiple, drop_mode, -1.0)
        .unwrap()
}

fn applied_shape(fixer: &ShapeFixWireframe, shape: &Shape) -> Shape {
    ReShape::apply(&*fixer.context(), shape, ShapeEnum::Shape).unwrap()
}

fn edge_shapes(shape: &Shape) -> Vec<Shape> {
    Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect()
}

fn edge_count(shape: &Shape) -> f64 {
    edge_shapes(shape).len() as f64
}

fn count_internal_vertices(edge: &Shape) -> usize {
    Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .filter(|vertex| vertex.orientation() == rustycad_tkg3d::top_abs::Orientation::Internal)
        .count()
}

fn contains_vertex(edge: &Shape, vertex: &Shape) -> bool {
    Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)
        .unwrap()
        .any(|candidate| candidate.is_same(vertex))
}

fn done_status(fixer: &ShapeFixWireframe) -> f64 {
    f64::from(fixer.status_small_edges(ShapeExtendStatus::Done))
}

fn shape_map(shape: &Shape) -> MapOfShape {
    let mut map = MapOfShape::default();
    map.add(shape.clone());
    map
}

fn repeated_edge_wire() -> (Wire, rustycad_tkbrep::topods::Edge) {
    let edges = edge_shapes(&tiny_wire().into_shape());
    let repeated = topods::edge(&edges[0]).unwrap();
    let neighbor = topods::edge(&edges[1]).unwrap();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &repeated).unwrap();
    Builder.add(&mut wire, &repeated).unwrap();
    Builder.add(&mut wire, &neighbor).unwrap();
    (wire, repeated)
}

fn wire_with_internal_vertex() -> (Shape, Edge, Vertex) {
    let source = tiny_wire();
    let source_shape = source.clone().into_shape();
    let source_edges = edge_shapes(&source_shape);
    let mut copy = BRepBuilderAPICopy::with_shape(&source_edges[0], true, false).unwrap();
    let mut small_edge = topods::edge(copy.shape().unwrap()).unwrap();
    let builder = BRepBuilder::default();
    let mut internal = Vertex::default();
    builder
        .make_vertex_at(&mut internal, Point3::new(0.005, 0.0, 0.0), 1e-7)
        .unwrap();
    internal.set_orientation(rustycad_tkg3d::top_abs::Orientation::Internal);
    builder.add(&mut small_edge, &internal).unwrap();
    let (first, last) = BRepTool::range(&small_edge).unwrap();
    builder
        .update_vertex_on_edge(&internal, (first + last) * 0.5, &small_edge, 1e-7)
        .unwrap();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &small_edge).unwrap();
    for edge_shape in source_edges.into_iter().skip(1) {
        Builder
            .add(&mut wire, &topods::edge(&edge_shape).unwrap())
            .unwrap();
    }
    (wire.into_shape(), small_edge, internal)
}

fn two_face_compound() -> (Shape, Face, Face) {
    let first = plane_face(&tiny_wire());
    let second = plane_face(&tiny_wire());
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, &first).unwrap();
    Builder.add(&mut compound, &second).unwrap();
    (compound.into_shape(), first, second)
}

fn plane_face(wire: &Wire) -> Face {
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), wire, true)
        .unwrap()
        .face()
        .unwrap()
}

fn face_edge_maps(
    faces: &[&Face],
) -> (
    MapOfShape,
    ShapeFixWireframeShapeLists,
    ShapeFixWireframeShapeLists,
) {
    let mut small = MapOfShape::default();
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    for face in faces {
        add_face_links(face, &mut small, &mut edge_faces, &mut face_small);
    }
    (small, edge_faces, face_small)
}

fn add_face_links(
    face: &Face,
    small: &mut MapOfShape,
    edge_faces: &mut ShapeFixWireframeShapeLists,
    face_small: &mut ShapeFixWireframeShapeLists,
) {
    let face_shape = face.clone().into_shape();
    let mut face_edges = List::default();
    let mut first = true;
    for edge in edge_shapes(&face_shape) {
        edge_faces.bind(edge.clone(), List::from_iter([face_shape.clone()]));
        if first {
            small.add(edge.clone());
            face_edges.append(edge);
            first = false;
        }
    }
    face_small.bind(face_shape, face_edges);
}

fn tiny_wire() -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

fn single_tiny_wire() -> Wire {
    BRepBuilderAPIMakePolygon::from_two_points(Point3::new(0., 0., 0.), Point3::new(0.01, 0., 0.))
        .unwrap()
        .wire()
        .unwrap()
}

fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_fix_small_edges.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
