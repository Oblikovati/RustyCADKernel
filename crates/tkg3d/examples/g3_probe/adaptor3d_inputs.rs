//! The inputs and the curve summary of tools/oracle/tkg3d_probe_adaptor3d.cxx: 2D
//! curves and adaptors, surface adaptors, and the Adaptor3d_Curve summary.
use super::common::{Reader, attempt, ax1, ax2, flag, int, pnt, real, show, vec};
use super::shapes;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dLine2d};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface, Adaptor3dSurfaceBase,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfLinearExtrusion,
};
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{Axis2d, Circle3, Direction2, Point2};

pub fn pnt2(r: &mut Reader) -> Point2 {
    let x = r.real();
    Point2::new(x, r.real())
}
pub fn dir2(r: &mut Reader) -> Direction2 {
    let x = r.real();
    Direction2::new(x, r.real()).expect("probe 2D direction")
}

/// One 2D curve specification (a3_curve2d).
pub fn curve2d(r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let kind = r.integer();
    match kind {
        0 => {
            let p = pnt2(r);
            Ok(Handle::new(Geom2dLine::from_point(p, dir2(r))).cast())
        }
        1..=4 => conic2d(kind, r),
        5 => bezier2d(r),
        6 => bspline2d(r),
        7 => {
            let (u1, u2) = (r.real(), r.real());
            let basis = curve2d(r)?;
            Ok(Handle::new(Geom2dTrimmedCurve::new(&basis, u1, u2, true, true)?).cast())
        }
        _ => panic!("2D curve kind {kind} has no builder"),
    }
}

fn conic2d(kind: i32, r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let p = pnt2(r);
    let axis = Axis2d::new(p, dir2(r));
    let (sense, x) = (r.flag(), r.real());
    match kind {
        1 => Ok(Handle::new(Geom2dCircle::from_axis(axis, x, sense)?).cast()),
        4 => Ok(Handle::new(Geom2dParabola::from_axis(axis, x, sense)?).cast()),
        2 => Ok(Handle::new(Geom2dEllipse::from_axis(axis, x, r.real(), sense)?).cast()),
        _ => Ok(Handle::new(Geom2dHyperbola::from_axis(axis, x, r.real(), sense)?).cast()),
    }
}

fn poles2d(r: &mut Reader, n: i32) -> (Array1<'static, Point2>, Array1<'static, f64>) {
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, n).unwrap();
    let mut weights: Array1<'static, f64> = Array1::with_bounds(1, n).unwrap();
    for i in 1..=n {
        poles.set_value(i, pnt2(r)).unwrap();
        weights.set_value(i, r.real()).unwrap();
    }
    (poles, weights)
}

fn bezier2d(r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (rational, n) = (r.flag(), r.integer());
    let (poles, weights) = poles2d(r, n);
    let curve = match rational {
        true => Geom2dBezierCurve::with_weights(&poles, &weights)?,
        false => Geom2dBezierCurve::new(&poles)?,
    };
    Ok(Handle::new(curve).cast())
}

fn bspline2d(r: &mut Reader) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (rational, periodic) = (r.flag(), r.flag());
    let (degree, nk) = (r.integer(), r.integer());
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, nk).unwrap();
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, nk).unwrap();
    for i in 1..=nk {
        knots.set_value(i, r.real()).unwrap();
        mults.set_value(i, r.integer()).unwrap();
    }
    let np = r.integer();
    let (poles, weights) = poles2d(r, np);
    let curve = match rational {
        true => {
            Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)?
        }
        false => Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, periodic)?,
    };
    Ok(Handle::new(curve).cast())
}

/// One 2D adaptor specification (a3_adaptor2d).
pub fn adaptor2d(r: &mut Reader) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    match r.integer() {
        0 => Ok(Handle::new(Geom2dAdaptorCurve::with_curve(&curve2d(r)?)?).cast()),
        1 => {
            let (u1, u2) = (r.real(), r.real());
            Ok(Handle::new(Geom2dAdaptorCurve::with_range(&curve2d(r)?, u1, u2)?).cast())
        }
        _ => {
            let (p, d) = (pnt2(r), dir2(r));
            let (u1, u2) = (r.real(), r.real());
            Ok(Handle::new(Adaptor2dLine2d::new(p, d, u1, u2)).cast())
        }
    }
}

/// One surface adaptor specification (a3_surface).
pub fn surface(r: &mut Reader) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
    match r.integer() {
        0 => Ok(Handle::new(GeomAdaptorSurface::with_surface(&shapes::surface(r)?)?).cast()),
        1 => {
            let b: Vec<f64> = (0..6).map(|_| r.real()).collect();
            let range = ((b[0], b[1], b[2], b[3]), (b[4], b[5]));
            Ok(Handle::new(GeomAdaptorSurface::with_range(&shapes::surface(r)?, range)?).cast())
        }
        2 => Ok(Handle::new(Adaptor3dSurfaceBase::default()).cast()),
        _ => {
            let d = r.dir();
            let curve: Handle<dyn Adaptor3dCurve> =
                Handle::new(GeomAdaptorCurve::with_curve(&shapes::curve(r)?)?).cast();
            Ok(Handle::new(
                GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&curve, d),
            )
            .cast())
        }
    }
}

/// Reads a 2D adaptor and a surface adaptor, reporting a construction failure.
pub fn inputs(
    r: &mut Reader,
) -> Option<(Handle<dyn Adaptor2dCurve2d>, Handle<dyn Adaptor3dSurface>)> {
    let built = adaptor2d(r).and_then(|curve| Ok((curve, surface(r)?)));
    match built {
        Ok(pair) => Some(pair),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

pub fn circ(c: Circle3) {
    ax2(c.position());
    real(c.radius());
}

fn shape_of(index: i32) -> Shape {
    [
        Shape::C0,
        Shape::G1,
        Shape::C1,
        Shape::G2,
        Shape::C2,
        Shape::C3,
        Shape::CN,
    ][index as usize]
}

/// NbIntervals and Intervals for every continuity (a3_curve_intervals).
pub fn curve_intervals(a: &dyn Adaptor3dCurve) {
    for s in 0..=6 {
        let n = match a.nb_intervals(shape_of(s)) {
            Ok(n) => n,
            Err(e) => {
                print!("{} ", e.exception_type());
                continue;
            }
        };
        int(n);
        let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).unwrap();
        attempt(|| {
            a.intervals(&mut t, shape_of(s))?;
            t.as_slice().iter().for_each(|v| real(*v));
            Ok(())
        });
    }
}

/// Whether a spline query would dereference a null 2D spline natively (a3_null_spline):
/// a curve on a plane whose 2D Geom2dAdaptor_Curve is of another kind.
fn null_spline(a: &dyn Adaptor3dCurve, kind: CurveType) -> bool {
    let transient: &dyn Transient = a;
    let Some(cos) = Adaptor3dCurveOnSurface::project_from(transient) else {
        return false;
    };
    let (curve, surface) = (cos.get_curve(), cos.get_surface());
    if surface.get_type().ok() != Some(SurfaceType::Plane) {
        return false;
    }
    curve.get_type().ok() != Some(kind) && curve.cast::<Adaptor2dLine2d>().is_null()
}

/// The kind queries (a3_curve_kinds).
pub fn curve_kinds(a: &dyn Adaptor3dCurve) {
    show(a.get_type(), |k| int(k as i32));
    show(a.line(), |l| ax1(l.position()));
    show(a.circle(), circ);
    show(a.ellipse(), |e| {
        ax2(e.position());
        real(e.major_radius());
        real(e.minor_radius());
    });
    show(a.hyperbola(), |h| {
        ax2(h.position());
        real(h.major_radius());
        real(h.minor_radius());
    });
    show(a.parabola(), |p| {
        ax2(p.position());
        real(p.focal());
    });
    show(a.degree(), int);
    show(a.is_rational(), flag);
    show(a.nb_poles(), int);
    show(a.nb_knots(), int);
    attempt(|| {
        if null_spline(a, CurveType::BezierCurve) {
            return Ok(());
        }
        let b = a.bezier()?;
        super::common::name(&b);
        if let Some(b) = b.get() {
            flag(b.is_rational());
            for i in 1..=b.nb_poles() {
                pnt(b.pole(i)?);
                real(b.weight(i)?);
            }
        }
        Ok(())
    });
    attempt(|| {
        if null_spline(a, CurveType::BSplineCurve) {
            return Ok(());
        }
        let b = a.bspline()?;
        super::common::name(&b);
        if let Some(b) = b.get() {
            use rustycad_tkg3d::geom::GeomCurve;
            flag(b.is_rational());
            flag(b.is_periodic());
            int(b.degree());
            for i in 1..=b.nb_knots() {
                real(b.knot(i)?);
                int(b.multiplicity(i)?);
            }
            for i in 1..=b.nb_poles() {
                pnt(b.pole(i)?);
                real(b.weight(i)?);
            }
        }
        Ok(())
    });
}

/// The evaluators at one parameter (a3_curve_evaluations).
pub fn curve_evaluations(a: &dyn Adaptor3dCurve, u: f64) {
    show(a.eval_d0(u), pnt);
    show(a.eval_d1(u), |e| {
        pnt(e.point);
        vec(e.d1);
    });
    show(a.eval_d2(u), |e| {
        pnt(e.point);
        vec(e.d1);
        vec(e.d2);
    });
    show(a.eval_d3(u), |e| {
        pnt(e.point);
        vec(e.d1);
        vec(e.d2);
        vec(e.d3);
    });
    for n in 0..=4 {
        show(a.eval_dn(u, n), vec);
    }
    show(a.value(u), pnt);
    show(a.d0(u), pnt);
    show(a.d1(u), |(p, v)| {
        pnt(p);
        vec(v);
    });
    show(a.d2(u), |(p, v1, v2)| {
        pnt(p);
        vec(v1);
        vec(v2);
    });
    show(a.d3(u), |(p, v1, v2, v3)| {
        pnt(p);
        vec(v1);
        vec(v2);
        vec(v3);
    });
    show(a.dn(u, 2), vec);
}

/// The curve-level summary (a3_curve_summary).
pub fn curve_summary(a: &dyn Adaptor3dCurve, params: &[f64]) {
    print!("{} ", a.dynamic_type().name());
    show(a.first_parameter(), real);
    show(a.last_parameter(), real);
    show(a.continuity(), |c| int(c as i32));
    curve_intervals(a);
    curve_intervals(a);
    show(a.is_closed(), flag);
    show(a.is_periodic(), flag);
    show(a.period(), real);
    for r in [1e-7, 1e-3, 0.5] {
        show(a.resolution(r), real);
    }
    curve_kinds(a);
    attempt(|| {
        curve_evaluations(a, a.first_parameter()?);
        Ok(())
    });
    attempt(|| {
        curve_evaluations(a, a.last_parameter()?);
        Ok(())
    });
    for u in params {
        curve_evaluations(a, *u);
    }
}
