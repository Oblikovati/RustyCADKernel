//! Builders of the geometry the GeomTools write cases print, from the recursive numeric
//! spec of tools/oracle/tkgeombase_probe_geomtools_build.cxx (same kinds, same order).
use super::super::common::Reader;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_eval::Geom2dEvalSineWaveCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCircle,
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomOffsetSurface, GeomParabola, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_eval::{GeomEvalCircularHelixCurve, GeomEvalEllipsoidSurface};
use rustycad_tkmath::gp::{Axis3, Point2, Point3};

type Built<T> = Result<Handle<T>, Failure>;

fn up<T: Transient + RuntimeTyped, U: ?Sized + RuntimeTyped>(value: T) -> Handle<U> {
    Handle::new(value).into_cast()
}

/// `gt_ax3`: a frame and an indirect flag.
pub fn ax3(r: &mut Reader<'_>) -> Axis3 {
    let mut frame = r.ax3();
    if r.flag() {
        frame.y_reverse();
    }
    frame
}

fn points<P: Default>(
    r: &mut Reader<'_>,
    n: i32,
    read: fn(&mut Reader<'_>) -> P,
) -> Array1<'static, P> {
    let mut array = Array1::with_bounds(1, n).expect("probe count");
    for i in 1..=n {
        array.set_value(i, read(r)).expect("probe index");
    }
    array
}

fn reals(r: &mut Reader<'_>, n: i32) -> Array1<'static, f64> {
    points(r, n, |r| r.real())
}

fn ints(r: &mut Reader<'_>, n: i32) -> Array1<'static, i32> {
    points(r, n, |r| r.integer())
}

/// `gt_curve`: 1 line ... 9 offset, 10 GeomEval helix.
pub fn curve(r: &mut Reader<'_>) -> Built<dyn GeomCurve> {
    Ok(match r.integer() {
        1 => {
            let p = r.pnt();
            up(GeomLine::from_point(p, r.dir()))
        }
        2 => {
            let a = r.ax2();
            up(GeomCircle::new(a, r.real())?)
        }
        3 => {
            let (a, m) = (r.ax2(), r.real());
            up(GeomEllipse::new(a, m, r.real())?)
        }
        4 => {
            let a = r.ax2();
            up(GeomParabola::new(a, r.real())?)
        }
        5 => {
            let (a, m) = (r.ax2(), r.real());
            up(GeomHyperbola::new(a, m, r.real())?)
        }
        6 => bezier_curve(r)?,
        7 => bspline_curve(r)?,
        8 => {
            let (u1, u2, sense) = (r.real(), r.real(), r.flag());
            let basis = curve(r)?;
            up(GeomTrimmedCurve::new(&basis, u1, u2, sense, true)?)
        }
        9 => {
            let (offset, d) = (r.real(), r.dir());
            let basis = curve(r)?;
            up(GeomOffsetCurve::new(&basis, offset, d, false)?)
        }
        10 => {
            let (a, radius) = (r.ax2(), r.real());
            up(GeomEvalCircularHelixCurve::new(a, radius, r.real())?)
        }
        _ => Handle::default(),
    })
}

fn bezier_curve(r: &mut Reader<'_>) -> Built<dyn GeomCurve> {
    let (rational, n) = (r.flag(), r.integer());
    let p = points(r, n, |r| r.pnt());
    if rational {
        return Ok(up(GeomBezierCurve::with_weights(&p, &reals(r, n))?));
    }
    Ok(up(GeomBezierCurve::new(&p)?))
}

fn bspline_curve(r: &mut Reader<'_>) -> Built<dyn GeomCurve> {
    let (rational, periodic, degree, n) = (r.flag(), r.flag(), r.integer(), r.integer());
    let p = points(r, n, |r| r.pnt());
    let w = if rational {
        reals(r, n)
    } else {
        Array1::with_bounds(1, n).expect("probe count")
    };
    let k = r.integer();
    let (knots, mults) = (reals(r, k), ints(r, k));
    if rational {
        return Ok(up(GeomBSplineCurve::with_weights(
            &p, &w, &knots, &mults, degree, periodic,
        )?));
    }
    Ok(up(GeomBSplineCurve::new(
        &p, &knots, &mults, degree, periodic,
    )?))
}

/// `gt_curve2d`: the same kinds; 10 is the Geom2dEval sine wave.
pub fn curve2d(r: &mut Reader<'_>) -> Built<dyn Geom2dCurve> {
    Ok(match r.integer() {
        1 => {
            let p = r.pnt2d();
            up(Geom2dLine::from_point(p, r.dir2d()))
        }
        2 => {
            let a = r.ax22d();
            up(Geom2dCircle::new(a, r.real())?)
        }
        3 => {
            let (a, m) = (r.ax22d(), r.real());
            up(Geom2dEllipse::new(a, m, r.real())?)
        }
        4 => {
            let a = r.ax22d();
            up(Geom2dParabola::new(a, r.real())?)
        }
        5 => {
            let (a, m) = (r.ax22d(), r.real());
            up(Geom2dHyperbola::new(a, m, r.real())?)
        }
        6 => bezier_curve2d(r)?,
        7 => bspline_curve2d(r)?,
        8 => {
            let (u1, u2, sense) = (r.real(), r.real(), r.flag());
            let basis = curve2d(r)?;
            up(Geom2dTrimmedCurve::new(&basis, u1, u2, sense, true)?)
        }
        9 => {
            let offset = r.real();
            let basis = curve2d(r)?;
            up(Geom2dOffsetCurve::new(&basis, offset, false)?)
        }
        10 => {
            let (a, amplitude, omega) = (r.ax2d(), r.real(), r.real());
            up(Geom2dEvalSineWaveCurve::new(a, amplitude, omega, r.real())?)
        }
        _ => Handle::default(),
    })
}

fn bezier_curve2d(r: &mut Reader<'_>) -> Built<dyn Geom2dCurve> {
    let (rational, n) = (r.flag(), r.integer());
    let p: Array1<'static, Point2> = points(r, n, |r| r.pnt2d());
    if rational {
        return Ok(up(Geom2dBezierCurve::with_weights(&p, &reals(r, n))?));
    }
    Ok(up(Geom2dBezierCurve::new(&p)?))
}

fn bspline_curve2d(r: &mut Reader<'_>) -> Built<dyn Geom2dCurve> {
    let (rational, periodic, degree, n) = (r.flag(), r.flag(), r.integer(), r.integer());
    let p: Array1<'static, Point2> = points(r, n, |r| r.pnt2d());
    let w = if rational {
        reals(r, n)
    } else {
        Array1::with_bounds(1, n).expect("probe count")
    };
    let k = r.integer();
    let (knots, mults) = (reals(r, k), ints(r, k));
    if rational {
        return Ok(up(Geom2dBSplineCurve::with_weights(
            &p, &w, &knots, &mults, degree, periodic,
        )?));
    }
    Ok(up(Geom2dBSplineCurve::new(
        &p, &knots, &mults, degree, periodic,
    )?))
}

fn grid<T: Default>(
    r: &mut Reader<'_>,
    nu: i32,
    nv: i32,
    read: fn(&mut Reader<'_>) -> T,
) -> Array2<'static, T> {
    let mut array = Array2::with_bounds(1, nu, 1, nv).expect("probe grid");
    for i in 1..=nu {
        for j in 1..=nv {
            array.set_value(i, j, read(r)).expect("probe index");
        }
    }
    array
}

/// `gt_surface`: 1 plane ... 11 offset, 12 GeomEval ellipsoid.
pub fn surface(r: &mut Reader<'_>) -> Built<dyn GeomSurface> {
    Ok(match r.integer() {
        1 => up(GeomPlane::new(ax3(r))),
        2 => {
            let a = ax3(r);
            up(GeomCylindricalSurface::new(a, r.real())?)
        }
        3 => {
            let (a, angle) = (ax3(r), r.real());
            up(GeomConicalSurface::new(a, angle, r.real())?)
        }
        4 => {
            let a = ax3(r);
            up(GeomSphericalSurface::new(a, r.real())?)
        }
        5 => {
            let (a, m) = (ax3(r), r.real());
            up(GeomToroidalSurface::new(a, m, r.real())?)
        }
        6 => {
            let d = r.dir();
            let c = curve(r)?;
            up(GeomSurfaceOfLinearExtrusion::new(&c, d)?)
        }
        7 => {
            let a = r.ax1();
            let c = curve(r)?;
            up(GeomSurfaceOfRevolution::new(&c, a)?)
        }
        8 => bezier_surface(r)?,
        9 => bspline_surface(r)?,
        10 => {
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let basis = surface(r)?;
            up(GeomRectangularTrimmedSurface::new(
                &basis,
                (u1, u2),
                (v1, v2),
                (true, true),
            )?)
        }
        11 => {
            let offset = r.real();
            let basis = surface(r)?;
            up(GeomOffsetSurface::new(&basis, offset, false)?)
        }
        12 => {
            let (a, x, y) = (ax3(r), r.real(), r.real());
            up(GeomEvalEllipsoidSurface::new(a, x, y, r.real())?)
        }
        _ => Handle::default(),
    })
}

fn bezier_surface(r: &mut Reader<'_>) -> Built<dyn GeomSurface> {
    let (rational, nu, nv) = (r.flag(), r.integer(), r.integer());
    let p: Array2<'static, Point3> = grid(r, nu, nv, |r| r.pnt());
    if rational {
        return Ok(up(GeomBezierSurface::with_weights(
            &p,
            &grid(r, nu, nv, |r| r.real()),
        )?));
    }
    Ok(up(GeomBezierSurface::new(&p)?))
}

fn bspline_surface(r: &mut Reader<'_>) -> Built<dyn GeomSurface> {
    let (rational, up_, vp) = (r.flag(), r.flag(), r.flag());
    let (ud, vd, nu, nv) = (r.integer(), r.integer(), r.integer(), r.integer());
    let p: Array2<'static, Point3> = grid(r, nu, nv, |r| r.pnt());
    let w = if rational {
        grid(r, nu, nv, |r| r.real())
    } else {
        Array2::with_bounds(1, nu, 1, nv).expect("probe grid")
    };
    let ku = r.integer();
    let (uk, um) = (reals(r, ku), ints(r, ku));
    let kv = r.integer();
    let (vk, vm) = (reals(r, kv), ints(r, kv));
    let (u, v) = ((&uk, &um), (&vk, &vm));
    if rational {
        return Ok(up(GeomBSplineSurface::with_weights(
            &p,
            &w,
            u,
            v,
            (ud, vd),
            (up_, vp),
        )?));
    }
    Ok(up(GeomBSplineSurface::new(&p, u, v, (ud, vd), (up_, vp))?))
}
