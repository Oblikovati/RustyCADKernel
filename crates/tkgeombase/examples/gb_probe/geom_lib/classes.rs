//! The GeomLib class requests; the counterpart of
//! tools/oracle/tkgeombase_probe_geomlib_classes.cxx (same request layout and token
//! stream, same probe function for the AdvApprox evaluator).
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
use super::super::common::{Reader, attempt, end, flag, int, lin2d, pln, pnt2d, real, vec2d};
use super::super::geom_convert::{UNSET, bspline, read_bspline, read_bspline_surface};
use super::super::geom_tools::build;
use super::super::geom2d_convert::{SHAPES, bspline as bspline2d};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::adv_approx::{AdvApproxApproxAFunction, AdvApproxEvaluatorFunction};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::geom_lib::{
    Check2dBSplineCurve, CheckBSplineCurve, CheckCurveOnSurface, DenominatorMultiplier,
    Interpolate, IsPlanarSurface, LogSample, MakeCurvefromApprox, PolyFunc, Tool,
    compute_deviation, compute_deviation_pso,
};
use rustycad_tkmath::gp::{Point2, Point3, Vector2};
use rustycad_tkmath::math::{Function, FunctionWithDerivative, Vector};

/// gl_check_bspline.
pub fn check_bspline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let dim = r.integer();
    let (tol, ang, ff, fl) = (r.real(), r.real(), r.flag(), r.flag());
    if dim == 3 {
        attempt(|| {
            let c = read_bspline(&mut r)?;
            let mut k = CheckBSplineCurve::new(&c, tol, ang)?;
            flag(k.is_done());
            let (a, b) = k.need_tangent_fix();
            flag(a);
            flag(b);
            attempt(|| Ok(bspline(&k.fixed_tangent(ff, fl)?)));
            attempt(|| k.fix_tangent(ff, fl));
            Ok(bspline(&c))
        });
        return end();
    }
    attempt(|| {
        let c: Handle<Geom2dBSplineCurve> = build::curve2d(&mut r)?.cast();
        let mut k = Check2dBSplineCurve::new(&c, tol, ang)?;
        flag(k.is_done());
        let (a, b) = k.need_tangent_fix();
        flag(a);
        flag(b);
        attempt(|| Ok(bspline2d(&k.fixed_tangent(ff, fl)?)));
        attempt(|| k.fix_tangent(ff, fl));
        Ok(bspline2d(&c))
    });
    end();
}

/// gl_cos.
pub fn cos_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (f, l, f2, l2, tol, par) = (r.real(), r.real(), r.real(), r.real(), r.real(), r.flag());
    attempt(|| {
        let c = build::curve(&mut r)?;
        let p = build::curve2d(&mut r)?;
        let s = build::surface(&mut r)?;
        let ca: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_range(&c, f, l)?).cast();
        let c2d: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_range(&p, f2, l2)?).cast();
        let sa: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
        let cos = Handle::new(Adaptor3dCurveOnSurface::with_curve(&c2d, &sa)?);
        let mut k = if mode == 0 {
            CheckCurveOnSurface::with_curve(&ca, tol)
        } else {
            CheckCurveOnSurface::new()
        };
        if mode != 0 {
            k.init(&ca, tol);
        }
        if mode == 2 {
            k.init_empty();
        }
        k.set_parallel(par);
        flag(k.is_parallel());
        k.perform(&cos);
        flag(k.is_done());
        int(k.error_status());
        real(k.max_distance());
        Ok(real(k.max_parameter()))
    });
    end();
}

/// gl_denom.
pub fn denom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let nk = r.integer();
    let mut k: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    (1..=nk).for_each(|i| k[i] = r.real());
    let n = r.integer();
    let uv: Vec<f64> = (0..2 * n).map(|_| r.real()).collect();
    attempt(|| {
        let s = read_bspline_surface(&mut r)?;
        let d = DenominatorMultiplier::new(&s, &k)?;
        for pair in uv.chunks(2) {
            attempt(|| Ok(real(d.value(pair[0], pair[1])?)));
        }
        Ok(())
    });
    end();
}

/// gl_interpolate.
pub fn interpolate_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (deg, n) = (r.integer(), r.integer());
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, n).unwrap();
    let mut t: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| p[i] = r.pnt());
    (1..=n).for_each(|i| t[i] = r.real());
    attempt(|| {
        let g = Interpolate::new(deg, n, &p, &t)?;
        flag(g.is_done());
        if g.is_done() {
            return Ok(bspline(&g.curve()));
        }
        Ok(int(g.error() as i32))
    });
    end();
}

/// gl_is_planar.
pub fn is_planar_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    attempt(|| {
        let s = build::surface(&mut r)?;
        let k = IsPlanarSurface::new(&s, tol)?;
        flag(k.is_planar());
        attempt(|| Ok(pln(k.plan()?)));
        Ok(())
    });
    end();
}

/// gl_log_sample.
pub fn log_sample_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b, n) = (r.real(), r.real(), r.integer());
    let s = LogSample::new(a, b, n);
    for i in 0..=n + 1 {
        attempt(|| Ok(real(s.get_parameter(i)?)));
    }
    end();
}

/// gl_poly_func.
pub fn poly_func_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (lower, n) = (r.integer(), r.integer());
    let mut c: Vector<'static> = Vector::with_bounds(lower, lower + n - 1).unwrap();
    (lower..lower + n).for_each(|i| c[i] = r.real());
    let x = r.real();
    attempt(|| {
        let mut f = PolyFunc::new(&c)?;
        let (mut v, mut d) = (UNSET, UNSET);
        let ok = f.value(x, &mut v);
        f.take_failure()?;
        flag(ok);
        real(v);
        let ok = f.derivative(x, &mut d);
        f.take_failure()?;
        flag(ok);
        real(d);
        (v, d) = (UNSET, UNSET);
        let ok = f.values(x, &mut v, &mut d);
        f.take_failure()?;
        flag(ok);
        real(v);
        Ok(real(d))
    });
    end();
}

/// gl_tool.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, md, p) = (r.integer(), r.real(), r.pnt());
    attempt(|| {
        let (found, u, v) = match mode {
            0 => one_parameter(Tool::parameter(&build::curve(&mut r)?, p, md)?),
            1 => match Tool::parameters(&build::surface(&mut r)?, p, md)? {
                Some((u, v)) => (true, u, v),
                None => (false, 0.0, 0.0),
            },
            _ => {
                let c = build::curve2d(&mut r)?;
                one_parameter(Tool::parameter_2d(&c, Point2::new(p.x(), p.y()), md)?)
            }
        };
        flag(found);
        real(u);
        Ok(real(v))
    });
    end();
}

// A curve parameter search: found, U (0 when not found), V unset.
fn one_parameter(found: Option<f64>) -> (bool, f64, f64) {
    match found {
        Some(u) => (true, u, UNSET),
        None => (false, 0.0, UNSET),
    }
}

/// gl_deviation.
pub fn deviation_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, f, l, st) = (r.integer(), r.real(), r.real(), r.real());
    let (it, sub) = (r.integer(), r.integer());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let a = Geom2dAdaptorCurve::with_curve(&c)?;
        if mode == 0 {
            let d = compute_deviation(&a, f, l, st, it)?;
            real(d.deviation);
            real(d.prm_on_curve.unwrap_or(UNSET));
            pnt2d(d.pt_on_curve.unwrap_or(Point2::new(UNSET, UNSET)));
            vec2d(d.vec_curv_line.unwrap_or(Vector2::new(UNSET, UNSET)));
            return Ok(lin2d(d.line.unwrap_or_default()));
        }
        let (d, prm) = compute_deviation_pso(&a, f, l, sub, it)?;
        real(d);
        Ok(real(prm.unwrap_or(UNSET)))
    });
    end();
}

// The probe function of gb_gl_aa_value (kind 0 cubic, 1 waves, otherwise a helix).
fn aa_value(kind: i32, c: i32, t: f64, d: i32) -> f64 {
    let s = f64::from(c) + 1.0;
    match kind {
        0 => match d {
            0 => ((s * t - 2.0) * t + 0.5) * t + f64::from(c),
            1 => (3.0 * s * t - 4.0) * t + 0.5,
            _ => 6.0 * s * t - 4.0,
        },
        1 => match d {
            0 => (s * t).sin() + f64::from(c) * (0.5 * t).cos(),
            1 => s * (s * t).cos() - 0.5 * f64::from(c) * (0.5 * t).sin(),
            _ => -s * s * (s * t).sin() - 0.25 * f64::from(c) * (0.5 * t).cos(),
        },
        _ => helix_value(c, t, d),
    }
}

fn helix_value(c: i32, t: f64, d: i32) -> f64 {
    let rad = 2.0 + f64::from(c / 3);
    match (c % 3, d) {
        (0, 0) => rad * t.cos(),
        (0, 1) => -rad * t.sin(),
        (0, _) => -rad * t.cos(),
        (1, 0) => rad * t.sin(),
        (1, 1) => rad * t.cos(),
        (1, _) => -rad * t.sin(),
        (_, 0) => 1.0 + 0.3 * t,
        (_, 1) => 0.3,
        _ => 0.0,
    }
}

/// The probe evaluator (GbGlEvaluator).
struct Evaluator {
    kind: i32,
}

impl AdvApproxEvaluatorFunction for Evaluator {
    fn evaluate(&mut self, dimension: i32, _: [f64; 2], t: f64, d: i32, result: &mut [f64]) -> i32 {
        for c in 0..dimension {
            result[c as usize] = aa_value(self.kind, c, t, d);
        }
        0
    }
}

fn tolerances(n: i32, tol: f64) -> Option<Array1<'static, f64>> {
    (n > 0).then(|| {
        let mut a = Array1::with_bounds(1, n).unwrap();
        a.init(tol);
        a
    })
}

/// gl_make_curve.
pub fn make_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut f = Evaluator { kind: r.integer() };
    let (n1, n2, n3) = (r.integer(), r.integer(), r.integer());
    let (tol, first, last) = (r.real(), r.real(), r.real());
    let cont = SHAPES[r.integer() as usize];
    let (deg, seg) = (r.integer(), r.integer());
    let (t1, t2, t3) = (
        tolerances(n1, tol),
        tolerances(n2, tol),
        tolerances(n3, tol),
    );
    attempt(|| {
        let a = AdvApproxApproxAFunction::new(
            n1,
            n2,
            n3,
            t1.as_ref(),
            t2.as_ref(),
            t3.as_ref(),
            first,
            last,
            cont,
            deg,
            seg,
            &mut f,
        )?;
        make_curve_result(&MakeCurvefromApprox::new(&a), (n1, n2, n3))
    });
    end();
}

fn make_curve_result(
    m: &MakeCurvefromApprox<'_>,
    (n1, n2, n3): (i32, i32, i32),
) -> Result<(), Failure> {
    flag(m.is_done());
    int(m.nb_1d_spaces()?);
    int(m.nb_2d_spaces()?);
    int(m.nb_3d_spaces()?);
    for i in (0..=n2 + 1).filter(|_| n2 > 0) {
        attempt(|| Ok(bspline2d(&m.curve2d(i)?)));
        if n1 > 0 {
            attempt(|| Ok(bspline2d(&m.curve2d_weighted(1, i)?)));
        }
    }
    for i in (0..=n1 + 1).filter(|_| n1 > 0) {
        attempt(|| Ok(bspline2d(&m.curve2d_from_two_1d(i, 2)?)));
    }
    for i in (0..=n3 + 1).filter(|_| n3 > 0) {
        attempt(|| Ok(bspline(&m.curve(i)?)));
        if n1 > 0 {
            attempt(|| Ok(bspline(&m.curve_weighted(1, i)?)));
        }
    }
    Ok(())
}
