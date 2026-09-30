//! Geom2dGcc probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_geom2dgcc.cxx:
//! the readers, the qualified curve, the curve tool and the tangency functions (the solvers
//! are in geom2d_gcc_lines.rs and geom2d_gcc_circles.rs).
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show, xy};
use super::geom2d_hatch::{adaptor_out, hatch_adaptor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dPoint};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::gcc_ent::Position;
use rustycad_tkgeomalgo::geom2d_gcc::{
    self as gcc, CurveTool, FunctionTanCirCu, FunctionTanCuCu, FunctionTanCuCuCu,
    FunctionTanCuCuOnCu, FunctionTanCuPnt, FunctionTanObl, QCurve, QualifiedCurve,
};
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkmath::math::{FunctionSetWithDerivatives, FunctionWithDerivative, Matrix, Vector};

/// The derivative outputs of the three-argument functions, `(Point, Tan, D2)` each.
type Derivatives3 = [(Point2, Vector2, Vector2); 3];

impl Reader<'_> {
    /// `position curve u0 u1`: a Geom2dGcc_QualifiedCurve (also a Geom2dGcc_QCurve).
    pub(super) fn g2gcc_q(&mut self) -> Result<QualifiedCurve, Failure> {
        let p = Position::ALL[self.integer() as usize];
        QualifiedCurve::new(&hatch_adaptor(self)?, p)
    }
    /// `x y`: a Geom2d_CartesianPoint.
    pub(super) fn g2gcc_point(&mut self) -> Handle<dyn Geom2dPoint> {
        let x = self.real();
        Handle::new(Geom2dCartesianPoint::new(x, self.real())).cast()
    }
}

pub(super) fn v2d(v: Vector2) {
    xy(v.xy());
}

fn qflags(q: &QCurve) -> Result<(), Failure> {
    int(q.qualifier() as i32);
    flag(q.is_unqualified());
    flag(q.is_enclosing());
    flag(q.is_enclosed());
    flag(q.is_outside());
    let a: Geom2dAdaptorCurve = q.qualified()?;
    adaptor_out(&a);
    int(a.get_type()? as i32);
    Ok(())
}

/// `position curve u0 u1`: the four makers, then the qualified curve and the QCurve.
pub fn qualified_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = Position::ALL[r.integer() as usize];
        let a = hatch_adaptor(&mut r)?;
        int(gcc::unqualified(&a)?.qualifier() as i32);
        int(gcc::enclosing(&a)?.qualifier() as i32);
        int(gcc::enclosed(&a)?.qualifier() as i32);
        int(gcc::outside(&a)?.qualifier() as i32);
        qflags(&QualifiedCurve::new(&a, p)?)?;
        qflags(&QCurve::new(&a, p)?)
    });
    end();
}

/// `curve u0 u1 tol n u...`: every member of Geom2dGcc_CurveTool.
pub fn tool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = hatch_adaptor(&mut r)?;
        let tol = r.real();
        let us = r.reals();
        show(CurveTool::first_parameter(&a), real);
        show(CurveTool::last_parameter(&a), real);
        show(CurveTool::eps_x(&a, tol), real);
        int(CurveTool::nb_samples(&a));
        for u in us {
            show(CurveTool::value(&a, u), pnt2d);
            show(CurveTool::d1(&a, u), |(p, t)| {
                pnt2d(p);
                v2d(t);
            });
            show(CurveTool::d2(&a, u), |(p, t, n)| {
                pnt2d(p);
                v2d(t);
                v2d(n);
            });
            show(CurveTool::d3(&a, u), |(p, t, n, dn)| {
                pnt2d(p);
                v2d(t);
                v2d(n);
                v2d(dn);
            });
        }
        Ok(())
    });
    end();
}

// The scalar functions behind one probe operation.
trait Raised {
    fn raised(&self) -> Result<(), Failure>;
}
impl Raised for FunctionTanObl<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanObl::raised(self)
    }
}
impl Raised for FunctionTanCuPnt<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanCuPnt::raised(self)
    }
}
impl Raised for FunctionTanCirCu<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanCirCu::raised(self)
    }
}
impl Raised for FunctionTanCuCu<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanCuCu::raised(self)
    }
}
impl Raised for FunctionTanCuCuCu<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanCuCuCu::raised(self)
    }
}
impl Raised for FunctionTanCuCuOnCu<'_> {
    fn raised(&self) -> Result<(), Failure> {
        FunctionTanCuCuOnCu::raised(self)
    }
}

// A member call: its exception type (the native throw) or its outputs.
fn outcome<F: Raised>(f: &F, print: impl FnOnce()) {
    show(f.raised(), |()| print());
}

// Value, Derivative and Values of a scalar function at x, each on a fresh function.
fn scalar<'a, F>(make: impl Fn() -> Result<F, Failure>, x: f64)
where
    F: FunctionWithDerivative + Raised + 'a,
{
    attempt(|| {
        let mut f = make()?;
        let mut v = 0.0;
        let ok = f.value(x, &mut v);
        outcome(&f, || {
            flag(ok);
            real(v);
        });
        Ok(())
    });
    attempt(|| {
        let mut f = make()?;
        let mut d = 0.0;
        let ok = f.derivative(x, &mut d);
        outcome(&f, || {
            flag(ok);
            real(d);
        });
        Ok(())
    });
    attempt(|| {
        let mut f = make()?;
        let (mut v, mut d) = (0.0, 0.0);
        let ok = f.values(x, &mut v, &mut d);
        outcome(&f, || {
            flag(ok);
            real(v);
            real(d);
        });
        Ok(())
    });
}

/// `kind curve u0 u1 args n x...`: FunctionTanObl, FunctionTanCuPnt, FunctionTanCirCu.
pub fn func1_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let a = hatch_adaptor(&mut r)?;
        if kind == 0 {
            let d = r.dir2d();
            for x in r.reals() {
                scalar(|| Ok(FunctionTanObl::new(&a, d)), x);
            }
        } else if kind == 1 {
            let p = r.pnt2d();
            for x in r.reals() {
                scalar(|| Ok(FunctionTanCuPnt::new(&a, p)), x);
            }
        } else {
            let c = r.circ2d();
            for x in r.reals() {
                scalar(|| FunctionTanCirCu::new(c, &a), x);
            }
        }
        Ok(())
    });
    end();
}

fn vector_out(v: &Vector<'_>) {
    for i in v.lower()..=v.upper() {
        real(v[i]);
    }
}
fn matrix_out(m: &Matrix<'_>) {
    for i in m.lower_row()..=m.upper_row() {
        for j in m.lower_col()..=m.upper_col() {
            real(m[(i, j)]);
        }
    }
}

// NbVariables, NbEquations, then at each point InitDerivative, Value, Derivatives and
// Values into outputs filled with 7, each on a fresh function.
fn set_functions<F>(
    make: impl Fn() -> F,
    init: impl Fn(&F, &Vector<'_>) -> Result<(), Failure>,
    r: &mut Reader<'_>,
) where
    F: FunctionSetWithDerivatives + Raised,
{
    let f = make();
    int(f.variables());
    int(f.equations());
    let (n, nv) = (r.integer(), r.integer());
    for _ in 0..n {
        let mut x = Vector::with_bounds(1, nv).expect("probe vector");
        for i in 1..=nv {
            x[i] = r.real();
        }
        attempt(|| init(&make(), &x));
        attempt(|| {
            let mut f = make();
            let mut v = Vector::filled(1, nv, 7.0)?;
            let ok = f.value(&x, &mut v);
            outcome(&f, || {
                flag(ok);
                vector_out(&v);
            });
            Ok(())
        });
        attempt(|| {
            let mut f = make();
            let mut m = Matrix::filled(1, nv, 1, nv, 7.0).expect("probe matrix");
            let ok = f.derivatives(&x, &mut m);
            outcome(&f, || {
                flag(ok);
                matrix_out(&m);
            });
            Ok(())
        });
        attempt(|| {
            let mut f = make();
            let mut v = Vector::filled(1, nv, 7.0)?;
            let mut m = Matrix::filled(1, nv, 1, nv, 7.0).expect("probe matrix");
            let ok = f.values(&x, &mut v, &mut m);
            outcome(&f, || {
                flag(ok);
                vector_out(&v);
                matrix_out(&m);
            });
            Ok(())
        });
    }
}

/// `kind curve1 | circle, curve2, n nv x...`: FunctionTanCuCu.
pub fn func2_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let (a1, c1) = if kind == 0 {
            (Some(hatch_adaptor(&mut r)?), None)
        } else {
            (None, Some(r.circ2d()))
        };
        let a2 = hatch_adaptor(&mut r)?;
        let make = || match (&a1, c1) {
            (Some(a1), _) => FunctionTanCuCu::new(a1, &a2),
            (None, c1) => FunctionTanCuCu::with_circle(c1.expect("circle"), &a2),
        };
        let init = |f: &FunctionTanCuCu<'_>, x: &Vector<'_>| {
            let (p1, p2, t1, t2, d1, d2) = f.init_derivative(x)?;
            pnt2d(p1);
            pnt2d(p2);
            v2d(t1);
            v2d(t2);
            v2d(d1);
            v2d(d2);
            Ok(())
        };
        set_functions(make, init, &mut r);
        Ok(())
    });
    end();
}

// The nine InitDerivative outputs: the points, the tangents, the second derivatives.
fn init3(d: Derivatives3) {
    for (p, _, _) in d {
        pnt2d(p);
    }
    for (_, t, _) in d {
        v2d(t);
    }
    for (_, _, n) in d {
        v2d(n);
    }
}

/// `kind args... n nv x...`: FunctionTanCuCuCu.
pub fn func3_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let (mut a1, mut a2) = (None, None);
        let (mut c1, mut c2, mut l1, mut l2) = (None, None, None, None);
        match kind {
            0 => a1 = Some(hatch_adaptor(&mut r)?),
            1..=3 => c1 = Some(r.circ2d()),
            _ => l1 = Some(r.lin2d()),
        }
        match kind {
            0 | 1 | 5 => a2 = Some(hatch_adaptor(&mut r)?),
            2 => c2 = Some(r.circ2d()),
            _ => l2 = Some(r.lin2d()),
        }
        let a3 = hatch_adaptor(&mut r)?;
        let make = || match kind {
            0 => FunctionTanCuCuCu::new(a1.as_ref().unwrap(), a2.as_ref().unwrap(), &a3),
            1 => FunctionTanCuCuCu::circ_curve_curve(c1.unwrap(), a2.as_ref().unwrap(), &a3),
            2 => FunctionTanCuCuCu::circ_circ_curve(c1.unwrap(), c2.unwrap(), &a3),
            3 => FunctionTanCuCuCu::circ_lin_curve(c1.unwrap(), l2.unwrap(), &a3),
            4 => FunctionTanCuCuCu::lin_lin_curve(l1.unwrap(), l2.unwrap(), &a3),
            _ => FunctionTanCuCuCu::lin_curve_curve(l1.unwrap(), a2.as_ref().unwrap(), &a3),
        };
        let init = |f: &FunctionTanCuCuCu<'_>, x: &Vector<'_>| f.init_derivative(x).map(init3);
        set_functions(make, init, &mut r);
        Ok(())
    });
    end();
}

// The arguments of a FunctionTanCuCuOnCu request.
struct OnArgs {
    a1: Option<Geom2dAdaptorCurve>,
    c1: Option<rustycad_tkmath::gp::Circle2>,
    l1: Option<rustycad_tkmath::gp::Line2>,
    p2: Point2,
    a2: Option<Geom2dAdaptorCurve>,
    oc: Option<rustycad_tkmath::gp::Circle2>,
    ol: Option<rustycad_tkmath::gp::Line2>,
    on: Option<Geom2dAdaptorCurve>,
}

fn read_on_args(r: &mut Reader<'_>, kind: i32) -> Result<OnArgs, Failure> {
    let (first, locus) = (kind % 4, kind / 4);
    let mut args = OnArgs {
        a1: None,
        c1: None,
        l1: None,
        p2: Point2::default(),
        a2: None,
        oc: None,
        ol: None,
        on: None,
    };
    match first {
        0 | 3 => args.a1 = Some(hatch_adaptor(r)?),
        1 => args.c1 = Some(r.circ2d()),
        _ => args.l1 = Some(r.lin2d()),
    }
    if first == 3 {
        args.p2 = r.pnt2d();
    } else {
        args.a2 = Some(hatch_adaptor(r)?);
    }
    match locus {
        0 => args.oc = Some(r.circ2d()),
        1 => args.ol = Some(r.lin2d()),
        _ => args.on = Some(hatch_adaptor(r)?),
    }
    Ok(args)
}

fn make_on<'a>(a: &'a OnArgs, kind: i32, rad: f64) -> FunctionTanCuCuOnCu<'a> {
    let (a1, a2) = (a.a1.as_ref(), a.a2.as_ref());
    type F<'a> = FunctionTanCuCuOnCu<'a>;
    match kind {
        0 => F::curve_curve_on_circle(a1.unwrap(), a2.unwrap(), a.oc.unwrap(), rad),
        1 => F::circ_curve_on_circle(a.c1.unwrap(), a2.unwrap(), a.oc.unwrap(), rad),
        2 => F::lin_curve_on_circle(a.l1.unwrap(), a2.unwrap(), a.oc.unwrap(), rad),
        3 => F::curve_pnt_on_circle(a1.unwrap(), a.p2, a.oc.unwrap(), rad),
        4 => F::curve_curve_on_line(a1.unwrap(), a2.unwrap(), a.ol.unwrap(), rad),
        5 => F::circ_curve_on_line(a.c1.unwrap(), a2.unwrap(), a.ol.unwrap(), rad),
        6 => F::lin_curve_on_line(a.l1.unwrap(), a2.unwrap(), a.ol.unwrap(), rad),
        7 => F::curve_pnt_on_line(a1.unwrap(), a.p2, a.ol.unwrap(), rad),
        8 => F::curve_curve_on_curve(a1.unwrap(), a2.unwrap(), a.on.as_ref().unwrap(), rad),
        9 => F::circ_curve_on_curve(a.c1.unwrap(), a2.unwrap(), a.on.as_ref().unwrap(), rad),
        10 => F::lin_curve_on_curve(a.l1.unwrap(), a2.unwrap(), a.on.as_ref().unwrap(), rad),
        _ => F::curve_pnt_on_curve(a1.unwrap(), a.p2, a.on.as_ref().unwrap(), rad),
    }
}

/// `kind first second locus rad n nv x...`: FunctionTanCuCuOnCu.
pub fn func4_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let kind = r.integer();
        let args = read_on_args(&mut r, kind)?;
        let rad = r.real();
        let init = |f: &FunctionTanCuCuOnCu<'_>, x: &Vector<'_>| f.init_derivative(x).map(init3);
        set_functions(|| make_on(&args, kind, rad), init, &mut r);
        Ok(())
    });
    end();
}
