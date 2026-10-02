//! Translation of the pinned ShapeFix_Face_Test.cxx regression suite.
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Builder, Compound, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCurve, GeomSurface};
use rustycad_tkgeomalgo::geom_api::Interpolate;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFace};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use std::f64::consts::PI;

fn periodic_cone_fixture() -> (Handle<GeomConicalSurface>, Wire) {
    let axis = Axis3::new(
        Point3::new(0.0, 4.0, -9.1),
        Direction3::new(0.0, 0.09, -1.0).unwrap(),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    let cone = Handle::new(GeomConicalSurface::new(axis, 0.35, 0.0).unwrap());
    let mut samples = Array1::with_bounds(1, 8).unwrap();
    for index in 1..=8 {
        samples[index] = cone
            .value(2.0 * PI * (index - 1) as f64 / 8.0, 0.5)
            .unwrap();
    }

    let mut interpolation =
        Interpolate::new(&Handle::new(HArray1::from_owned(samples)), true, 1e-6).unwrap();
    interpolation.perform().unwrap();
    assert!(interpolation.is_done());
    let curve: Handle<dyn GeomCurve> = interpolation.curve().unwrap().cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&curve).unwrap();
    assert!(edge.is_done());
    let mut wire_builder = BRepBuilderAPIMakeWire::new();
    wire_builder.add_edge(&edge.edge().unwrap()).unwrap();
    assert!(wire_builder.is_done());
    let wire = wire_builder.wire().unwrap();
    assert!(rustycad_tkbrep::brep::BRepTool::is_closed(&wire).unwrap());
    (cone, wire)
}

fn periodic_cone_face() -> (Handle<GeomConicalSurface>, Face) {
    let (cone, wire) = periodic_cone_fixture();
    let surface: Handle<dyn GeomSurface> = cone.cast();
    let face = BRepBuilderAPIMakeFace::from_surface_wire(&surface, &wire, true)
        .unwrap()
        .face()
        .unwrap();
    (cone, face)
}

#[test]
fn no_crash_on_periodic_conical_single_wire() {
    let (_, face) = periodic_cone_face();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.perform().unwrap();
    assert!(!fixer.face().unwrap().is_null());
}

#[test]
fn valid_face_on_periodic_conical_single_wire_with_context() {
    let (_, face) = periodic_cone_face();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
    fixer.perform().unwrap();
    let fixed_shape = fixer.face().unwrap().into_shape();
    let analyzer = BRepCheckAnalyzer::new(&fixed_shape, true, false, false).unwrap();
    let mut wires = ShapeIterator::new(&fixed_shape, false, true);
    let face_wire = wires.value().unwrap().clone();
    wires.next();
    assert!(!wires.more());
    let mut edges = ShapeIterator::new(&face_wire, false, true);
    let mut edge_count = 0;
    while edges.more() {
        edge_count += 1;
        edges.next();
    }
    assert_eq!(edge_count, 4);
    assert!(analyzer.is_valid().unwrap());
}

#[test]
fn perform_face_replaced_by_compound_preserves_replacement() {
    let face = planar_face();
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    let replacement: Shape = compound.into_shape();
    let context = Handle::new(ShapeBuildReShape::new());
    context
        .replace(&face.clone().into_shape(), &replacement)
        .unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_context(&context);
    assert!(!fixer.perform().unwrap());
    assert!(fixer.status(ShapeExtendStatus::Ok));
    assert!(fixer.result().is_same(&replacement));
}

#[test]
fn perform_removed_face_preserves_null_replacement() {
    let face = planar_face();
    let context = Handle::new(ShapeBuildReShape::new());
    context.remove(&face.clone().into_shape()).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_context(&context);
    assert!(!fixer.perform().unwrap());
    assert!(fixer.status(ShapeExtendStatus::Ok));
    assert!(fixer.result().is_null());
}

fn planar_face() -> Face {
    BRepLibMakeFace::from_plane_bounds(rustycad_tkmath::gp::Plane3::default(), (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
}
