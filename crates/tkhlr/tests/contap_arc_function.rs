//! Restriction contour outputs and state numbering.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::{
    geom2d::{Geom2dCurve, Geom2dLine},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkgeomalgo::int_start::TheArcFunction;
use rustycad_tkhlr::contap::ContapArcFunction;
use rustycad_tkmath::{
    gp::{Axis2d, Axis3, Direction2, Point2, Point3},
    math::{Function, FunctionWithDerivative},
};
fn plane_function() -> ContapArcFunction {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let adaptor =
        Handle::new(GeomAdaptorSurface::with_bounds(&surface, (0., 1., 0., 1.)).unwrap()).cast();
    let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::X,
    )))
    .cast();
    let arc = Handle::new(Geom2dAdaptorCurve::with_range(&curve, 0., 1.).unwrap()).cast();
    let mut function = ContapArcFunction::new();
    function.set_surface(&adaptor).unwrap();
    function.set_arc(&arc);
    function
}
#[test]
fn arc_draft_keeps_value_but_zeroes_derivative() {
    let mut function = plane_function();
    assert_eq!(function.values_at(0.5, 17.).unwrap(), (1., 0.));
    function.set_eye_angle(Point3::new(1., 2., 3.), 0.3);
    let (mut value, mut derivative) = (17., 19.);
    assert!(
        function
            .values_into(0.5, &mut value, &mut derivative)
            .unwrap()
    );
    assert_eq!((value, derivative), (17., 0.));
    assert_eq!(function.nb_samples().unwrap(), 2);
    assert_eq!(function.get_state_number(), 1);
    assert_eq!(function.valpoint(1).unwrap(), Point3::new(0.5, 0., 0.));
    assert!(function.valpoint(i32::MIN).is_err());
    assert!(function.valpoint(2).is_err());
}
#[test]
fn setting_arc_clears_saved_points_but_surface_does_not() {
    let mut function = plane_function();
    function.value_at(0.2, 0.).unwrap();
    assert_eq!(function.get_state_number(), 1);
    let surface = function.surface();
    function.set_surface(&surface).unwrap();
    assert_eq!(function.get_state_number(), 2);
    function.set_arc(&Handle::default());
    assert!(function.valpoint(1).is_err());
    assert_eq!(function.get_state_number(), 1);
}
#[test]
fn failed_callbacks_preserve_output_and_capture_error() {
    let mut function = ContapArcFunction::new();
    let (mut value, mut derivative) = (17., 19.);
    assert!(!Function::value(&mut function, 0.5, &mut value));
    assert_eq!(value, 17.);
    assert!(function.take_failure().is_err());
    assert!(!FunctionWithDerivative::values(
        &mut function,
        0.5,
        &mut value,
        &mut derivative
    ));
    assert_eq!((value, derivative), (17., 19.));
    assert!(function.take_failure().is_err());
    assert!(function.take_failure().is_ok());
}
