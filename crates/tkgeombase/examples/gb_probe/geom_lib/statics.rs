//! The GeomLib static member requests; the counterpart of
//! tools/oracle/tkgeombase_probe_geomlib.cxx (same request layout and token stream).
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
use super::super::common::{Reader, attempt, ax2, dir, end, flag, int, pnt, real};
use super::super::geom_convert::{
    UNSET, bspline_surface, curve, curve2d, read_bspline_surface, surface,
};
use super::super::geom_tools::build;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomBoundedCurve, GeomBoundedSurface, GeomCurve,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_lib::{self as gl, IsoLine};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Direction3, GeneralTransform2, Point2, Point3};

const SHAPES: [Shape; 7] = super::super::geom2d_convert::SHAPES;

fn curve_on_surface(
    c: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn GeomSurface>,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let c2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_curve(c)?).cast();
    let surf: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
    Adaptor3dCurveOnSurface::with_curve(&c2d, &surf)
}

/// gl_to3d.
pub fn to3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let a = r.ax2();
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        Ok(curve(&gl::to_3d(a, &c)?))
    });
    end();
}

/// gl_gtransform.
pub fn gtransform_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut g = GeneralTransform2::default();
    for i in 1..=2 {
        for j in 1..=2 {
            g.set_value(i, j, r.real()).unwrap();
        }
    }
    let (t1, t2) = (r.real(), r.real());
    g.set_value(1, 3, t1).unwrap();
    g.set_value(2, 3, t2).unwrap();
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        Ok(curve2d(&gl::gtransform(&c, &g)?))
    });
    end();
}

/// gl_same_range.
pub fn same_range_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (tol, f, l, rf, rl) = (r.real(), r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let mut nc: Handle<dyn Geom2dCurve> = Handle::default();
        attempt(|| gl::same_range(tol, &c, (f, l), (rf, rl), &mut nc));
        Ok(curve2d(&nc))
    });
    end();
}

/// gl_build_curve3d.
pub fn build_curve3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (tol, f, l) = (r.real(), r.real(), r.real());
    let cont = SHAPES[r.integer() as usize];
    let (deg, seg) = (r.integer(), r.integer());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let s = build::surface(&mut r)?;
        let cos = curve_on_surface(&c, &s)?;
        let mut nc: Handle<dyn GeomCurve> = Handle::default();
        let (mut mx, mut av) = (UNSET, UNSET);
        attempt(|| {
            (mx, av) = gl::build_curve3d(tol, &cos, (f, l), &mut nc, cont, deg, seg)?;
            Ok(())
        });
        curve(&nc);
        real(mx);
        Ok(real(av))
    });
    end();
}

/// gl_adjust.
pub fn adjust_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p1, p2, t1, t2) = (r.pnt(), r.pnt(), r.vec(), r.vec());
    attempt(|| {
        let mut c: Handle<dyn GeomBoundedCurve> = build::curve(&mut r)?.cast();
        attempt(|| gl::adjust_extremity(&mut c, p1, p2, t1, t2));
        Ok(curve(&c.cast()))
    });
    end();
}

/// gl_extend_curve.
pub fn extend_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (cont, after, p) = (r.integer(), r.flag(), r.pnt());
    attempt(|| {
        let mut c: Handle<dyn GeomBoundedCurve> = build::curve(&mut r)?.cast();
        attempt(|| gl::extend_curve_to_point(&mut c, p, cont, after));
        Ok(curve(&c.cast()))
    });
    end();
}

/// gl_extend_surf.
pub fn extend_surf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (len, cont, in_u, after) = (r.real(), r.integer(), r.flag(), r.flag());
    attempt(|| {
        let mut s: Handle<dyn GeomBoundedSurface> = build::surface(&mut r)?.cast();
        attempt(|| gl::extend_surf_by_length(&mut s, len, cont, in_u, after));
        Ok(surface(&s.cast()))
    });
    end();
}

/// gl_inertia.
pub fn inertia_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (tol, n) = (r.real(), r.integer());
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| p[i] = r.pnt());
    let (mut b, mut x, mut y) = (
        Point3::default(),
        Direction3::default(),
        Direction3::default(),
    );
    let mut gaps = [UNSET; 3];
    attempt(|| {
        let axes = gl::inertia(&p)?;
        (b, x, y, gaps) = (
            axes.bary,
            axes.x_dir,
            axes.y_dir,
            [axes.x_gap, axes.y_gap, axes.z_gap],
        );
        Ok(())
    });
    pnt(b);
    dir(x);
    dir(y);
    gaps.into_iter().for_each(real);
    let (mut a, mut singular) = (Axis2::default(), false);
    attempt(|| {
        let axe = gl::axe_of_inertia(&p, tol)?;
        (a, singular) = (axe.axe, axe.is_singular);
        Ok(())
    });
    ax2(a);
    flag(singular);
    end();
}

fn reals(a: &Array1<'_, f64>) {
    int(a.lower());
    int(a.upper());
    a.iter().for_each(|v| real(*v));
}

fn read_reals(r: &mut Reader<'_>) -> Array1<'static, f64> {
    let (lower, n) = (r.integer(), r.integer());
    if n == 0 {
        return Array1::default();
    }
    let mut a = Array1::with_bounds(lower, lower + n - 1).unwrap();
    (lower..lower + n).for_each(|i| a[i] = r.real());
    a
}

/// gl_arrays.
pub fn arrays_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, count) = (r.integer(), r.integer());
    let a = read_reals(&mut r);
    if mode == 2 {
        let b = read_reals(&mut r);
        let (conf, adjust) = (r.real(), r.flag());
        let mut f: LinearVector<f64> = LinearVector::default();
        f.append(-1.0);
        gl::fuse_intervals(&a, &b, &mut f, conf, adjust);
        int(f.len() as i32);
        f.iter().for_each(|v| real(*v));
        return end();
    }
    let mut out: Array1<'static, f64> = Array1::default();
    attempt(|| {
        if mode == 0 {
            gl::remove_points_from_array(count, &a, &mut out)?;
        } else {
            gl::densify_array1_of_real(count, &a, &mut out)?;
        }
        Ok(reals(&out))
    });
    end();
}

/// gl_eval_max.
pub fn eval_max_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, tol) = (r.integer(), r.real());
    let p = read_reals(&mut r);
    attempt(|| {
        let a = build::curve(&mut r)?;
        let b = build::curve(&mut r)?;
        let (ca, cb) = (
            GeomAdaptorCurve::with_curve(&a)?,
            GeomAdaptorCurve::with_curve(&b)?,
        );
        let mut d = UNSET;
        attempt(|| {
            d = if mode == 0 {
                gl::eval_max_parametric_distance(&ca, &cb, tol, &p)?
            } else {
                gl::eval_max_distance_along_parameter(&ca, &cb, tol, &p)?
            };
            Ok(())
        });
        Ok(real(d))
    });
    end();
}

/// gl_cancel_denom.
pub fn cancel_denom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (ud, vd) = (r.flag(), r.flag());
    attempt(|| {
        let mut s = read_bspline_surface(&mut r)?;
        attempt(|| gl::cancel_denominator_derivative(&mut s, ud, vd));
        Ok(bspline_surface(&s))
    });
    end();
}

/// gl_norm_estim.
pub fn norm_estim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (u, v, tol) = (r.real(), r.real(), r.real());
    attempt(|| {
        let s = build::surface(&mut r)?;
        let mut n = Direction3::default();
        int(gl::norm_estim(&s, Point2::new(u, v), tol, &mut n)?);
        Ok(dir(n))
    });
    end();
}

/// gl_is_closed.
pub fn is_closed_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (p1, p2, tol) = (r.real(), r.real(), r.real());
    attempt(|| {
        let s = build::surface(&mut r)?;
        if mode == 0 {
            let (u, v) = gl::is_closed(&s, tol)?;
            flag(u);
            return Ok(flag(v));
        }
        if mode <= 2 {
            let b: Handle<GeomBSplineSurface> = s.cast();
            let closed = if mode == 1 {
                gl::is_bspl_u_closed(&b, p1, p2, tol)?
            } else {
                gl::is_bspl_v_closed(&b, p1, p2, tol)?
            };
            return Ok(flag(closed));
        }
        let b: Handle<GeomBezierSurface> = s.cast();
        let closed = if mode == 3 {
            gl::is_bz_u_closed(&b, p1, p2, tol)?
        } else {
            gl::is_bz_v_closed(&b, p1, p2, tol)?
        };
        Ok(flag(closed))
    });
    end();
}

/// gl_iso_line.
pub fn iso_line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (f, l, tol, explicit) = (r.real(), r.real(), r.real(), r.flag());
    let given = IsoLine {
        is_u: r.flag(),
        param: r.real(),
        is_forward: r.flag(),
    };
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let s = build::surface(&mut r)?;
        let c2d: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_range(&c, f, l)?).cast();
        let sa: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
        let detected = gl::is_iso_line(&c2d)?;
        let shown = detected.unwrap_or(IsoLine {
            is_u: false,
            param: UNSET,
            is_forward: false,
        });
        flag(detected.is_some());
        flag(shown.is_u);
        real(shown.param);
        flag(shown.is_forward);
        let iso = if explicit { given } else { shown };
        if detected.is_some() || explicit {
            attempt(|| {
                Ok(curve(&gl::build_c3d_on_iso_line(
                    &c2d,
                    &sa,
                    (f, l),
                    tol,
                    iso,
                )?))
            });
        }
        Ok(())
    });
    end();
}
