//! 2D-specific native semantics and shared-kernel RTTI/state regressions.
use rustycad_tkernel::{
    ncollection::{Array1, HSequence},
    standard::Handle,
};
use rustycad_tkg2d::geom2d::{
    Geom2dBezierCurve, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Point2};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_upgrade::{ShapeUpgradeSplitCurve, ShapeUpgradeSplitCurve2d, SplitCurve},
};
fn quadratic() -> Handle<dyn Geom2dCurve> {
    let poles = [
        Point2::new(0., 0.),
        Point2::new(1., 2.),
        Point2::new(3., 0.),
    ];
    let poles = Array1::from_shared(&poles, 1);
    Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast()
}
fn insert_middle(split: &ShapeUpgradeSplitCurve2d) {
    let values = Handle::new(HSequence::default());
    values.append(0.5);
    split.set_split_values(&values).unwrap();
}
#[test]
fn single_bezier_segments_without_segment_or_done3_status() {
    let curve = quadratic();
    let split = ShapeUpgradeSplitCurve2d::new();
    split.init_curve_range(&curve, 0.2, 0.8).unwrap();
    split.build(false).unwrap();
    let piece = split.curves().array1().value(1).unwrap().clone();
    assert!(!piece.cast::<Geom2dBezierCurve>().is_null());
    // Segment re-associates the polynomial arithmetic; the oracle checks exact native bits.
    assert!(piece.value(0.).unwrap().distance(curve.value(0.2).unwrap()) < 1e-14);
    assert!(piece.value(1.).unwrap().distance(curve.value(0.8).unwrap()) < 1e-14);
    assert!(split.status_flag(ShapeExtendStatus::Ok));
}
#[test]
fn done2_single_bezier_sets_done3_through_general_segmentation() {
    let split = ShapeUpgradeSplitCurve2d::new();
    split.init_curve_range(&quadratic(), 0.2, 0.8).unwrap();
    split.split_curve_data().with_mut(|f| f.status = 2);
    split.build(true).unwrap();
    assert!(split.status_flag(ShapeExtendStatus::Done2));
    assert!(split.status_flag(ShapeExtendStatus::Done3));
    assert!(
        !split
            .curves()
            .array1()
            .value(1)
            .unwrap()
            .cast::<Geom2dBezierCurve>()
            .is_null()
    );
}
#[test]
fn multipart_false_retains_null_pieces_then_perform_segments() {
    let split = ShapeUpgradeSplitCurve2d::new();
    split.init_curve(&quadratic()).unwrap();
    insert_middle(&split);
    split.build(false).unwrap();
    let previous = split.curves();
    assert_eq!(previous.array1().len(), 2);
    assert!(previous.array1().iter().all(Handle::is_null));
    split.perform(true).unwrap();
    assert!(split.curves().array1().iter().all(|c| !c.is_null()));
    assert!(previous.array1().iter().all(Handle::is_null));
}
#[test]
fn native_offset_null_piece_crash_stays_an_explicit_divergence() {
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dOffsetCurve::new(&quadratic(), 0.2, false).unwrap()).cast();
    let split = ShapeUpgradeSplitCurve2d::new();
    split.init_curve(&curve).unwrap();
    insert_middle(&split);
    assert_eq!(
        split.build(false).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(split.curves().array1().len(), 2);
    assert!(split.curves().array1().iter().all(Handle::is_null));
}
#[test]
fn native_parent_projection_observes_the_same_parameter_state() {
    let split = Handle::new(ShapeUpgradeSplitCurve2d::new());
    split.init_curve(&quadratic()).unwrap();
    let base = split.cast::<ShapeUpgradeSplitCurve>();
    insert_middle(&split);
    assert_eq!(
        base.split_values().object_address(),
        split.split_values().object_address()
    );
    assert_eq!(base.split_values().sequence(|s| s.len()), 3);
    split.build(true).unwrap();
    assert!(base.status_flag(ShapeExtendStatus::Done1));
}
#[test]
fn trimmed_single_piece_preserves_wrapper_and_private_input_copy() {
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&line, 0., 1., true, true).unwrap()).cast();
    let split = ShapeUpgradeSplitCurve2d::new();
    split.init_curve_range(&curve, 0.2, 0.8).unwrap();
    split.build(true).unwrap();
    let piece = split.curves().array1().value(1).unwrap().clone();
    assert!(!piece.cast::<Geom2dTrimmedCurve>().is_null());
    assert_eq!(piece.first_parameter(), 0.2);
    assert_eq!(curve.first_parameter(), 0.);
    assert_ne!(piece.object_address(), curve.object_address());
}
