//! Behaviour tests of the BRepSweep package: the topology of prisms and revolutions,
//! their history and their errors.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Vector3};
use rustycad_tkprim::brep_sweep::{BRepSweepPrism, BRepSweepRevol};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use std::f64::consts::PI;

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut n = 0;
    while explorer.more() {
        n += 1;
        explorer.next().unwrap();
    }
    n
}

fn square() -> Shape {
    let maker =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0)).unwrap();
    maker.face().unwrap().into()
}

fn segment(a: Point3, b: Point3) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(a, b)
        .unwrap()
        .edge()
        .unwrap()
        .into()
}

fn z_axis() -> Axis1 {
    Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
}

#[test]
fn a_prism_of_a_square_is_a_box_solid() {
    let mut prism =
        BRepSweepPrism::new(&square(), Vector3::new(0.0, 0.0, 2.0), false, true).unwrap();
    let solid = prism.shape().unwrap();
    assert_eq!(solid.shape_type().unwrap(), ShapeEnum::Solid);
    assert_eq!(count(&solid, ShapeEnum::Face), 6);
    let top = prism.last_shape().unwrap();
    assert_eq!(top.shape_type().unwrap(), ShapeEnum::Face);
}

#[test]
fn a_prism_shares_its_base_unless_copied() {
    let base = square();
    let mut shared = BRepSweepPrism::new(&base, Vector3::new(0.0, 0.0, 1.0), false, true).unwrap();
    assert!(shared.first_shape().unwrap().is_same(&base));
    let mut copied = BRepSweepPrism::new(&base, Vector3::new(0.0, 0.0, 1.0), true, true).unwrap();
    assert!(!copied.first_shape().unwrap().is_same(&base));
}

#[test]
fn a_vector_of_the_confusion_length_is_refused() {
    let base = square();
    let error = BRepSweepPrism::new(&base, Vector3::new(1e-7, 0.0, 0.0), false, true)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn an_infinite_prism_has_no_end_shapes() {
    let edge = segment(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let direction = Direction3::new(0.0, 1.0, 0.0).unwrap();
    let mut prism = BRepSweepPrism::infinite(&edge, direction, true, false, true).unwrap();
    assert_eq!(
        prism.shape().unwrap().shape_type().unwrap(),
        ShapeEnum::Face
    );
    assert!(prism.first_shape().unwrap().is_null());
    assert!(prism.last_shape().unwrap().is_null());
}

#[test]
fn the_history_of_a_foreign_shape_is_empty_or_out_of_range() {
    let mut prism =
        BRepSweepPrism::new(&square(), Vector3::new(0.0, 0.0, 1.0), false, true).unwrap();
    let foreign: Shape = BRepBuilderAPIMakeVertex::new(Point3::new(5.0, 5.0, 5.0))
        .unwrap()
        .vertex()
        .unwrap()
        .into();
    assert!(!prism.is_used(&foreign).unwrap());
    assert!(prism.shape_of(&foreign).unwrap().is_null());
    let error = prism.first_shape_of(&foreign).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn a_parallel_segment_revolves_into_a_cylinder() {
    let edge = segment(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 2.0));
    let mut revol = BRepSweepRevol::full(&edge, z_axis(), false).unwrap();
    let face = topods::face(&revol.shape().unwrap()).unwrap();
    let surface = BRepTool::surface(&face).unwrap();
    assert_eq!(
        surface.dynamic_type().unwrap().name(),
        "Geom_CylindricalSurface"
    );
}

#[test]
fn a_revolution_reduces_its_angle_and_reverses_the_axis_for_a_negative_one() {
    let edge = segment(Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 1.0));
    let revol = BRepSweepRevol::new(&edge, z_axis(), -3.0 * PI, false).unwrap();
    assert!((revol.angle() - PI).abs() < 1e-12);
    assert_eq!(revol.axe().direction().z(), -1.0);
}

#[test]
fn a_tiny_angle_is_refused() {
    let edge = segment(Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 1.0));
    let error = BRepSweepRevol::new(&edge, z_axis(), 1e-13, false)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn a_vertex_on_the_axis_is_invariant() {
    let vertex: Shape = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 1.0))
        .unwrap()
        .vertex()
        .unwrap()
        .into();
    let mut revol = BRepSweepRevol::new(&vertex, z_axis(), PI, false).unwrap();
    let edge = topods::edge(&revol.shape().unwrap()).unwrap();
    assert!(BRepTool::degenerated(&edge).unwrap());
}
