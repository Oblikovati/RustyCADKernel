//! The ExtremaPC requests; the counterpart of tools/oracle/tkgeombase_probe_extrema_pc.cxx
//! (same curve encoding and token order).
use super::common::{Reader, attempt, ax3, end, flag, int, pnt, pnt2d, real};
use super::pc2d::{mode, pc_result, vector};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorTransformedCurve};
use rustycad_tkgeombase::extrema_pc::{
    self as pc, Curve, DistanceFunction, Domain1D, ExtremaResult, GridEvaluator, SearchMode,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkmath::math::Vector;

fn bezier(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let rational = r.flag();
    let n = r.integer();
    let mut p = Array1::<Point3>::with_bounds(1, n).unwrap();
    let mut w = Array1::<f64>::with_bounds(1, n).unwrap();
    for i in 1..=n {
        p[i] = r.pnt();
        w[i] = r.real();
    }
    let curve = if rational {
        GeomBezierCurve::with_weights(&p, &w)?
    } else {
        GeomBezierCurve::new(&p)?
    };
    Ok(Handle::new(curve).cast())
}

fn bspline(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (rational, periodic) = (r.flag(), r.flag());
    let (degree, nk) = (r.integer(), r.integer());
    let mut k = Array1::<f64>::with_bounds(1, nk).unwrap();
    let mut m = Array1::<i32>::with_bounds(1, nk).unwrap();
    for i in 1..=nk {
        k[i] = r.real();
        m[i] = r.integer();
    }
    let np = r.integer();
    let mut p = Array1::<Point3>::with_bounds(1, np).unwrap();
    let mut w = Array1::<f64>::with_bounds(1, np).unwrap();
    for i in 1..=np {
        p[i] = r.pnt();
        w[i] = r.real();
    }
    let curve = if rational {
        GeomBSplineCurve::with_weights(&p, &w, &k, &m, degree, periodic)?
    } else {
        GeomBSplineCurve::new(&p, &k, &m, degree, periodic)?
    };
    Ok(Handle::new(curve).cast())
}

fn conic(kind: i32, r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let a = r.ax2();
    let x = r.real();
    if kind == 1 {
        return Ok(Handle::new(GeomCircle::new(a, x)?).cast());
    }
    if kind == 4 {
        return Ok(Handle::new(GeomParabola::new(a, x)?).cast());
    }
    let y = r.real();
    if kind == 2 {
        return Ok(Handle::new(GeomEllipse::new(a, x, y)?).cast());
    }
    Ok(Handle::new(GeomHyperbola::new(a, x, y)?).cast())
}

/// One curve specification, as gb_curve3d reads it.
pub fn curve3d(r: &mut Reader) -> Result<Handle<dyn GeomCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = r.pnt();
            Ok(Handle::new(GeomLine::from_point(p, r.dir())).cast())
        }
        1..=4 => conic(kind, r),
        5 => bezier(r),
        6 => bspline(r),
        7 => {
            let (u1, u2) = (r.real(), r.real());
            let (sense, adjust) = (r.flag(), r.flag());
            let basis = curve3d(r)?;
            Ok(Handle::new(GeomTrimmedCurve::new(&basis, u1, u2, sense, adjust)?).cast())
        }
        8 => {
            let d = r.real();
            let v = r.dir();
            let basis = curve3d(r)?;
            Ok(Handle::new(GeomOffsetCurve::new(&basis, d, v, false)?).cast())
        }
        _ => Err(Failure::with_type(
            "Standard_ProgramError",
            "unknown curve kind",
            "",
        )),
    }
}

/// gb_trsf3d.
fn trsf(r: &mut Reader) -> Result<Transform3, Failure> {
    let kind = r.integer();
    let p: Vec<f64> = (0..7).map(|_| r.real()).collect();
    let mut t = Transform3::default();
    if kind == 1 {
        t.set_translation(Vector3::new(p[0], p[1], p[2]));
    }
    if kind == 2 || kind == 4 {
        let axis = Axis1::new(
            Point3::new(p[0], p[1], p[2]),
            Direction3::new(p[3], p[4], p[5])?,
        );
        t.set_rotation(axis, p[6])?;
    }
    if kind == 3 {
        t.set_scale(Point3::new(p[0], p[1], p[2]), p[3])?;
    }
    if kind == 4 {
        let mut s = Transform3::default();
        s.set_scale(Point3::default(), p[0])?;
        t = t.multiplied(s);
    }
    Ok(t)
}

struct Query3d {
    point: Point3,
    tol: f64,
    mode: SearchMode,
    ends: bool,
}

fn queries(r: &mut Reader) -> Vec<Query3d> {
    let n = r.integer();
    (0..n)
        .map(|_| {
            let point = r.pnt();
            let tol = r.real();
            let mode = mode(r.integer());
            Query3d {
                point,
                tol,
                mode,
                ends: r.flag(),
            }
        })
        .collect()
}

fn guarded(body: impl FnOnce(&mut Reader) -> Result<(), Failure>, values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| body(&mut r));
    end();
}

fn run_queries(e: &mut Curve<'_>, r: &mut Reader) {
    flag(e.is_initialized());
    for q in queries(r) {
        attempt(|| {
            let result: &ExtremaResult = if q.ends {
                e.perform_with_endpoints(q.point, q.tol, q.mode)?
            } else {
                e.perform(q.point, q.tol, q.mode)?
            };
            pc_result(result, pnt);
            Ok(())
        });
    }
}

fn curve_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let ctor = r.integer();
    let t = if ctor >= 4 {
        trsf(r)?
    } else {
        Transform3::default()
    };
    let (mut u1, mut u2) = (0.0, 0.0);
    if ctor % 2 == 1 {
        u1 = r.real();
        u2 = r.real();
    }
    let adaptor = GeomAdaptorCurve::default();
    if ctor == 2 || ctor == 3 {
        adaptor.load(&c)?;
    }
    let transformed = if ctor >= 4 {
        Some(GeomAdaptorTransformedCurve::with_curve(&c, t)?)
    } else {
        None
    };
    let mut e: Curve = match (ctor, &transformed) {
        (0, _) => Curve::from_geom(&c)?,
        (1, _) => Curve::from_geom_range(&c, u1, u2)?,
        (2, _) => Curve::from_adaptor(&adaptor)?,
        (3, _) => Curve::from_adaptor_range(&adaptor, u1, u2)?,
        (4, Some(tc)) => Curve::from_transformed(tc)?,
        (_, Some(tc)) => Curve::from_transformed_range(tc, u1, u2)?,
        _ => return Err(Failure::new("probe: bad constructor")),
    };
    run_queries(&mut e, r);
    Ok(())
}
pub fn curve_case(values: &[f64]) {
    guarded(curve_body, values);
}

fn grid_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c)?;
    let v = r.reals();
    let mut params: Vector<'static> = Vector::with_size(v.len());
    for (i, x) in v.iter().enumerate() {
        params[i as i32] = *x;
    }
    let dmin = r.real();
    let d = Domain1D::new(dmin, r.real());
    let mut e = GridEvaluator::default();
    e.set_params(&params);
    for q in queries(r) {
        attempt(|| {
            pc_result(e.perform(&adaptor, &q.point, &d, q.tol, q.mode)?, pnt);
            Ok(())
        });
    }
    flag(GridEvaluator::is_closed_domain(&adaptor, &d));
    attempt(|| {
        vector(&GridEvaluator::build_curve_aware_params(&adaptor, &d)?);
        Ok(())
    });
    Ok(())
}
pub fn grid_case(values: &[f64]) {
    guarded(grid_body, values);
}

fn distance_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c)?;
    let p = r.pnt();
    let f = DistanceFunction::new(&adaptor, p);
    let u = r.reals();
    real(f.first_parameter()?);
    real(f.last_parameter()?);
    pnt(*f.point());
    for x in u {
        attempt(|| {
            real(f.value(x)?);
            let (v, dv) = f.values(x)?;
            real(v);
            real(dv);
            real(f.derivative(x)?);
            Ok(())
        });
    }
    Ok(())
}
pub fn distance_case(values: &[f64]) {
    guarded(distance_body, values);
}

struct AdaptorValue<'a>(&'a dyn Adaptor3dCurve);
impl pc::CurveEvaluator<Point3> for AdaptorValue<'_> {
    type Error = Failure;
    fn evaluate(&self, u: f64) -> Result<Point3, Failure> {
        self.0.value(u)
    }
}

fn endpoints_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c)?;
    let dmin = r.real();
    let d = Domain1D::new(dmin, r.real());
    let p = r.pnt();
    let m = mode(r.integer());
    let closed = r.flag();
    let mut result = ExtremaResult::default();
    pc::add_endpoint_extrema(&mut result, &p, &d, &AdaptorValue(&adaptor), m, closed)?;
    pc_result(&result, pnt);
    Ok(())
}
pub fn endpoints_case(values: &[f64]) {
    guarded(endpoints_body, values);
}

fn project_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let u = r.reals();
    let mut plane = Plane3::default();
    let mut c2: Handle<dyn Geom2dCurve> = Handle::default();
    let mut done = false;
    if let Some(line) = c.cast::<GeomLine>().get() {
        attempt(|| {
            plane = pc::line_plane(&line.lin())?;
            done = true;
            Ok(())
        });
    }
    let bezier: Handle<GeomBezierCurve> = c.cast();
    if !bezier.is_null() {
        attempt(|| {
            let mut x = Handle::default();
            done = pc::project_bezier(&bezier, &mut plane, &mut x)?;
            c2 = x.cast();
            Ok(())
        });
    }
    let bspline: Handle<GeomBSplineCurve> = c.cast();
    if !bspline.is_null() {
        attempt(|| {
            let mut x = Handle::default();
            done = pc::project_bspline(&bspline, &mut plane, &mut x)?;
            c2 = x.cast();
            Ok(())
        });
    }
    let offset: Handle<GeomOffsetCurve> = c.cast();
    if !offset.is_null() {
        attempt(|| {
            let mut x = Handle::default();
            done = pc::project_offset(&offset, &mut plane, &mut x)?;
            c2 = x.cast();
            Ok(())
        });
    }
    flag(done);
    ax3(plane.position());
    flag(c2.is_null());
    if let Some(curve) = c2.get() {
        for x in u {
            attempt(|| {
                pnt2d(curve.value(x)?);
                Ok(())
            });
        }
    }
    Ok(())
}
pub fn project_case(values: &[f64]) {
    guarded(project_body, values);
}

fn td3d_body(r: &mut Reader) -> Result<(), Failure> {
    let c = curve3d(r)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c)?;
    let ranged = r.flag();
    let range = if ranged {
        Some((r.real(), r.real()))
    } else {
        None
    };
    let criteria = [r.real(), r.real(), r.real(), r.real(), r.real()];
    attempt(|| {
        let (params, points) = rustycad_tkgeombase::probe_support::tangential_deflection_3d(
            &adaptor, range, criteria,
        )?;
        int(params.len() as i32);
        for (u, p) in params.iter().zip(&points) {
            real(*u);
            pnt(*p);
        }
        Ok(())
    });
    attempt(|| {
        let values = <rustycad_tkgeombase::extrema::CurveTool as rustycad_tkgeombase::extrema::ExtremaCurveTool>::defl_curv_intervals(&adaptor)?;
        int(values.len() as i32);
        values.iter().for_each(|x| real(*x));
        Ok(())
    });
    Ok(())
}
pub fn td3d_case(values: &[f64]) {
    guarded(td3d_body, values);
}
