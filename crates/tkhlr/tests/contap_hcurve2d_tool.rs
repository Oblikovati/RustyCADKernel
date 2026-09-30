//! Contap arc sampling and forwarding contracts.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::{
    geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::int_start::TheArc;
use rustycad_tkhlr::contap::ContapHCurve2dTool as Tool;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
fn spline_arc() -> TheArc {
    let poles = [
        Point2::default(),
        Point2::new(1.0, 2.0),
        Point2::new(3.0, 0.0),
    ];
    let knots = [0.0, 1.0];
    let mults = [3, 3];
    let spline: Handle<dyn Geom2dCurve> = Handle::new(
        Geom2dBSplineCurve::new(
            &Array1::from_shared(&poles, 1),
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&mults, 1),
            2,
            false,
        )
        .unwrap(),
    )
    .cast();
    Handle::new(Geom2dAdaptorCurve::with_curve(&spline).unwrap()).cast()
}
#[test]
fn spline_sample_count_uses_original_ratio_and_caps() {
    let arc = spline_arc();
    assert_eq!(Tool::nb_samples(&arc, 0.0, 1.0).unwrap(), 4);
    assert_eq!(Tool::nb_samples(&arc, 0.0, 0.5).unwrap(), 8);
    assert_eq!(Tool::nb_samples(&arc, 0.0, 0.0).unwrap(), 50);
    assert_eq!(Tool::nb_samples(&arc, 1.0, 0.0).unwrap(), 2);
}
#[test]
fn line_evaluation_derivatives_and_sampling_forward_exactly() {
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::X,
    )))
    .cast();
    let arc = Handle::new(Geom2dAdaptorCurve::with_range(&line, 2.0, 5.0).unwrap()).cast();
    assert_eq!(Tool::first_parameter(&arc).unwrap(), 2.0);
    assert_eq!(Tool::last_parameter(&arc).unwrap(), 5.0);
    assert_eq!(Tool::value(&arc, 3.0).unwrap(), Point2::new(3.0, 0.0));
    assert_eq!(
        Tool::d0(&arc, 3.0).unwrap(),
        Tool::value(&arc, 3.0).unwrap()
    );
    assert_eq!(Tool::nb_samples(&arc, 0.0, 0.0).unwrap(), 2);
    assert!(Tool::circle(&arc).is_err());
}
#[test]
fn null_arc_has_explicit_error_instead_of_native_dereference() {
    assert!(Tool::nb_samples(&Handle::default(), 0.0, 1.0).is_err());
    assert!(Tool::value(&Handle::default(), 0.0).is_err());
}
