use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomCircle, GeomCurve, GeomLine, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
pub fn geometry(kind: i32, first: f64, last: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    let curve = basis(kind)?;
    if matches!(kind, 0 | 2 | 4 | 5 | 6) {
        return Ok(
            Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).into_cast(),
        );
    }
    Ok(curve)
}
fn closed_bezier() -> Result<Handle<dyn GeomCurve>, Failure> {
    let points = [
        Point3::default(),
        Point3::new(1., 2., 0.),
        Point3::new(-1., 2., 0.),
        Point3::default(),
    ];
    Ok(Handle::new(GeomBezierCurve::new(&Array1::from_shared(&points, 1))?).into_cast())
}

fn basis(kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match kind {
        0 => Handle::new(GeomLine::new(Axis1::default())).into_cast(),
        1 | 2 => Handle::new(GeomCircle::new(Axis2::default(), 2.)?).into_cast(),
        3 => closed_bezier()?,
        4 | 5 => Handle::new(GeomParabola::new(
            Axis2::default(),
            if kind == 4 { 0.5 } else { 0. },
        )?)
        .into_cast(),
        _ => Handle::new(GeomLine::new(Axis1::new(
            Point3::default(),
            Direction3::new(1., 0., 0.)?,
        )))
        .into_cast(),
    })
}
