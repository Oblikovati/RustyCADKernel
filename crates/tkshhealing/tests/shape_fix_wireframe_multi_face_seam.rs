//! Native two-face seam p-curve rejection path for MergeSmallEdges.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{Builder, Compound, Edge, Face, Shape, Vertex, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{ncollection::List, precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomCylindricalSurface, GeomLine},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::{
    gp::{Axis1, Axis3, Direction2, Direction3, Point2, Point3, Transform3},
    toploc::Location,
};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge,
    shape_build::ShapeBuildReShape,
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};

#[test]
fn native_declines_join_when_second_located_seam_support_is_incompatible() {
    let (shape, first_face, second_face, edges) = multi_face_seam_fixture();
    assert_eq!(
        input_seam_row(&edges[..2], &first_face, &second_face),
        vec![1, 1, 1, 1]
    );
    let fixer = ShapeFixWireframe::from_shape(&shape);
    fixer.set_precision(0.1);
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    let mut small = shape_map(&edges[1].clone().into_shape());
    let mut multiple = MapOfShape::default();
    let (mut edge_faces, mut face_small) = seam_maps(&first_face, &second_face, &edges);
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
    assert_eq!(
        multi_face_row(&fixer, &shape, &first_face, &second_face, changed),
        expected_row()
    );
}

fn input_seam_row(edges: &[Edge], first_face: &Face, second_face: &Face) -> Vec<i32> {
    let analyzer = ShapeAnalysisEdge::new();
    edges
        .iter()
        .flat_map(|edge| {
            [first_face, second_face]
                .map(|face| i32::from(analyzer.is_seam_on_face(edge, face).unwrap()))
        })
        .collect()
}

fn multi_face_seam_fixture() -> (Shape, Face, Face, [Edge; 3]) {
    let builder = BRepBuilder::default();
    let (mut first_face, mut second_face) = cylinder_faces(&builder);
    let edges = seam_edges(&builder, &first_face, &second_face);
    add_seam_wire(&mut first_face, &edges);
    add_seam_wire(&mut second_face, &edges);
    let shape = compound_faces(&first_face, &second_face);
    (shape, first_face, second_face, edges)
}

fn cylinder_faces(builder: &BRepBuilder) -> (Face, Face) {
    let surface = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap());
    let mut first_face = Face::default();
    builder
        .make_face_on_surface(&mut first_face, &surface.cast(), precision::confusion())
        .unwrap();
    let transform =
        Transform3::rotation(Axis1::new(Point3::default(), Direction3::Z), 0.7).unwrap();
    let location = Location::new(transform);
    let mut second_face = Face::default();
    builder
        .make_face_on_located_surface(
            &mut second_face,
            &surface.cast(),
            &location,
            precision::confusion(),
        )
        .unwrap();
    (first_face, second_face)
}

fn seam_edges(builder: &BRepBuilder, first: &Face, second: &Face) -> [Edge; 3] {
    [
        seam_edge(builder, first, second, 0.0, 1.0),
        seam_edge(builder, first, second, 1.0, 1.001),
        seam_edge(builder, first, second, 0.0, 1.001),
    ]
}

fn compound_faces(first: &Face, second: &Face) -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, first).unwrap();
    Builder.add(&mut compound, second).unwrap();
    compound.into_shape()
}

fn seam_edge(
    builder: &BRepBuilder,
    first_face: &Face,
    second_face: &Face,
    first: f64,
    last: f64,
) -> Edge {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(1.0, 0.0, 0.0),
        Direction3::Z,
    ))
    .cast();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve, precision::confusion())
        .unwrap();
    add_endpoints(builder, &mut edge, first, last);
    builder.range(&edge, first, last, true).unwrap();
    add_face_seams(builder, &edge, first_face, second_face, first, last);
    edge
}

fn add_face_seams(
    builder: &BRepBuilder,
    edge: &Edge,
    first_face: &Face,
    second_face: &Face,
    first: f64,
    last: f64,
) {
    let first_pair = (pcurve(0.0, first), pcurve(std::f64::consts::TAU, first));
    builder
        .update_edge_pcurves_on_face(edge, (&first_pair.0, &first_pair.1), first_face, 0.0)
        .unwrap();
    let shifted_u = -0.7;
    let second_pair = (
        pcurve(shifted_u, first),
        pcurve(shifted_u + std::f64::consts::TAU, first),
    );
    builder
        .update_edge_pcurves_on_face(edge, (&second_pair.0, &second_pair.1), second_face, 0.0)
        .unwrap();
    builder
        .range_on_face(edge, first_face, first, last)
        .unwrap();
    builder
        .range_on_face(edge, second_face, first, last)
        .unwrap();
}

fn add_endpoints(builder: &BRepBuilder, edge: &mut Edge, first: f64, last: f64) {
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

fn add_seam_wire(face: &mut Face, edges: &[Edge; 3]) {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    Builder.add(&mut wire, &edges[0]).unwrap();
    Builder.add(&mut wire, &edges[1]).unwrap();
    Builder
        .add(&mut wire, &edges[2].oriented(Orientation::Reversed))
        .unwrap();
    Builder.add(face, &wire).unwrap();
}

fn pcurve(u: f64, first: f64) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(u, first), Direction2::Y)).cast()
}

fn seam_maps(
    first: &Face,
    second: &Face,
    edges: &[Edge; 3],
) -> (ShapeFixWireframeShapeLists, ShapeFixWireframeShapeLists) {
    let faces = [first.clone().into_shape(), second.clone().into_shape()];
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    for edge in edges {
        edge_faces.bind(
            edge.clone().into_shape(),
            List::from_iter(faces.iter().cloned()),
        );
    }
    let mut face_small = ShapeFixWireframeShapeLists::default();
    face_small.bind(
        faces[0].clone(),
        List::from_iter([edges[1].clone().into_shape()]),
    );
    (edge_faces, face_small)
}

fn multi_face_row(
    fixer: &ShapeFixWireframe,
    source: &Shape,
    first_face: &Face,
    second_face: &Face,
    changed: bool,
) -> Vec<i32> {
    let result = ReShape::apply(&*fixer.context(), source, ShapeEnum::Shape).unwrap();
    let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect::<Vec<_>>();
    let analyzer = ShapeAnalysisEdge::new();
    let first_seams = seam_count(&analyzer, &edges, first_face);
    let second_seams = seam_count(&analyzer, &edges, second_face);
    vec![
        i32::from(changed),
        edges.len() as i32,
        first_seams as i32,
        second_seams as i32,
        i32::from(fixer.status_small_edges(ShapeExtendStatus::Done)),
        i32::from(fixer.status_small_edges(ShapeExtendStatus::Fail1)),
    ]
}

fn seam_count(analyzer: &ShapeAnalysisEdge, edges: &[Shape], face: &Face) -> usize {
    edges
        .iter()
        .filter_map(|shape| rustycad_tkbrep::topods::edge(shape).ok())
        .filter(|edge| analyzer.is_seam_on_face(edge, face).unwrap())
        .count()
}

fn shape_map(shape: &Shape) -> MapOfShape {
    let mut map = MapOfShape::default();
    map.add(shape.clone());
    map
}

fn expected_row() -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_multi_face_seam.expected")
        .lines()
        .nth(1)
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
