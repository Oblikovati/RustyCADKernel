//! The curve and surface builders of tools/oracle/tkgeombase_probe_lprop_build.cxx.
use super::super::common::Reader;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomCircle, GeomConicalSurface,
    GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola, GeomLine, GeomOffsetCurve,
    GeomOffsetSurface, GeomParabola, GeomPlane, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Parabola2, Point3};

/// gb_lp_knots: `n` (knot, multiplicity) pairs.
pub fn knots(r: &mut Reader<'_>, n: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut k = Array1::with_bounds(1, n).unwrap();
    let mut m = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        k[i] = r.real();
        m[i] = r.integer();
    }
    (k, m)
}

// Poles with their weights: (x, y, w) or (x, y, z, w) records.
fn weighted<P: Copy + Default>(
    r: &mut Reader<'_>,
    point: fn(&mut Reader<'_>) -> P,
) -> (Array1<'static, P>, Array1<'static, f64>) {
    let n: i32 = r.integer();
    let mut p = Array1::with_bounds(1, n).unwrap();
    let mut w = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        p[i] = point(r);
        w[i] = r.real();
    }
    (p, w)
}

/// gb_lp_curve2d.
pub fn curve2d(r: &mut Reader<'_>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let kind: i32 = r.integer();
    Ok(match kind {
        0 => {
            let p = r.pnt2d();
            Handle::new(Geom2dLine::from_point(p, r.dir2d())).cast()
        }
        1..=4 => {
            let (a, s, r1) = (r.ax2d(), r.flag(), r.real());
            match kind {
                1 => Handle::new(Geom2dCircle::from_circ2d(Circle2::from_x_axis(a, r1, s)?)).cast(),
                4 => Handle::new(Geom2dParabola::from_parab2d(Parabola2::from_mirror_axis(
                    a, r1, s,
                )?))
                .cast(),
                2 => Handle::new(Geom2dEllipse::from_elips2d(Ellipse2::from_x_axis(
                    a,
                    r1,
                    r.real(),
                    s,
                )?))
                .cast(),
                _ => Handle::new(Geom2dHyperbola::from_hypr2d(Hyperbola2::from_x_axis(
                    a,
                    r1,
                    r.real(),
                    s,
                )?))
                .cast(),
            }
        }
        5 => {
            let (p, w) = weighted(r, |r| r.pnt2d());
            Handle::new(Geom2dBezierCurve::with_weights(&p, &w)?).cast()
        }
        6 => {
            let (deg, per) = (r.integer(), r.flag());
            let (p, w) = weighted(r, |r| r.pnt2d());
            let nk = r.integer();
            let (k, m) = knots(r, nk);
            Handle::new(Geom2dBSplineCurve::with_weights(&p, &w, &k, &m, deg, per)?).cast()
        }
        7 => {
            let (u1, u2) = (r.real(), r.real());
            let b = curve2d(r)?;
            Handle::new(Geom2dTrimmedCurve::new(&b, u1, u2, true, true)?).cast()
        }
        _ => {
            let d = r.real();
            let b = curve2d(r)?;
            Handle::new(Geom2dOffsetCurve::new(&b, d, false)?).cast()
        }
    })
}

/// gb_lp_curve3d.
pub fn curve3d(r: &mut Reader<'_>) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind: i32 = r.integer();
    Ok(match kind {
        0 => {
            let p = r.pnt();
            Handle::new(GeomLine::from_point(p, r.dir())).cast()
        }
        1..=4 => {
            let (a, r1) = (r.ax2(), r.real());
            match kind {
                1 => Handle::new(GeomCircle::new(a, r1)?).cast(),
                4 => Handle::new(GeomParabola::new(a, r1)?).cast(),
                2 => Handle::new(GeomEllipse::new(a, r1, r.real())?).cast(),
                _ => Handle::new(GeomHyperbola::new(a, r1, r.real())?).cast(),
            }
        }
        5 => {
            let (p, w) = weighted(r, |r| r.pnt());
            Handle::new(GeomBezierCurve::with_weights(&p, &w)?).cast()
        }
        6 => {
            let (deg, per) = (r.integer(), r.flag());
            let (p, w) = weighted(r, |r| r.pnt());
            let nk = r.integer();
            let (k, m) = knots(r, nk);
            Handle::new(GeomBSplineCurve::with_weights(&p, &w, &k, &m, deg, per)?).cast()
        }
        7 => {
            let (u1, u2) = (r.real(), r.real());
            let b = curve3d(r)?;
            Handle::new(GeomTrimmedCurve::new(&b, u1, u2, true, true)?).cast()
        }
        _ => {
            let (d, v) = (r.real(), r.dir());
            let b = curve3d(r)?;
            Handle::new(GeomOffsetCurve::new(&b, d, v, false)?).cast()
        }
    })
}

/// gb_lp_surface.
pub fn surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind: i32 = r.integer();
    Ok(match kind {
        ..=4 => {
            let a = r.ax3();
            if kind == 0 {
                return Ok(Handle::new(GeomPlane::new(a)).cast());
            }
            let r1 = r.real();
            match kind {
                1 => Handle::new(GeomCylindricalSurface::new(a, r1)?).cast(),
                3 => Handle::new(GeomSphericalSurface::new(a, r1)?).cast(),
                2 => Handle::new(GeomConicalSurface::new(a, r1, r.real())?).cast(),
                _ => Handle::new(GeomToroidalSurface::new(a, r1, r.real())?).cast(),
            }
        }
        5 => bspline_surface(r)?,
        6 => {
            let a = r.ax1();
            let c = curve3d(r)?;
            Handle::new(GeomSurfaceOfRevolution::new(&c, a)?).cast()
        }
        7 => {
            let d = r.dir();
            let c = curve3d(r)?;
            Handle::new(GeomSurfaceOfLinearExtrusion::new(&c, d)?).cast()
        }
        _ => {
            let d = r.real();
            let s = surface(r)?;
            Handle::new(GeomOffsetSurface::new(&s, d, false)?).cast()
        }
    })
}

fn bspline_surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (du, dv, nu, nv) = (r.integer(), r.integer(), r.integer(), r.integer());
    let mut p: Array2<'static, Point3> = Array2::with_bounds(1, nu, 1, nv).unwrap();
    let mut w: Array2<'static, f64> = Array2::with_bounds(1, nu, 1, nv).unwrap();
    for i in 1..=nu {
        for j in 1..=nv {
            p[(i, j)] = r.pnt();
            w[(i, j)] = r.real();
        }
    }
    let nku = r.integer();
    let (ku, mu) = knots(r, nku);
    let nkv = r.integer();
    let (kv, mv) = knots(r, nkv);
    let s =
        GeomBSplineSurface::with_weights(&p, &w, (&ku, &mu), (&kv, &mv), (du, dv), (false, false))?;
    Ok(Handle::new(s).cast())
}
