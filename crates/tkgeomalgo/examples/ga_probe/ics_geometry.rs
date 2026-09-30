//! The geometry readers of the IntCurveSurface probe, the Rust twins of `ga_ics_curve`,
//! `ga_ics_surface` and their helpers in tools/oracle/tkgeomalgo_probe_intcurvesurface.cxx.
use super::common::Reader;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCircle,
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomOffsetSurface, GeomParabola, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::int_curve_surface::HCurve;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Point3};

/// Weights `flag [w...]` of `n` poles, or none.
fn weights(r: &mut Reader, n: i32) -> Option<Array1<'static, f64>> {
    if !r.flag() {
        return None;
    }
    Some(reals(r, n))
}

/// `n` reals from index 1.
fn reals(r: &mut Reader, n: i32) -> Array1<'static, f64> {
    let mut a: Array1<'static, f64> = Array1::with_bounds(1, n).expect("probe array");
    for i in 1..=n {
        a[i] = r.real();
    }
    a
}

/// Knots and multiplicities `nk knots... mults...`.
fn knots(r: &mut Reader) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let nk = r.integer();
    let k = reals(r, nk);
    let mut m: Array1<'static, i32> = Array1::with_bounds(1, nk).expect("probe mults");
    for i in 1..=nk {
        m[i] = r.integer();
    }
    (k, m)
}

fn points(r: &mut Reader, n: i32) -> Array1<'static, Point3> {
    let mut poles: Array1<'static, Point3> = Array1::with_bounds(1, n).expect("probe poles");
    for i in 1..=n {
        poles[i] = r.pnt();
    }
    poles
}

fn conic(r: &mut Reader, kind: i32) -> Handle<dyn GeomCurve> {
    let (p, d) = (r.pnt(), r.dir());
    if kind == 0 {
        return Handle::new(GeomLine::from_point(p, d)).cast();
    }
    let a = r.real();
    let ax = Axis2::new(p, d).expect("probe frame");
    match kind {
        1 => Handle::new(GeomCircle::new(ax, a).expect("probe circle")).cast(),
        3 => Handle::new(GeomParabola::new(ax, a).expect("probe parabola")).cast(),
        2 => Handle::new(GeomEllipse::new(ax, a, r.real()).expect("probe ellipse")).cast(),
        _ => Handle::new(GeomHyperbola::new(ax, a, r.real()).expect("probe hyperbola")).cast(),
    }
}

fn bezier_curve(r: &mut Reader) -> Handle<dyn GeomCurve> {
    let n = r.integer();
    let poles = points(r, n);
    let curve = match weights(r, n) {
        Some(w) => GeomBezierCurve::with_weights(&poles, &w),
        None => GeomBezierCurve::new(&poles),
    };
    Handle::new(curve.expect("probe Bezier curve")).cast()
}

fn bspline_curve(r: &mut Reader) -> Handle<dyn GeomCurve> {
    let (deg, n) = (r.integer(), r.integer());
    let poles = points(r, n);
    let w = weights(r, n);
    let (k, m) = knots(r);
    let periodic = r.flag();
    let curve = match w {
        Some(w) => GeomBSplineCurve::with_weights(&poles, &w, &k, &m, deg, periodic),
        None => GeomBSplineCurve::new(&poles, &k, &m, deg, periodic),
    };
    Handle::new(curve.expect("probe B-spline curve")).cast()
}

/// `ga_ics_geom_curve`: a curve `kind ...`.
pub fn geom_curve(r: &mut Reader) -> Handle<dyn GeomCurve> {
    let kind = r.integer();
    match kind {
        0..=4 => conic(r, kind),
        5 => bezier_curve(r),
        6 => bspline_curve(r),
        7 => {
            let off = r.real();
            let d = r.dir();
            let basis = geom_curve(r);
            Handle::new(GeomOffsetCurve::new(&basis, off, d, false).expect("probe offset curve"))
                .cast()
        }
        _ => {
            let (u1, u2) = (r.real(), r.real());
            let basis = geom_curve(r);
            Handle::new(
                GeomTrimmedCurve::new(&basis, u1, u2, true, true).expect("probe trimmed curve"),
            )
            .cast()
        }
    }
}

/// `ga_ics_curve`: a curve adaptor with its natural or given range.
pub fn curve(r: &mut Reader) -> HCurve {
    let c = geom_curve(r);
    if !r.flag() {
        return Handle::new(GeomAdaptorCurve::with_curve(&c).expect("probe adaptor")).cast();
    }
    let (a, b) = (r.real(), r.real());
    Handle::new(GeomAdaptorCurve::with_range(&c, a, b).expect("probe adaptor")).cast()
}

fn elementary(r: &mut Reader, kind: i32) -> Handle<dyn GeomSurface> {
    let (p, d) = (r.pnt(), r.dir());
    let ax = Axis3::from_normal(p, d).expect("probe frame");
    if kind == 0 {
        return Handle::new(GeomPlane::new(ax)).cast();
    }
    let a = r.real();
    match kind {
        1 => Handle::new(GeomCylindricalSurface::new(ax, a).expect("probe cylinder")).cast(),
        3 => Handle::new(GeomSphericalSurface::new(ax, a).expect("probe sphere")).cast(),
        2 => Handle::new(GeomConicalSurface::new(ax, a, r.real()).expect("probe cone")).cast(),
        _ => Handle::new(GeomToroidalSurface::new(ax, a, r.real()).expect("probe torus")).cast(),
    }
}

fn grid_surface(r: &mut Reader, kind: i32) -> Handle<dyn GeomSurface> {
    let (ud, vd) = if kind == 6 {
        (r.integer(), r.integer())
    } else {
        (0, 0)
    };
    let (nu, nv) = (r.integer(), r.integer());
    let mut poles: Array2<'static, Point3> =
        Array2::with_bounds(1, nu, 1, nv).expect("probe poles");
    for i in 1..=nu {
        for j in 1..=nv {
            poles[(i, j)] = r.pnt();
        }
    }
    if kind == 5 {
        return Handle::new(GeomBezierSurface::new(&poles).expect("probe Bezier surface")).cast();
    }
    let (uk, um) = knots(r);
    let (vk, vm) = knots(r);
    let surface = GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (ud, vd), (false, false));
    Handle::new(surface.expect("probe B-spline surface")).cast()
}

/// `ga_ics_geom_surface`: a surface `kind ...`.
pub fn geom_surface(r: &mut Reader) -> Handle<dyn GeomSurface> {
    let kind = r.integer();
    match kind {
        0..=4 => elementary(r, kind),
        5 | 6 => grid_surface(r, kind),
        7 => {
            let (p, d) = (r.pnt(), r.dir());
            let c = geom_curve(r);
            Handle::new(
                GeomSurfaceOfRevolution::new(&c, Axis1::new(p, d)).expect("probe revolution"),
            )
            .cast()
        }
        8 => {
            let d = r.dir();
            let c = geom_curve(r);
            Handle::new(GeomSurfaceOfLinearExtrusion::new(&c, d).expect("probe extrusion")).cast()
        }
        9 => {
            let off = r.real();
            let s = geom_surface(r);
            Handle::new(GeomOffsetSurface::new(&s, off, false).expect("probe offset surface"))
                .cast()
        }
        _ => {
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let s = geom_surface(r);
            let trimmed = GeomRectangularTrimmedSurface::new(&s, (u1, u2), (v1, v2), (true, true));
            Handle::new(trimmed.expect("probe trimmed surface")).cast()
        }
    }
}

/// `ga_ics_surface`: a surface adaptor with its natural or given bounds.
pub fn surface(r: &mut Reader) -> Adaptor3dSurfaceHandle {
    let s = geom_surface(r);
    if !r.flag() {
        return Handle::new(GeomAdaptorSurface::with_surface(&s).expect("probe adaptor")).cast();
    }
    let bounds = (r.real(), r.real(), r.real(), r.real());
    Handle::new(GeomAdaptorSurface::with_bounds(&s, bounds).expect("probe adaptor")).cast()
}

/// `ga_ics_params`: parameters `n values...` from index 1 (empty for `n <= 0`).
pub fn params(r: &mut Reader) -> Array1<'static, f64> {
    let n = r.integer();
    if n <= 0 {
        return Array1::with_bounds(1, 0).expect("empty array");
    }
    reals(r, n)
}
