//! Real faces: analytic, swept, polynomial and rational surfaces with locations.
use rustycad_tkbrep::{brep::BRepBuilder, brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::{
    ncollection::{Array1, Array2},
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::*;
use rustycad_tkmath::{
    gp::{Axis1, Axis3, Direction3, Point3, Transform3},
    toploc::Location,
};
pub fn surface(kind: i32, shift: f64) -> Result<BRepAdaptorSurface, Failure> {
    let geometry = geometry(kind)?;
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_surface(&mut face, &geometry, 1e-7)?;
    face.set_location(
        &Location::new(Transform3::translation([shift, 2. * shift, -shift])),
        true,
    )?;
    BRepAdaptorSurface::with_face(&face, false)
}
fn geometry(kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let a = Axis3::default();
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(a)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(a, 2.)?).cast(),
        2 => Handle::new(GeomConicalSurface::new(a, 0.4, 2.)?).cast(),
        3 => Handle::new(GeomSphericalSurface::new(a, 2.)?).cast(),
        4 => Handle::new(GeomToroidalSurface::new(a, 3., 1.)?).cast(),
        5..=8 => return spline(kind),
        11 => Handle::new(GeomOffsetSurface::new(&spline(5)?, 0.5, false)?).cast(),
        _ => return swept(kind),
    })
}
fn swept(kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomParabola::new(Default::default(), 1.)?).cast();
    if kind == 9 {
        return Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, Direction3::Z)?).cast());
    }
    Ok(Handle::new(GeomSurfaceOfRevolution::new(
        &curve,
        Axis1::new(Point3::new(0., -2., 0.), Direction3::X),
    )?)
    .cast())
}
fn spline(kind: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut poles = Array2::with_bounds(1, 9, 1, 9).unwrap();
    let mut weights = Array2::with_bounds(1, 9, 1, 9).unwrap();
    for i in 1..=9 {
        for j in 1..=9 {
            poles[(i, j)] = Point3::new(f64::from(i - 1), f64::from(j - 1), f64::from(i * j % 3));
            weights[(i, j)] = if i == 2 && j == 3 { 2. } else { 1. };
        }
    }
    if kind == 5 {
        return Ok(Handle::new(GeomBezierSurface::new(&poles)?).cast());
    }
    if kind == 6 {
        return Ok(Handle::new(GeomBezierSurface::with_weights(&poles, &weights)?).cast());
    }
    bspline(kind, &poles, &weights)
}
fn bspline(
    kind: i32,
    poles: &Array2<'_, Point3>,
    weights: &Array2<'_, f64>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut knots = Array1::with_bounds(1, 7).unwrap();
    let mut mults = Array1::with_bounds(1, 7).unwrap();
    for i in 1..=7 {
        knots[i] = f64::from(i - 1);
        mults[i] = if i == 1 || i == 7 { 4 } else { 1 };
    }
    bspline_from_knots(kind, poles, weights, &knots, &mults)
}
fn bspline_from_knots(
    kind: i32,
    poles: &Array2<'_, Point3>,
    weights: &Array2<'_, f64>,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let u = (knots, mults);
    let v = (knots, mults);
    if kind == 7 {
        return Ok(Handle::new(GeomBSplineSurface::new(
            poles,
            u,
            v,
            (3, 3),
            (false, false),
        )?)
        .cast());
    }
    Ok(Handle::new(GeomBSplineSurface::with_weights(
        poles,
        weights,
        u,
        v,
        (3, 3),
        (false, false),
    )?)
    .cast())
}
