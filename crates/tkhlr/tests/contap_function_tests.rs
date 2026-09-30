//! Contour retained output and failure capture regressions.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle,
    geom::{GeomPlane, GeomSurface},
    geom_adaptor::GeomAdaptorSurface,
};
use rustycad_tkhlr::contap::{ContapSurfFunction, ContapTFunction};
use rustycad_tkmath::{
    gp::{Axis3, Direction3, Point3},
    math::{FunctionSet, Matrix, Vector},
};
fn plane() -> Adaptor3dSurfaceHandle {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&surface, (0., 1., 0., 1.)).unwrap()).cast()
}
#[test]
fn perspective_draft_retains_outputs_and_cached_tangent() {
    let mut function = ContapSurfFunction::new();
    function.set_surface(&plane()).unwrap();
    let point = Vector::filled(1, 2, 0.5).unwrap();
    let mut value = Vector::filled(1, 1, 17.).unwrap();
    assert!(function.value(&point, &mut value).unwrap());
    assert_eq!(value[1], 1.);
    assert!(function.is_tangent().unwrap());
    function.set_eye_angle(Point3::new(1., 2., 3.), 0.3);
    assert_eq!(function.function_type(), ContapTFunction::DraftPrs);
    value[1] = 19.;
    assert!(function.value(&point, &mut value).unwrap());
    assert_eq!(value[1], 19.);
    assert_eq!(function.root(), 1.);
    let mut gradient = Matrix::filled(1, 1, 1, 2, 23.).unwrap();
    function.values(&point, &mut value, &mut gradient).unwrap();
    assert_eq!(function.root(), 19.);
    assert_eq!(gradient[(1, 1)], 23.);
    assert!(!function.is_tangent().unwrap());
    assert!(function.direction3d().is_ok());
    assert!(function.direction2d().is_ok());
}
#[test]
fn tolerance_keeps_native_nan_and_minimum_rules() {
    let mut function = ContapSurfFunction::new();
    function.set_tolerance(-1.);
    assert_eq!(function.tolerance(), 1e-12);
    function.set_tolerance(f64::NAN);
    assert!(function.tolerance().is_nan());
    function.set_direction(Direction3::X);
    assert_eq!(function.direction(), Direction3::X);
    assert_eq!(function.nb_variables(), 2);
    assert_eq!(function.nb_equations(), 1);
}
#[test]
fn solver_callback_retains_first_surface_failure() {
    let mut function = ContapSurfFunction::new();
    let point = Vector::filled(1, 2, 0.).unwrap();
    let mut value = Vector::filled(1, 1, 0.).unwrap();
    assert!(!FunctionSet::value(&mut function, &point, &mut value));
    assert!(function.take_failure().is_err());
    assert!(function.take_failure().is_ok());
}
