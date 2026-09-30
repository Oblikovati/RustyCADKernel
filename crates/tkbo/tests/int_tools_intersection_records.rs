//! Executable intersection records, including the native copy and out-of-range quirks.
use rustycad_tkbo::int_tools::*;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine, GeomTrimmedCurve},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::{
    geom_abs::CurveType,
    gp::{Axis1, Axis2d, Point3},
};
#[test]
fn face_point_initialization_preserves_explicit_validity() {
    let mut point = PntOnFace::default();
    assert!(!point.valid());
    assert!(point.face().is_null());
    assert_eq!(point.parameters(), (99.0, 99.0));
    assert_eq!(point.pnt(), &Point3::default());
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    point.init(&face, Point3::new(1.0, 2.0, 3.0), 4.0, 5.0);
    assert!(!point.valid());
    point.set_valid(true);
    point.set_face(&Face::default());
    point.set_pnt(Point3::default());
    point.set_parameters(6.0, 7.0);
    assert!(point.valid());
    assert!(point.face().is_null());
    assert_eq!(point.parameters(), (6.0, 7.0));
    assert_eq!(point.pnt(), &Point3::default());
}
#[test]
fn paired_point_validity_is_independent_of_children() {
    let mut first = PntOnFace::default();
    first.set_valid(true);
    first.set_parameters(1.0, 2.0);
    let second = PntOnFace::default();
    let mut pair = PntOn2Faces::new(&first, &second);
    assert!(!pair.is_valid());
    assert!(pair.p1().valid());
    assert!(!pair.p2().valid());
    pair.set_p1(&second);
    pair.set_p2(&first);
    pair.set_valid(true);
    assert!(pair.is_valid());
    assert!(!pair.p1().valid());
    assert!(pair.p2().valid());
    assert!(!PntOn2Faces::default().is_valid());
}
fn common_part_fixture() -> CommonPrt {
    let mut part = CommonPrt::default();
    let mut edge = Edge::default();
    BRepBuilder::default().make_edge(&mut edge).unwrap();
    part.set_edge1(&edge);
    part.set_edge2(&edge);
    part.set_type(ShapeEnum::Edge);
    part.set_range1(Range::new(1.0, 2.0));
    part.set_vertex_parameter1(3.0);
    part.set_vertex_parameter2(4.0);
    part.append_range2(Range::new(5.0, 6.0));
    part.set_all_null_flag(true);
    part.set_bounding_points(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
    part
}
#[test]
fn common_part_copy_constructor_resets_flag_and_assignment_appends() {
    let part = common_part_fixture();
    let mut other = part.clone();
    assert!(!other.all_null_flag());
    assert_eq!(other.bounding_points(), part.bounding_points());
    assert_eq!(other.ranges2(), part.ranges2());
    other.assign(&part);
    assert!(other.all_null_flag());
    assert_eq!(other.ranges2().len(), 2);
    assert!(other.edge1().is_same(part.edge1()));
    assert!(other.edge2().is_same(part.edge2()));
    assert_eq!(other.shape_type(), ShapeEnum::Edge);
    assert_eq!(other.range1(), part.range1());
    assert_eq!(other.vertex_parameter1(), 3.0);
    assert_eq!(other.vertex_parameter2(), 4.0);
}
#[test]
fn common_part_copy_preserves_target_flags_and_points() {
    let part = common_part_fixture();
    let mut other = CommonPrt::default();
    part.copy_to(&mut other);
    assert!(!other.all_null_flag());
    assert_eq!(
        other.bounding_points(),
        (Point3::default(), Point3::default())
    );
    other.set_range1_bounds(-1.0, 9.0);
    assert_eq!(other.range1_bounds(), (-1.0, 9.0));
    other.append_range2_bounds(7.0, 8.0);
    assert_eq!(other.ranges2().len(), 2);
    other.change_ranges2().clear();
    assert!(other.ranges2().is_empty());
}
fn line_fixture() -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::default())).into_cast()
}
#[test]
fn curve_handles_and_tolerances_are_independent() {
    let line = line_fixture();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let mut curve = Curve::new(&line, &pcurve, &pcurve, 0.1, 0.2);
    assert_eq!(curve.tolerance(), 0.1);
    assert_eq!(curve.tangential_tolerance(), 0.2);
    assert!(curve.curve() == &line);
    curve.set_curves(&Handle::default(), &Handle::default(), &Handle::default());
    assert!(curve.curve().is_null());
    curve.set_curve(&line);
    curve.set_first_curve2d(&pcurve);
    curve.set_second_curve2d(&pcurve);
    assert!(curve.first_curve2d() == &pcurve);
    assert!(curve.second_curve2d() == &pcurve);
}
#[test]
fn curve_tolerance_setters_are_independent() {
    let mut curve = Curve::default();
    curve.set_tolerance(0.3);
    curve.set_tangential_tolerance(0.4);
    assert_eq!(
        (curve.tolerance(), curve.tangential_tolerance()),
        (0.3, 0.4)
    );
}
#[test]
fn curve_bounds_preserve_outputs_for_unbounded_curves() {
    let curve = Curve::new(
        &line_fixture(),
        &Handle::default(),
        &Handle::default(),
        0.0,
        0.0,
    );
    let (mut first, mut last) = (11.0, 12.0);
    let (mut p, mut q) = (Point3::default(), Point3::default());
    assert!(!curve.has_bounds());
    assert!(!Curve::default().has_bounds());
    assert!(!curve.bounds(&mut first, &mut last, &mut p, &mut q).unwrap());
    assert_eq!((first, last), (11.0, 12.0));
    assert_eq!(curve.curve_type().unwrap(), CurveType::Line);
}
#[test]
fn bounded_curve_d0_preserves_native_or_predicate() {
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&line_fixture(), 2.0, 4.0, true, true).unwrap())
            .into_cast();
    let curve = Curve::new(&trimmed, &Handle::default(), &Handle::default(), 0.0, 0.0);
    let (mut first, mut last) = (0.0, 0.0);
    let (mut p, mut q) = (Point3::default(), Point3::default());
    assert!(curve.has_bounds());
    assert!(curve.bounds(&mut first, &mut last, &mut p, &mut q).unwrap());
    assert_eq!((first, last), (2.0, 4.0));
    assert!(curve.d0(1.0, &mut p).unwrap());
    assert_eq!(p.z(), 1.0);
    assert!(curve.d0(5.0, &mut p).unwrap());
    assert_eq!(p.z(), 5.0);
    assert!(!curve.d0(f64::NAN, &mut p).unwrap());
    assert_eq!(p.z(), 5.0);
}
