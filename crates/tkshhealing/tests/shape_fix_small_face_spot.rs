//! Native-backed no-change behavior of the `ShapeFix_FixSmallFace` spot phase.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixFixSmallFace},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn regular_box_is_preserved_by_the_spot_phase() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_fix_small_face_spot.expected")
        .lines()
        .find(|row| row.starts_with("regular_solid "))
        .unwrap();
    assert_eq!(expected, "regular_solid 1 1 2 6 24 48");
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let input = box_maker.shape().unwrap().clone();
    let fixer = ShapeFixFixSmallFace::new(input.clone());
    let result = fixer.fix_spot_face().unwrap();
    assert!(result.is_same(&input));
    assert!(fixer.shape().is_same(&input));
    assert!(fixer.fix_shape().unwrap().is_same(&input));
    let mut first_face = first_face(&input);
    let before = first_face.clone().into_shape();
    assert!(fixer.fix_pin_face(&mut first_face));
    assert!(first_face.into_shape().is_same(&before));
}

#[test]
fn spot_face_is_removed_from_a_compound_and_regular_faces_survive() {
    let native = include_str!("../../../tools/oracle/tkshhealing_fix_small_face_spot.expected")
        .lines()
        .find(|row| row.starts_with("tiny_and_regular "))
        .unwrap();
    assert_eq!(native, "tiny_and_regular 0 0 0 6 24 48");
    let input = compound_with_tiny_face_and_regular_box();
    let fixer = ShapeFixFixSmallFace::new(input.clone());
    let output = fixer.fix_spot_face().unwrap();
    assert!(!output.is_same(&input));
    assert_eq!(output.shape_type().unwrap(), ShapeEnum::Compound);
    assert_eq!(count_shapes(&output, ShapeEnum::Face), 6);
    assert_eq!(count_shapes(&output, ShapeEnum::Edge), 24);
    assert_eq!(count_shapes(&output, ShapeEnum::Vertex), 48);
    assert!(fixer.status(ShapeExtendStatus::Done1));
}

#[test]
fn strip_face_replacement_matches_thin_box_topology() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_fix_small_face_spot.expected")
        .lines()
        .find(|row| row.starts_with("thin_solid "))
        .unwrap();
    let native_bounds = expected
        .split_whitespace()
        .skip(7)
        .map(|value| value.parse::<f64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(native_bounds.len(), 6);
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0e-4, 1.0e-4).unwrap();
    let input = box_maker.shape().unwrap().clone();
    let fixer = ShapeFixFixSmallFace::new(input.clone());
    FixRoot::set_precision(&fixer, 0.001);
    let output = fixer.fix_strip_face(false).unwrap();
    assert!(!output.is_same(&input));
    assert_eq!(output.shape_type().unwrap(), ShapeEnum::Solid);
    assert_eq!(count_shapes(&output, ShapeEnum::Face), 3);
    assert_eq!(count_shapes(&output, ShapeEnum::Edge), 6);
    assert_eq!(count_shapes(&output, ShapeEnum::Vertex), 12);
    let actual_bounds = vertex_bounds(&output);
    assert_eq!(actual_bounds, native_bounds.as_slice());
    assert!(fixer.status(ShapeExtendStatus::Done2));
}

fn compound_with_tiny_face_and_regular_box() -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let mut compound_shape = compound.into_shape();
    Builder
        .add(&mut compound_shape, &tiny_circle_face().into_shape())
        .unwrap();
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let solid = box_maker.shape().unwrap().clone();
    Builder.add(&mut compound_shape, &solid).unwrap();
    compound_shape
}

fn tiny_circle_face() -> Face {
    let builder = BRepBuilder::default();
    let circle = GeomCircle::new(
        Axis2::new(Point3::default(), Direction3::Z).unwrap(),
        1.0e-8,
    )
    .unwrap();
    let curve: Handle<dyn GeomCurve> = Handle::new(circle).cast();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1.0e-7)
        .unwrap();
    builder
        .range(&edge, 0.0, std::f64::consts::TAU, true)
        .unwrap();
    attach_shared_circle_vertex(&builder, &mut edge);
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    Builder.add(&mut wire_shape, &edge.into_shape()).unwrap();
    let mut face = Face::default();
    builder.make_face(&mut face);
    let mut face_shape = face.into_shape();
    Builder.add(&mut face_shape, &wire_shape).unwrap();
    topods::face(&face_shape).unwrap()
}

fn attach_shared_circle_vertex(builder: &BRepBuilder, edge: &mut Edge) {
    let mut shared = Vertex::default();
    builder
        .make_vertex_at(&mut shared, Point3::new(1.0e-8, 0.0, 0.0), 1.0e-7)
        .unwrap();
    let mut edge_shape = edge.clone().into_shape();
    Builder
        .add(&mut edge_shape, &shared.oriented(Orientation::Forward))
        .unwrap();
    Builder
        .add(&mut edge_shape, &shared.oriented(Orientation::Reversed))
        .unwrap();
    *edge = topods::edge(&edge_shape).unwrap();
}

fn count_shapes(shape: &Shape, target: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, target, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

fn first_face(shape: &Shape) -> Face {
    let explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    topods::face(explorer.current()).unwrap()
}

fn vertex_bounds(shape: &Shape) -> [f64; 6] {
    let mut bounds = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    let mut vertices = Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap();
    while vertices.more() {
        let vertex = topods::vertex(vertices.current()).unwrap();
        let point = BRepTool::pnt(&vertex).unwrap();
        update_bounds(&mut bounds, point);
        vertices.next().unwrap();
    }
    bounds
}

fn update_bounds(bounds: &mut [f64; 6], point: Point3) {
    bounds[0] = bounds[0].min(point.x());
    bounds[1] = bounds[1].max(point.x());
    bounds[2] = bounds[2].min(point.y());
    bounds[3] = bounds[3].max(point.y());
    bounds[4] = bounds[4].min(point.z());
    bounds[5] = bounds[5].max(point.z());
}
