//! The GeomConvert recognizer, least-squares and units requests; the counterpart of
//! tools/oracle/tkgeombase_probe_geomconvert_ana.cxx.
// The printers run inside `attempt`, whose closure returns `Ok(printer(...))`.
#![allow(clippy::unit_arg)]
use super::super::common::{Reader, attempt, circ, end, flag, int, lin, real};
use super::super::geom_tools::build;
use super::{UNSET, curve, curve2d, surface};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkgeombase::geom_convert::{
    AnaCurveFit, ConvType, CurveToAnaCurve, FuncConeLSDist, FuncCylinderLSDist, FuncSphereLSDist,
    HArrayOfXyz, SurfToAnaSurf, Units,
};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Circle3, Point3, Xyz};
use rustycad_tkmath::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};

const CURVE_TYPES: [CurveType; 9] = [
    CurveType::Line,
    CurveType::Circle,
    CurveType::Ellipse,
    CurveType::Hyperbola,
    CurveType::Parabola,
    CurveType::BezierCurve,
    CurveType::BSplineCurve,
    CurveType::OffsetCurve,
    CurveType::OtherCurve,
];
const SURFACE_TYPES: [SurfaceType; 11] = [
    SurfaceType::Plane,
    SurfaceType::Cylinder,
    SurfaceType::Cone,
    SurfaceType::Sphere,
    SurfaceType::Torus,
    SurfaceType::BezierSurface,
    SurfaceType::BSplineSurface,
    SurfaceType::SurfaceOfRevolution,
    SurfaceType::SurfaceOfExtrusion,
    SurfaceType::OffsetSurface,
    SurfaceType::OtherSurface,
];

fn unset_fit() -> AnaCurveFit {
    AnaCurveFit {
        first: UNSET,
        last: UNSET,
        deviation: UNSET,
    }
}

fn fit_values(fit: &AnaCurveFit) {
    real(fit.first);
    real(fit.last);
    real(fit.deviation);
}

/// gcv_ana_curve.
pub fn ana_curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let ct = ConvType::from_native(r.integer()).expect("probe conversion type");
    let target = CURVE_TYPES[r.integer() as usize];
    let (tol, f, l) = (r.real(), r.real(), r.real());
    attempt(|| {
        let c = build::curve(&mut r)?;
        let mut fit = unset_fit();
        let mut res: Handle<dyn GeomCurve> = Handle::default();
        match mode {
            0 => convert_curve(&c, (ct, target), (tol, f, l), (&mut res, &mut fit)),
            1 => attempt(|| {
                res = CurveToAnaCurve::compute_curve(&c, tol, (f, l), &mut fit, ct, target)?;
                Ok(())
            }),
            2 => attempt(|| {
                res = CurveToAnaCurve::compute_line(&c, tol, (f, l), &mut fit)?.cast();
                Ok(())
            }),
            3 => attempt(|| {
                res = CurveToAnaCurve::compute_circle(&c, tol, (f, l), &mut fit)?;
                Ok(())
            }),
            _ => attempt(|| {
                res = CurveToAnaCurve::compute_ellipse(&c, tol, (f, l), &mut fit)?;
                Ok(())
            }),
        }
        curve(&res);
        Ok(fit_values(&fit))
    });
    end();
}

// Mode 0 of gcv_ana_curve: the converter with its type and target, the conversion,
// its gap, then the gap after Init.
fn convert_curve(
    c: &Handle<dyn GeomCurve>,
    (ct, target): (ConvType, CurveType),
    (tol, f, l): (f64, f64, f64),
    (res, fit): (&mut Handle<dyn GeomCurve>, &mut AnaCurveFit),
) {
    let mut conv = CurveToAnaCurve::with_curve(c);
    conv.set_conv_type(ct);
    conv.set_target(target);
    int(conv.get_conv_type() as i32);
    int(conv.get_target() as i32);
    let mut range = (fit.first, fit.last);
    attempt(|| {
        Ok(flag(conv.convert_to_analytical(
            tol,
            res,
            (f, l),
            &mut range,
        )?))
    });
    (fit.first, fit.last) = range;
    real(conv.gap());
    conv.init(c);
    fit.deviation = conv.gap();
}

/// gcv_ana_empty.
pub fn ana_empty_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    let mut conv = CurveToAnaCurve::new();
    let mut res: Handle<dyn GeomCurve> = Handle::default();
    let mut fit = unset_fit();
    real(conv.gap());
    int(conv.get_conv_type() as i32);
    int(conv.get_target() as i32);
    let mut range = (fit.first, fit.last);
    attempt(|| {
        Ok(flag(conv.convert_to_analytical(
            tol,
            &mut res,
            (0.0, 1.0),
            &mut range,
        )?))
    });
    (fit.first, fit.last) = range;
    curve(&res);
    real(fit.first);
    real(fit.last);
    let (min_gap, line) = (ConvType::MinGap, CurveType::Line);
    attempt(|| {
        res = CurveToAnaCurve::compute_curve(&res, tol, (0.0, 1.0), &mut fit, min_gap, line)?;
        Ok(())
    });
    curve(&res);
    fit_values(&fit);
    attempt(|| {
        res = CurveToAnaCurve::compute_line(&res, tol, (0.5, 2.0), &mut fit)?.cast();
        Ok(())
    });
    curve(&res);
    fit_values(&fit);
    end();
}

/// gcv_is_linear.
pub fn is_linear_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (tol, n) = (r.real(), r.integer());
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, n.max(1)).unwrap();
    (1..=n).for_each(|i| p[i] = r.pnt());
    let points: Array1<'static, Point3> = if n == 0 { Array1::default() } else { p };
    let mut dev = UNSET;
    attempt(|| {
        let found = CurveToAnaCurve::is_linear(&points, tol)?;
        if let Some(d) = found {
            dev = d;
        }
        Ok(flag(found.is_some()))
    });
    real(dev);
    end();
}

/// gcv_get_line.
pub fn get_line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p1, p2) = (r.pnt(), r.pnt());
    let (mut cf, mut cl) = (UNSET, UNSET);
    attempt(|| {
        let (line, first, last) = CurveToAnaCurve::get_line(p1, p2)?;
        (cf, cl) = (first, last);
        Ok(lin(line))
    });
    real(cf);
    real(cl);
    end();
}

/// gcv_get_circle.
pub fn get_circle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (p0, p1, p2) = (r.pnt(), r.pnt(), r.pnt());
    let mut c = Circle3::default();
    attempt(|| Ok(flag(CurveToAnaCurve::get_circle(&mut c, p0, p1, p2)?)));
    circ(c);
    end();
}

/// gcv_ana_surf.
pub fn ana_surf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, ct) = (r.integer(), r.integer());
    let target = SURFACE_TYPES[r.integer() as usize];
    let (tol, u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real(), r.real());
    attempt(|| {
        let s = build::surface(&mut r)?;
        let mut conv = SurfToAnaSurf::with_surface(&s);
        real(conv.gap());
        if ct >= 0 {
            conv.set_conv_type(ConvType::from_native(ct).expect("probe conversion type"));
            conv.set_target(target);
        }
        let mut res: Handle<dyn GeomSurface> = Handle::default();
        attempt(|| {
            res = if mode == 0 {
                conv.convert_to_analytical(tol)?
            } else {
                conv.convert_to_analytical_in(tol, (u1, u2), (v1, v2))?
            };
            Ok(())
        });
        surface(&res);
        real(conv.gap());
        flag(SurfToAnaSurf::is_canonical(&s));
        Ok(flag(SurfToAnaSurf::is_canonical(&res)))
    });
    end();
}

/// gcv_ana_surf_empty.
pub fn ana_surf_empty_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    let mut conv = SurfToAnaSurf::new();
    real(conv.gap());
    attempt(|| {
        Ok(surface(&conv.convert_to_analytical_in(
            tol,
            (0.0, 1.0),
            (0.0, 1.0),
        )?))
    });
    flag(SurfToAnaSurf::is_canonical(&Handle::default()));
    end();
}

/// gcv_same_surf.
pub fn same_surf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    attempt(|| {
        let a = build::surface(&mut r)?;
        let b = build::surface(&mut r)?;
        Ok(flag(SurfToAnaSurf::is_same(&a, &b, tol)?))
    });
    end();
}

fn vector(v: &Vector<'_>) {
    (v.lower()..=v.upper()).for_each(|i| real(v[i]));
}

// A callback's result: the flag, or the type of the failure it raised.
fn outcome(ok: bool, failure: Result<(), Failure>) {
    match failure {
        Ok(()) => flag(ok),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

/// The three least-squares functions behind one interface for the probe.
trait LsFunction: MultipleVarFunction {
    fn take(&mut self) -> Result<(), Failure>;
}
impl LsFunction for FuncConeLSDist {
    fn take(&mut self) -> Result<(), Failure> {
        self.take_failure()
    }
}
impl LsFunction for FuncCylinderLSDist {
    fn take(&mut self) -> Result<(), Failure> {
        self.take_failure()
    }
}
impl LsFunction for FuncSphereLSDist {
    fn take(&mut self) -> Result<(), Failure> {
        self.take_failure()
    }
}

// NbVariables and Value.
fn ls_value(fun: &mut dyn LsFunction, x: &Vector<'_>, f: &mut f64) {
    int(fun.variables());
    let ok = fun.value(x, f);
    outcome(ok, fun.take());
    real(*f);
}

// Gradient and Values, each followed by the gradient.
fn ls_gradient<F: LsFunction + MultipleVarFunctionWithGradient>(
    fun: &mut F,
    x: &Vector<'_>,
    (f, g): (&mut f64, &mut Vector<'_>),
) {
    let ok = fun.gradient(x, g);
    outcome(ok, fun.take());
    vector(g);
    let ok = fun.values(x, f, g);
    outcome(ok, fun.take());
    real(*f);
    vector(g);
}

/// gcv_ls_dist.
pub fn ls_dist_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, setters, d, n) = (r.integer(), r.flag(), r.dir(), r.integer());
    let mut cloud: Array1<'static, Xyz> = Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| cloud[i] = r.xyz());
    let pts: HArrayOfXyz = Handle::new(HArray1::from_owned(cloud));
    let nx = r.integer();
    let mut x: Vector<'static> = Vector::with_bounds(1, nx).unwrap();
    (1..=nx).for_each(|i| x[i] = r.real());
    let mut g: Vector<'static> = Vector::filled(1, 4, 9.5).unwrap();
    let mut f = UNSET;
    match kind {
        0 => {
            let mut fun = FuncConeLSDist::new(&pts, d);
            if setters {
                fun = FuncConeLSDist::default();
                fun.set_points(&pts);
                fun.set_dir(d);
            }
            ls_value(&mut fun, &x, &mut f);
        }
        1 => {
            let mut fun = FuncCylinderLSDist::new(&pts, d);
            if setters {
                fun = FuncCylinderLSDist::default();
                fun.set_points(&pts);
                fun.set_dir(d);
            }
            ls_value(&mut fun, &x, &mut f);
            ls_gradient(&mut fun, &x, (&mut f, &mut g));
        }
        _ => {
            let mut fun = FuncSphereLSDist::new(&pts);
            if setters {
                fun = FuncSphereLSDist::default();
                fun.set_points(&pts);
            }
            ls_value(&mut fun, &x, &mut f);
            ls_gradient(&mut fun, &x, (&mut f, &mut g));
        }
    }
    end();
}

/// gcv_units.
pub fn units_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, lf, fr) = (r.integer(), r.real(), r.real());
    attempt(|| {
        let c = build::curve2d(&mut r)?;
        let s = build::surface(&mut r)?;
        let converted = match mode {
            0 => Units::radian_to_degree(&c, &s, lf, fr)?,
            1 => Units::degree_to_radian(&c, &s, lf, fr)?,
            _ => Units::mirror_pcurve(&c)?,
        };
        Ok(curve2d(&converted))
    });
    end();
}
