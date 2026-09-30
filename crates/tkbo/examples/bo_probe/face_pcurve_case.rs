use super::{face_pcurve_checks, fclass_faces, flag};
use rustycad_tkbo::int_tools::Context;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
fn pcurve(kind: i32, x: f64, y: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let axis = Axis2d::new(Point2::new(x, y), Direction2::X);
    match kind {
        0 => Ok(Handle::default()),
        1 => {
            let line = Handle::new(Geom2dLine::new(axis)).cast();
            Ok(Handle::new(Geom2dTrimmedCurve::new(&line, 0., 1., true, true)?).cast())
        }
        2 => Ok(Handle::new(Geom2dCircle::from_axis(axis, 1., true)?).cast()),
        _ => spline(kind, x, y),
    }
}
fn spline(kind: i32, x: f64, y: f64) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let poles = spline_poles(kind, x, y);
    let knots = [0., 1., 2., 3., 4.];
    let mults = [2, 1, 1, 1, 2];
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&knots, 1),
        &Array1::from_shared(&mults, 1),
        1,
        false,
    )?)
    .cast())
}
fn run(v: &[f64]) -> Result<(), Failure> {
    let curve = pcurve(v[0] as i32, v[2], v[3])?;
    flag(face_pcurve_checks::curve_valid(&curve)?);
    if curve.is_null() {
        return Ok(());
    }
    let face = fclass_faces::face(v[1] as i32, 3.)?;
    flag(face_pcurve_checks::check_pcurve(
        &curve,
        &face,
        &Context::default(),
    )?);
    Ok(())
}
pub fn case(v: &[f64]) {
    if let Err(error) = run(v) {
        print!("{} ", error.exception_type());
    }
}

fn spline_poles(kind: i32, x: f64, y: f64) -> [Point2; 5] {
    let last = if kind == 3 {
        Point2::new(x + 3., y + 1.)
    } else {
        Point2::new(x, y)
    };
    [
        Point2::new(x, y),
        Point2::new(x + 2., y + 2.),
        Point2::new(x, y + 2.),
        Point2::new(x + 2., y),
        last,
    ]
}
