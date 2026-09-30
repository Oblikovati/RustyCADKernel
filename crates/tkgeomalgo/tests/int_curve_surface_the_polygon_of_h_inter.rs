//! Direct translation of IntCurveSurface_ThePolygonOfHInter_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::int_curve_surface::{HCurve, ThePolygonOfHInter};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};

#[test]
fn IntCurveSurface_ThePolygonOfHInter_Test__ClosedFlagReflectsStoredState() {
    let a_line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )))
    .cast();
    let an_adaptor: HCurve = Handle::new(GeomAdaptorCurve::with_curve(&a_line).unwrap()).cast();
    let mut a_polygon = ThePolygonOfHInter::new(&an_adaptor, 6).unwrap();

    assert!(!a_polygon.closed());
    a_polygon.set_closed(true);
    assert!(a_polygon.closed());
}
