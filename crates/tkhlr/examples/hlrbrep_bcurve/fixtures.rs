//! Production BRep curves, including rational splines and located edges.
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorCurve, topods::Edge};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::*;
use rustycad_tkmath::{
    gp::{Axis1, Axis2, Direction3, Point3, Transform3},
    toploc::Location,
};
pub fn curve(kind: i32, shift: f64) -> Result<BRepAdaptorCurve, Failure> {
    let geometry = geometry(kind)?;
    let mut edge = Edge::default();
    let builder = BRepBuilder::default();
    builder.make_edge_with_curve(&mut edge, &geometry, 1e-7)?;
    builder.range(&edge, 0., if kind == 10 { 0. } else { 1. }, false)?;
    let transform = Transform3::translation([shift, 2. * shift, -shift]);
    edge.set_location(&Location::new(transform), true)?;
    BRepAdaptorCurve::with_edge(&edge)
}
fn geometry(kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match kind {
        0 => Handle::new(GeomLine::new(Axis1::default())).cast(),
        1 => Handle::new(GeomCircle::new(Axis2::default(), 2.)?).cast(),
        2 => Handle::new(GeomEllipse::new(Axis2::default(), 3., 2.)?).cast(),
        3 => Handle::new(GeomHyperbola::new(Axis2::default(), 3., 2.)?).cast(),
        4 => Handle::new(GeomParabola::new(Axis2::default(), 1.)?).cast(),
        5..=8 => return spline(kind),
        10 => return spline(7),
        _ => Handle::new(GeomOffsetCurve::new(
            &Handle::new(GeomLine::new(Axis1::default())).cast(),
            0.5,
            Direction3::Y,
            false,
        )?)
        .cast(),
    })
}
fn spline(kind: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut poles = Array1::with_bounds(1, 4).unwrap();
    let mut weights = Array1::with_bounds(1, 4).unwrap();
    for i in 1..=4 {
        poles[i] = Point3::new(f64::from(i - 1), f64::from(i % 2), f64::from(i * i));
        weights[i] = if i == 2 { 2. } else { 1. };
    }
    if kind == 5 {
        return Ok(Handle::new(GeomBezierCurve::new(&poles)?).cast());
    }
    if kind == 6 {
        return Ok(Handle::new(GeomBezierCurve::with_weights(&poles, &weights)?).cast());
    }
    bspline(&poles, &weights, kind == 8)
}
fn bspline(
    poles: &Array1<'_, Point3>,
    weights: &Array1<'_, f64>,
    rational: bool,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut knots = Array1::with_bounds(1, 3).unwrap();
    let mut mults = Array1::with_bounds(1, 3).unwrap();
    for i in 1..=3 {
        knots[i] = f64::from(i - 1);
        mults[i] = if i == 2 { 1 } else { 3 };
    }
    if !rational {
        return Ok(Handle::new(GeomBSplineCurve::new(poles, &knots, &mults, 2, false)?).cast());
    }
    Ok(Handle::new(GeomBSplineCurve::with_weights(
        poles, weights, &knots, &mults, 2, false,
    )?)
    .cast())
}
