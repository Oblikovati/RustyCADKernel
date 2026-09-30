use super::{flag, integer, real};
use rustycad_tkbo::int_tools::Curve;
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis2d, Point3};
pub(super) fn geometry(kind: i32) -> Handle<dyn GeomCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).into_cast();
    match kind {
        1 => Handle::new(GeomTrimmedCurve::new(&line, 2.0, 4.0, true, true).unwrap()).into_cast(),
        2 => Handle::new(GeomCircle::new(Axis2::default(), 2.0).unwrap()).into_cast(),
        3 => bezier(),
        _ => line,
    }
}
fn bezier() -> Handle<dyn GeomCurve> {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(4.0, 5.0, 6.0),
    ];
    let poles = Array1::from_shared(&points, 1);
    Handle::new(GeomBezierCurve::new(&poles).unwrap()).into_cast()
}
fn emit(curve: &Curve, parameter: f64) {
    flag(curve.has_bounds());
    real(curve.tolerance());
    real(curve.tangential_tolerance());
    integer(curve.curve_type().unwrap() as i32);
    let (mut first, mut last) = (31.0, 32.0);
    let (mut p, mut q) = (Point3::new(8.0, 9.0, 10.0), Point3::new(11.0, 12.0, 13.0));
    flag(curve.bounds(&mut first, &mut last, &mut p, &mut q).unwrap());
    real(first);
    real(last);
    point(p);
    point(q);
    flag(curve.d0(parameter, &mut p).unwrap());
    point(p);
    flag(curve.d0(f64::NAN, &mut p).unwrap());
    point(p);
}
fn point(p: Point3) {
    real(p.x());
    real(p.y());
    real(p.z());
}
pub fn case(v: &[f64]) {
    flag(Curve::default().has_bounds());
    let geom = geometry(v[0] as i32);
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
    let mut curve = Curve::new(&geom, &pcurve, &pcurve, v[2], v[3]);
    emit(&curve, v[1]);
    flag(curve.curve() == &geom);
    flag(curve.first_curve2d() == &pcurve);
    flag(curve.second_curve2d() == &pcurve);
    replace_curves(&mut curve, &geom, &pcurve);
    curve.set_tolerance(v[3]);
    curve.set_tangential_tolerance(v[2]);
    emit(&curve, v[1]);
}
fn replace_curves(
    curve: &mut Curve,
    geom: &Handle<dyn GeomCurve>,
    pcurve: &Handle<dyn Geom2dCurve>,
) {
    curve.set_curves(&Handle::default(), &Handle::default(), &Handle::default());
    flag(curve.curve().is_null());
    curve.set_curve(geom);
    curve.set_first_curve2d(pcurve);
    curve.set_second_curve2d(pcurve);
}
