//! Native free-boundary repair cases, including the frozen-edge failure.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::{self, Explorer},
    topods::{Compound, Face, Shape, Wire},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_fix::ShapeFixFreeBounds;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;
const ORACLE: &str = include_str!("../../../tools/oracle/tkshhealing_fix_free_bounds.expected");
fn open_face(start: f64, end: f64) -> Face {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let mut wire = Wire::default();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &plane, 1e-7)
        .unwrap();
    builder.make_wire(&mut wire);
    let edge = BRepLibMakeEdge::from_points(Point3::new(start, 0., 0.), Point3::new(end, 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    builder.add(&mut wire, &edge).unwrap();
    builder.add(&mut face, &wire).unwrap();
    face
}
fn empty_compound() -> Compound {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    compound
}
fn faces() -> Compound {
    let builder = BRepBuilder::default();
    let mut compound = empty_compound();
    builder.add(&mut compound, &open_face(0., 1.)).unwrap();
    builder.add(&mut compound, &open_face(1.001, 2.)).unwrap();
    compound
}
fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    Explorer::new(shape, kind, ShapeEnum::Shape)
        .unwrap()
        .count()
}
fn assert_row(label: &str, fixer: &ShapeFixFreeBounds) {
    let mut vertices = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(fixer.shape(), ShapeEnum::Vertex, &mut vertices).unwrap();
    let observed = [
        usize::from(fixer.shape().is_null()),
        usize::from(fixer.closed_wires().is_null()),
        usize::from(fixer.open_wires().is_null()),
        count(fixer.closed_wires(), ShapeEnum::Wire),
        count(fixer.open_wires(), ShapeEnum::Wire),
        count(fixer.open_wires(), ShapeEnum::Edge),
        vertices.extent(),
    ];
    let expected: Vec<usize> = ORACLE
        .lines()
        .find(|line| line.split_whitespace().next() == Some(label))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap())
        .collect();
    assert_eq!(observed.as_slice(), expected, "{label}");
}
#[test]
fn defaults_and_empty_connection_preserve_native_null_outputs() {
    assert_row("default", &ShapeFixFreeBounds::new());
    let empty = empty_compound();
    assert_row(
        "empty-zero",
        &ShapeFixFreeBounds::from_shape(&empty, 0., false, false).unwrap(),
    );
    assert_row(
        "empty-positive",
        &ShapeFixFreeBounds::from_shape(&empty, 0.01, false, false).unwrap(),
    );
}
#[test]
fn shared_and_forecast_boundaries_match_native() {
    let source = faces();
    let actual = ShapeFixFreeBounds::from_shape(&source, 0.01, false, false).unwrap();
    assert_row("actual", &actual);
    assert!(actual.shape().is_same(&source));
    let forecast = ShapeFixFreeBounds::forecast(&faces(), 1e-7, 0., false, false).unwrap();
    assert_row("forecast-no-connect", &forecast);
}
#[test]
fn forecasting_connection_keeps_native_frozen_shape_failure() {
    let failure = ShapeFixFreeBounds::forecast(&faces(), 1e-7, 0.01, false, false)
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "TopoDS_FrozenShape");
    assert!(ORACLE.contains("forecast-connect TopoDS_FrozenShape"));
}
