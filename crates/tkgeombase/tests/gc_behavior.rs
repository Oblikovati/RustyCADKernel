//! Behaviour of the GC and GCE2d ports beyond the upstream suites: the StdFail_NotDone
//! results, the conversion operators, the deprecated GCE2d aliases and the native quirks
//! that tools/verify_tkgeombase_numeric.py compares with OCCT.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTransformation, GeomTrimmedCurve};
use rustycad_tkgeombase::gc::{
    ErrorType, MakeArcOfCircle, MakeArcOfCircle2d, MakeCircle, MakeCylindricalSurface, MakeEllipse,
    MakeLine2d, MakeScale, MakeSegment, MakeTranslation, Root,
};
use rustycad_tkgeombase::gce2d;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Cylinder3, Point2, Point3, TransformForm, Vector2, Vector3,
};

#[test]
fn value_of_a_failed_construction_raises_not_done() {
    let maker = MakeCircle::new(Axis2::default(), -1.0).unwrap();
    assert_eq!(maker.status(), ErrorType::NegativeRadius);
    let failure = maker.value().err().unwrap();
    assert_eq!(failure.exception_type(), "StdFail_NotDone");
    assert!(failure.message().contains("GC_MakeCircle::Value()"));
    assert!(Handle::<GeomCircle>::try_from(&maker).is_err());
}

#[test]
fn conversion_operators_share_the_handle() {
    let maker = MakeCircle::new(Axis2::default(), 2.0).unwrap();
    let converted: Handle<GeomCircle> = Handle::try_from(&maker).unwrap();
    assert!(converted == maker.value().unwrap());
    let translation = MakeTranslation::new(Vector3::new(1.0, 0.0, 0.0));
    let handle: Handle<GeomTransformation> = Handle::from(&translation);
    assert_eq!(handle.get().unwrap().form(), TransformForm::Translation);
}

#[test]
fn ellipse_with_inverted_radii_reports_invert_axis() {
    let maker = MakeEllipse::new(Axis2::default(), 1.0, 2.0).unwrap();
    assert_eq!(maker.status(), ErrorType::InvertAxis);
}

#[test]
fn segment_between_confused_points_raises_through_the_line_conversion() {
    let p = Point3::new(1.0, 2.0, 3.0);
    let failure = MakeSegment::from_points(p, p).err().unwrap();
    assert_eq!(failure.exception_type(), "StdFail_NotDone");
}

#[test]
fn tangent_arc_2d_raises_for_confused_points_and_null_tangents() {
    let p = Point2::new(1.0, 0.0);
    let confused = MakeArcOfCircle2d::from_tangent(p, Vector2::new(0.0, 1.0), p);
    assert_eq!(confused.err().unwrap().exception_type(), "StdFail_NotDone");
    let null = MakeArcOfCircle2d::from_tangent(p, Vector2::new(0.0, 0.0), Point2::new(0.0, 1.0));
    assert_eq!(
        null.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn tangent_along_the_chord_leaves_confused_points_in_2d() {
    let (p1, p2) = (Point2::new(1.0, 0.0), Point2::new(0.0, 1.0));
    let maker = MakeArcOfCircle2d::from_tangent(p1, Vector2::new(-1.0, 1.0), p2).unwrap();
    assert_eq!(maker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn nearly_parallel_tangent_centers_the_3d_arc_at_the_origin() {
    let (p1, p2) = (Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
    let maker =
        MakeArcOfCircle::from_tangent(p1, Vector3::new(-1.0, 1.0000000000001, 0.0), p2).unwrap();
    let arc: Handle<GeomTrimmedCurve> = maker.value().unwrap();
    let basis: Handle<GeomCircle> = arc.get().unwrap().basis_curve().cast();
    let center: Point3 = basis.get().unwrap().circ().unwrap().location();
    assert_eq!((center.x(), center.y(), center.z()), (0.0, 0.0, 0.0));
    assert!(arc.get().unwrap().first_parameter() < arc.get().unwrap().last_parameter());
}

#[test]
fn cylinder_at_a_distance_subtracts_it() {
    let cylinder = Cylinder3::new(Axis3::default(), 3.0).unwrap();
    let maker = MakeCylindricalSurface::from_cylinder_distance(cylinder, 5.0).unwrap();
    assert_eq!(maker.value().unwrap().get().unwrap().radius(), 2.0);
}

#[test]
fn null_scale_factor_raises() {
    let failure = MakeScale::new(Point3::default(), 0.0).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

#[test]
#[allow(deprecated)]
fn gce2d_aliases_name_the_gc_classes() {
    let maker: gce2d::MakeLine =
        MakeLine2d::from_points(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)).unwrap();
    let line: Handle<Geom2dLine> = maker.value().unwrap();
    assert!(!line.is_null());
    fn is_root<R: gce2d::Root>(_root: &R) {}
    is_root(&maker);
}

#[test]
fn arc_through_coincident_points_raises_from_the_equal_trim_bounds() {
    let p = Point3::new(1.0, 1.0, 1.0);
    let failure = MakeArcOfCircle::through_points(p, p, p).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}
