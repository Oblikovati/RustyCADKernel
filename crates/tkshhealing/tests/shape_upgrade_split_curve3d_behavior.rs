//! Focused native-contract regressions; differential corpus covers all dynamic kinds.
use rustycad_tkernel::{
    ncollection::{Array1, HSequence},
    standard::Handle,
};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis1, Point3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeSplitCurve3d, SplitCurve},
};
#[test]
fn multipart_without_segment_preserves_native_null_entries() {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let split = ShapeUpgradeSplitCurve3d::new();
    split.init_curve_range(&curve, 0., 1.).unwrap();
    let values = Handle::new(HSequence::default());
    values.append(0.5);
    split.set_split_values(&values).unwrap();
    split.build(false).unwrap();
    assert!(split.status_flag(ShapeExtendStatus::Done1));
    let result = split.curves();
    assert_eq!(result.array1().len(), 2);
    assert!(result.array1().iter().all(Handle::is_null));
}
#[test]
fn single_trimmed_piece_uses_private_geometry_and_retains_old_result() {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let split = ShapeUpgradeSplitCurve3d::new();
    assert!(split.curves().is_null());
    split.init_curve_range(&curve, 2., 4.).unwrap();
    split.build(true).unwrap();
    let previous = split.curves();
    let piece = previous.array1().value(1).unwrap().clone();
    assert!(!piece.cast::<GeomTrimmedCurve>().is_null());
    assert_eq!(piece.first_parameter(), 2.);
    assert_ne!(piece.object_address(), curve.object_address());
    split.build(true).unwrap();
    assert_ne!(split.curves().object_address(), previous.object_address());
    assert_eq!(piece.last_parameter(), 4.);
}
#[test]
fn done2_single_bezier_segments_and_sets_done3() {
    let points = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 2., 0.),
        Point3::new(2., 0., 0.),
    ];
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let split = ShapeUpgradeSplitCurve3d::new();
    split.init_curve_range(&curve, 0.2, 0.8).unwrap();
    split.split_curve_data().with_mut(|f| f.status = 2);
    split.build(true).unwrap();
    assert!(split.status_flag(ShapeExtendStatus::Done2));
    assert!(split.status_flag(ShapeExtendStatus::Done3));
    let piece = split.curves().array1().value(1).unwrap().clone();
    assert!(!piece.cast::<GeomBezierCurve>().is_null());
    assert_eq!(curve.first_parameter(), 0.);
    assert_eq!(curve.last_parameter(), 1.);
}
#[test]
fn nonperiodic_zero_range_expands_after_native_clamping() {
    let points = [Point3::new(0., 0., 0.), Point3::new(1., 0., 0.)];
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    let split = ShapeUpgradeSplitCurve3d::new();
    split.init_curve_range(&curve, 0.5, 0.5).unwrap();
    let values = split.split_values();
    assert_eq!(values.sequence(|s| *s.value(1).unwrap()), 0.5);
    assert_eq!(values.sequence(|s| *s.value(2).unwrap()), 0.5 + 2e-9);
}
#[test]
fn native_offset_null_basis_crash_is_an_explicit_typed_failure_divergence() {
    use rustycad_tkg3d::geom::GeomOffsetCurve;
    use rustycad_tkmath::gp::dz;
    let basis: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomOffsetCurve::new(&basis, 0.2, dz(), false).unwrap()).cast();
    let split = ShapeUpgradeSplitCurve3d::new();
    split.init_curve_range(&curve, 0., 1.).unwrap();
    let values = Handle::new(HSequence::default());
    values.append(0.5);
    split.set_split_values(&values).unwrap();
    let error = split.build(false).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(split.curves().array1().len(), 2);
    assert!(split.curves().array1().iter().all(Handle::is_null));
}
