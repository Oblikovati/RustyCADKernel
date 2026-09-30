//! Conic, spline and offset families exercise each native resolution policy.
use super::super::tools_geometry;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomEllipse, GeomHyperbola, GeomOffsetCurve,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
pub(super) fn geometry(kind: i32, offset: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    if kind <= 6 {
        return tools_geometry::geometry(kind, 0., 1.);
    }
    match kind {
        7 => Ok(Handle::new(GeomEllipse::new(Axis2::default(), 3., 1.)?).cast()),
        8 => Ok(Handle::new(GeomHyperbola::new(Axis2::default(), 2., 1.)?).cast()),
        9 => spline(),
        _ => offset_geometry(kind, offset),
    }
}
fn spline() -> Result<Handle<dyn GeomCurve>, Failure> {
    let poles = [
        Point3::new(0., 0., 0.),
        Point3::new(1., 2., 0.),
        Point3::new(3., 0., 0.),
    ];
    Ok(Handle::new(GeomBSplineCurve::new(
        &Array1::from_shared(&poles, 1),
        &Array1::from_shared(&[0., 1.], 1),
        &Array1::from_shared(&[3, 3], 1),
        2,
        false,
    )?)
    .cast())
}
fn offset_geometry(kind: i32, offset: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
    let basis = geometry(
        match kind {
            10 => 6,
            11 => 1,
            12 => 7,
            13 => 3,
            _ => 9,
        },
        offset,
    )?;
    Ok(Handle::new(GeomOffsetCurve::new(&basis, offset, Direction3::Z, false)?).cast())
}
