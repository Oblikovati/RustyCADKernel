// Rust translation of ChFi3d_BuildPCurve and ChFi3d_mkbound (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The cubic pcurves joining two points with given tangents and the boundaries of the
//! corner fillings built on them.
use super::super::coefficient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_fill::{
    GeomFillBoundWithSurf, GeomFillBoundary, GeomFillSimpleBound,
};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, Vector3};

/// `std::max(a, b)`.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// `ChFi3d_BuildPCurve(p1, d1, p2, d2, redresse)`: the cubic Bezier from `p1` to `p2`
/// leaving along `d1` and arriving against `d2` (turned toward each other first when
/// `redresse`, in place as natively); `build_pcurve(p1, &mut d1, p2, &mut d2, true)?`.
pub fn build_pcurve(
    p1: Point2,
    d1: &mut Direction2,
    p2: Point2,
    d2: &mut Direction2,
    redresse: bool,
) -> Result<Handle<Geom2dBezierCurve>, Failure> {
    let vref = Vector2::between(p1, p2);
    let dref = Direction2::from_vector(vref)?;
    let mref = vref.magnitude();
    if redresse {
        if d1.dot(dref) < 0.0 {
            d1.reverse();
        }
        if d2.dot(dref) > 0.0 {
            d2.reverse();
        }
    }
    let lambda1 = std_max(d2.dot(*d1).abs(), dref.dot(*d1).abs());
    let lambda1 = std_max(0.5 * mref * lambda1, 1.0e-5);
    let pole2 = Point2::new(p1.x() + lambda1 * d1.x(), p1.y() + lambda1 * d1.y());
    let lambda2 = std_max(d1.dot(*d2).abs(), dref.dot(*d2).abs());
    let lambda2 = std_max(0.5 * mref * lambda2, 1.0e-5);
    let pole3 = Point2::new(p2.x() + lambda2 * d2.x(), p2.y() + lambda2 * d2.y());
    let mut poles = [p1, pole2, pole3, p2];
    Ok(Handle::new(Geom2dBezierCurve::new(&Array1::from_slice(
        &mut poles,
    ))?))
}

/// `ChFi3d_BuildPCurve(Surf, p1, v1, p2, v2, redresse)` with 2d tangents: the cubic
/// built in the parameters scaled by the resolutions of the surface, then scaled back;
/// `build_pcurve_on(&s, p1, v1, p2, v2, false)?`.
pub fn build_pcurve_on(
    surf: &Handle<dyn Adaptor3dSurface>,
    p1: Point2,
    v1: Vector2,
    p2: Point2,
    v2: Vector2,
    redresse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (ures, vres) = (surf.u_resolution(1.0)?, surf.v_resolution(1.0)?);
    let (invures, invvres) = (1.0 / ures, 1.0 / vres);
    let scaled = |p: (f64, f64)| (invures * p.0, invvres * p.1);
    let (pp1, pp2) = (scaled((p1.x(), p1.y())), scaled((p2.x(), p2.y())));
    let (vv1, vv2) = (scaled((v1.x(), v1.y())), scaled((v2.x(), v2.y())));
    let mut d1 = Direction2::from_vector(Vector2::new(vv1.0, vv1.1))?;
    let mut d2 = Direction2::from_vector(Vector2::new(vv2.0, vv2.1))?;
    let pc = build_pcurve(
        Point2::new(pp1.0, pp1.1),
        &mut d1,
        Point2::new(pp2.0, pp2.1),
        &mut d2,
        redresse,
    )?;
    let curve = pc.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "ChFi3d_BuildPCurve: null curve")
    })?;
    for ip in 1..=curve.nb_poles() {
        let pol = curve.pole(ip)?;
        curve.set_pole(ip, Point2::new(ures * pol.x(), vres * pol.y()))?;
    }
    Ok(pc.cast())
}

/// `ChFi3d_BuildPCurve(Surf, p1, v1, p2, v2, redresse)` with 3d tangents: their
/// components in the tangent planes at both points (turned toward each other when
/// `redresse`); `build_pcurve_3d(&s, p1, v1, p2, v2, false)?`.
pub fn build_pcurve_3d(
    surf: &Handle<dyn Adaptor3dSurface>,
    p1: Point2,
    v1: Vector3,
    p2: Point2,
    v2: Vector3,
    redresse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let d1 = surf.d1(p1.x(), p1.y())?;
    let (du, dv) = coefficient(v1, d1.d1u, d1.d1v);
    let mut vv1 = Vector2::new(du, dv);
    let d2 = surf.d1(p2.x(), p2.y())?;
    let (du, dv) = coefficient(v2, d2.d1u, d2.d1v);
    let mut vv2 = Vector2::new(du, dv);
    let vref = Vector3::between(d1.point, d2.point);
    if redresse {
        if vref.dot(v1) < 0.0 {
            vv1.reverse();
        }
        if vref.dot(v2) > 0.0 {
            vv2.reverse();
        }
    }
    build_pcurve_on(surf, p1, vv1, p2, vv2, false)
}

/// `ChFi3d_mkbound(HS, curv, t3d, ta, isfreeboundary)`: the boundary of the curve on
/// the surface, a simple bound on a free boundary else a bound with the surface;
/// `mkbound(&s, &pcurve, 1e-4, 1e-2, false)?`.
pub fn mkbound(
    hs: &Handle<dyn Adaptor3dSurface>,
    curv: &Handle<dyn Geom2dCurve>,
    t3d: f64,
    ta: f64,
    isfreeboundary: bool,
) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
    let hc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(curv)?).cast();
    let cons = Adaptor3dCurveOnSurface::with_curve(&hc, hs)?;
    if isfreeboundary {
        let hcons: Handle<dyn Adaptor3dCurve> =
            Handle::new(Adaptor3dCurveOnSurface::copy_of(&cons)).cast();
        return Ok(Handle::new(GeomFillSimpleBound::new(&hcons, t3d, ta)).cast());
    }
    Ok(Handle::new(GeomFillBoundWithSurf::new(&cons, t3d, ta)).cast())
}

/// The straight segment from `p1` to `p2` as a Bezier pcurve.
fn segment(p1: Point2, p2: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut poles = [p1, p2];
    Ok(Handle::new(Geom2dBezierCurve::new(&Array1::from_slice(&mut poles))?).cast())
}

/// `ChFi3d_mkbound(Fac, curv, p1, p2, t3d, ta, isfreeboundary)`: `(boundary, pcurve)` of
/// the straight segment from `p1` to `p2`; `mkbound_segment(&s, p1, p2, 1e-4, 1e-2,
/// false)?`.
pub fn mkbound_segment(
    fac: &Handle<dyn Adaptor3dSurface>,
    p1: Point2,
    p2: Point2,
    (t3d, ta): (f64, f64),
    isfreeboundary: bool,
) -> Result<BoundAndPCurve, Failure> {
    let curv = segment(p1, p2)?;
    Ok((mkbound(fac, &curv, t3d, ta, isfreeboundary)?, curv))
}

/// `ChFi3d_mkbound(HS, p1, p2, t3d, ta, isfreeboundary)`: the boundary of the straight
/// segment from `p1` to `p2`; `mkbound_points(&s, p1, p2, (1e-4, 1e-2), false)?`.
pub fn mkbound_points(
    hs: &Handle<dyn Adaptor3dSurface>,
    p1: Point2,
    p2: Point2,
    tolerances: (f64, f64),
    isfreeboundary: bool,
) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
    Ok(mkbound_segment(hs, p1, p2, tolerances, isfreeboundary)?.0)
}

/// `ChFi3d_mkbound(s, p1, p2, t3d, ta, isfreeboundary)` on a Geom surface: the boundary
/// of the straight segment; `mkbound_geom(&s, p1, p2, (1e-4, 1e-2), false)?`.
pub fn mkbound_geom(
    s: &Handle<dyn GeomSurface>,
    p1: Point2,
    p2: Point2,
    tolerances: (f64, f64),
    isfreeboundary: bool,
) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
    let hs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
    mkbound_points(&hs, p1, p2, tolerances, isfreeboundary)
}

/// A boundary and the pcurve it is built on.
pub type BoundAndPCurve = (Handle<dyn GeomFillBoundary>, Handle<dyn Geom2dCurve>);

/// The end of a boundary: the 2d point, the side (`sens == 1` reverses the tangent).
pub type BoundEnd<V> = (i32, Point2, V);

/// `ChFi3d_mkbound(Fac, curv, sens1, pfac1, vfac1, sens2, pfac2, vfac2, t3d, ta)`:
/// `(boundary, pcurve)` of the cubic joining the ends with their 2d tangents;
/// `mkbound_tangents_2d(&s, (0, p1, v1), (1, p2, v2), (1e-4, 1e-2))?`.
pub fn mkbound_tangents_2d(
    fac: &Handle<dyn Adaptor3dSurface>,
    (sens1, pfac1, vfac1): BoundEnd<Vector2>,
    (sens2, pfac2, vfac2): BoundEnd<Vector2>,
    (t3d, ta): (f64, f64),
) -> Result<BoundAndPCurve, Failure> {
    let mut v1 = Direction2::from_vector(vfac1)?;
    if sens1 == 1 {
        v1.reverse();
    }
    let mut v2 = Direction2::from_vector(vfac2)?;
    if sens2 == 1 {
        v2.reverse();
    }
    let curv = build_pcurve_on(
        fac,
        pfac1,
        Vector2::from(v1),
        pfac2,
        Vector2::from(v2),
        false,
    )?;
    Ok((mkbound(fac, &curv, t3d, ta, false)?, curv))
}

/// `ChFi3d_mkbound(Surf, curv, sens1, p1, v1, sens2, p2, v2, t3d, ta)`: `(boundary,
/// pcurve)` of the cubic joining the ends with their 3d tangents (reversed in place
/// by the sides as natively); `mkbound_tangents_3d(&s, (0, p1, &mut v1), (1, p2, &mut
/// v2), (1e-4, 1e-2))?`.
pub fn mkbound_tangents_3d(
    surf: &Handle<dyn Adaptor3dSurface>,
    (sens1, p1, v1): BoundEnd<&mut Vector3>,
    (sens2, p2, v2): BoundEnd<&mut Vector3>,
    (t3d, ta): (f64, f64),
) -> Result<BoundAndPCurve, Failure> {
    if sens1 == 1 {
        v1.reverse();
    }
    if sens2 == 1 {
        v2.reverse();
    }
    let curv = build_pcurve_3d(surf, p1, *v1, p2, *v2, false)?;
    Ok((mkbound(surf, &curv, t3d, ta, false)?, curv))
}
