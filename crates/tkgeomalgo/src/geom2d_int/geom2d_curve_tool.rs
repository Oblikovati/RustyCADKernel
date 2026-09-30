// Rust translation of Geom2dInt_Geom2dCurveTool.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dInt_Geom2dCurveTool`: the static access to an `Adaptor2d_Curve2d` used by
//! the intersection templates.
use crate::int_curve::PCurveTool;
use crate::int_imp_par_gen::ParTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dD3};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// The adaptor type of the Geom2dInt instantiations.
pub type Curve2d = dyn Adaptor2dCurve2d;

/// `Geom2dInt_Geom2dCurveTool`: static members forwarding to the adaptor, with the
/// sampling and continuity choices of the 2D intersections (C1 intervals, 1e-10 as
/// parametric tolerance, at most 300 samples). The adaptor's exceptions are returned
/// as `Err`; `Geom2dCurveTool::nb_samples(&adaptor)?`.
pub struct Geom2dCurveTool;

impl Geom2dCurveTool {
    /// `GetType(C)`. `Geom2dCurveTool::get_type(&adaptor)?`.
    pub fn get_type(c: &Curve2d) -> Result<CurveType, Failure> {
        c.get_type()
    }
    /// `Line(C)`. `Geom2dCurveTool::line(&adaptor)?`.
    pub fn line(c: &Curve2d) -> Result<Line2, Failure> {
        c.line()
    }
    /// `Circle(C)`. `Geom2dCurveTool::circle(&adaptor)?`.
    pub fn circle(c: &Curve2d) -> Result<Circle2, Failure> {
        c.circle()
    }
    /// `Ellipse(C)`. `Geom2dCurveTool::ellipse(&adaptor)?`.
    pub fn ellipse(c: &Curve2d) -> Result<Ellipse2, Failure> {
        c.ellipse()
    }
    /// `Parabola(C)`. `Geom2dCurveTool::parabola(&adaptor)?`.
    pub fn parabola(c: &Curve2d) -> Result<Parabola2, Failure> {
        c.parabola()
    }
    /// `Hyperbola(C)`. `Geom2dCurveTool::hyperbola(&adaptor)?`.
    pub fn hyperbola(c: &Curve2d) -> Result<Hyperbola2, Failure> {
        c.hyperbola()
    }
    /// `EpsX(C)`: the tolerance of the mathematical algorithms, 1e-10.
    /// `Geom2dCurveTool::eps_x(&adaptor)`.
    pub fn eps_x(_c: &Curve2d) -> f64 {
        1.0e-10
    }
    /// `EpsX(C, Eps_XYZ)`: the adaptor's parametric resolution of `eps_xyz`.
    /// `Geom2dCurveTool::eps_x_of(&adaptor, 1e-7)?`.
    pub fn eps_x_of(c: &Curve2d, eps_xyz: f64) -> Result<f64, Failure> {
        c.resolution(eps_xyz)
    }
    /// `NbSamples(C)`: the adaptor's samples, and on a circle of radius above 1
    /// enough for a deflection of 1% of the radius.
    /// `Geom2dCurveTool::nb_samples(&adaptor)?`.
    pub fn nb_samples(c: &Curve2d) -> Result<i32, Failure> {
        let mut nbs: i32 = c.nb_samples()?;
        if c.get_type()? == CurveType::Circle {
            nbs = circle_samples(c, || Ok(c.last_parameter()? - c.first_parameter()?), nbs)?;
        }
        Ok(nbs)
    }
    /// `NbSamples(C, U0, U1)`: on a B-spline `NbKnots * Degree` scaled to the part
    /// `[U0, U1]` of the range (at least `max(Degree + 1, 4)`), on a circle the
    /// deflection rule; at most 300. `Geom2dCurveTool::nb_samples_in(&adaptor, 0.0, 1.0)?`.
    pub fn nb_samples_in(c: &Curve2d, u0: f64, u1: f64) -> Result<i32, Failure> {
        let typ_c: CurveType = c.get_type()?;
        let mut nbs: i32 = c.nb_samples()?;
        if typ_c == CurveType::BSplineCurve {
            nbs = bspline_samples(c, (u0, u1), nbs)?;
        } else if typ_c == CurveType::Circle {
            nbs = circle_samples(c, || Ok((u1 - u0).abs()), nbs)?;
        }
        if nbs > 300 {
            nbs = 300;
        }
        Ok(nbs)
    }
    /// `FirstParameter(C)`. `Geom2dCurveTool::first_parameter(&adaptor)?`.
    pub fn first_parameter(c: &Curve2d) -> Result<f64, Failure> {
        c.first_parameter()
    }
    /// `LastParameter(C)`. `Geom2dCurveTool::last_parameter(&adaptor)?`.
    pub fn last_parameter(c: &Curve2d) -> Result<f64, Failure> {
        c.last_parameter()
    }
    /// `Value(C, X)`. `Geom2dCurveTool::value(&adaptor, 0.5)?`.
    pub fn value(c: &Curve2d, x: f64) -> Result<Point2, Failure> {
        c.value(x)
    }
    /// `D0(C, U, P)`. `Geom2dCurveTool::d0(&adaptor, 0.5)?`.
    pub fn d0(c: &Curve2d, u: f64) -> Result<Point2, Failure> {
        c.d0(u)
    }
    /// `D1(C, U, P, T)`. `Geom2dCurveTool::d1(&adaptor, 0.5)?`.
    pub fn d1(c: &Curve2d, u: f64) -> Result<(Point2, Vector2), Failure> {
        c.d1(u)
    }
    /// `D2(C, U, P, T, N)`. `Geom2dCurveTool::d2(&adaptor, 0.5)?`.
    pub fn d2(c: &Curve2d, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        c.d2(u)
    }
    /// `D3(C, U, P, T, N, V)`. `Geom2dCurveTool::d3(&adaptor, 0.5)?`.
    pub fn d3(c: &Curve2d, u: f64) -> Result<Adaptor2dD3, Failure> {
        c.d3(u)
    }
    /// `DN(C, U, N)`. `Geom2dCurveTool::dn(&adaptor, 0.5, 2)?`.
    pub fn dn(c: &Curve2d, u: f64, n: i32) -> Result<Vector2, Failure> {
        c.dn(u, n)
    }
    /// `NbIntervals(C)`: the C1 intervals (`IS_C2_COMPOSITE` is 0).
    /// `Geom2dCurveTool::nb_intervals(&adaptor)?`.
    pub fn nb_intervals(c: &Curve2d) -> Result<i32, Failure> {
        c.nb_intervals(Shape::C1)
    }
    /// `Intervals(C, Tab)`: the bounds of the C1 intervals.
    /// `Geom2dCurveTool::intervals(&adaptor, &mut tab)?`.
    pub fn intervals(c: &Curve2d, tab: &mut Array1<'_, f64>) -> Result<(), Failure> {
        c.intervals(tab, Shape::C1)
    }
    /// `GetInterval(C, Index, Tab, U1, U2)`: `Tab(Index)` and `Tab(Index + 1)`;
    /// Standard_OutOfRange outside the table.
    /// `let (a, b) = Geom2dCurveTool::get_interval(&adaptor, 1, &tab)?;`
    pub fn get_interval(
        _c: &Curve2d,
        i: i32,
        tab: &Array1<'_, f64>,
    ) -> Result<(f64, f64), Failure> {
        let at = |index: i32| -> Result<f64, Failure> {
            tab.value(index)
                .copied()
                .map_err(|e| e.into_failure("Geom2dInt_Geom2dCurveTool::GetInterval"))
        };
        Ok((at(i)?, at(i + 1)?))
    }
    /// `Degree(C)`. `Geom2dCurveTool::degree(&adaptor)?`.
    pub fn degree(c: &Curve2d) -> Result<i32, Failure> {
        c.degree()
    }
}

// The deflection rule of a circle of radius above 1: `|span| / 0.283079` samples
// (2 acos(1 - eps), eps = 0.01) when that is more; the span is read only then.
fn circle_samples(
    c: &Curve2d,
    span: impl FnOnce() -> Result<f64, Failure>,
    nbs: i32,
) -> Result<i32, Failure> {
    // Try to reach deflection = eps*R, eps = 0.01
    let min_r: f64 = 1.0;
    let r: f64 = c.circle()?.radius();
    if r <= min_r {
        return Ok(nbs);
    }
    let angl: f64 = 0.283079; // 2.*std::acos(1. - eps);
    let n: i32 = real_to_int(span()? / angl);
    Ok(if n < nbs { nbs } else { n })
}

// The B-spline rule of NbSamples(C, U0, U1).
fn bspline_samples(c: &Curve2d, (u0, u1): (f64, f64), nbs: i32) -> Result<i32, Failure> {
    let t: f64 = c.last_parameter()? - c.first_parameter()?;
    if t <= precision::p_confusion() {
        return Ok(nbs);
    }
    let mut t1: f64 = u1 - u0;
    if t1 < 0.0 {
        t1 = -t1;
    }
    let mut nbs: i32 = c.nb_knots()?;
    nbs *= c.degree()?;
    let anb: f64 = t1 / t * f64::from(nbs);
    // `(int)anb`: truncation toward zero.
    nbs = anb as i32;
    let a_min_pnt_nb: i32 = (c.degree()? + 1).max(4);
    if nbs < a_min_pnt_nb {
        nbs = a_min_pnt_nb;
    }
    Ok(nbs)
}

impl ParTool for Geom2dCurveTool {
    type Curve = Curve2d;
    fn eps_x(curve: &Curve2d) -> f64 {
        Geom2dCurveTool::eps_x(curve)
    }
    fn nb_samples(curve: &Curve2d, u0: f64, u1: f64) -> Result<i32, Failure> {
        Geom2dCurveTool::nb_samples_in(curve, u0, u1)
    }
    fn value(curve: &Curve2d, x: f64) -> Result<Point2, Failure> {
        curve.value(x)
    }
    fn d1(curve: &Curve2d, u: f64, p: &mut Point2, t: &mut Vector2) -> Result<(), Failure> {
        (*p, *t) = curve.d1(u)?;
        Ok(())
    }
    fn d2(
        curve: &Curve2d,
        u: f64,
        p: &mut Point2,
        t: &mut Vector2,
        n: &mut Vector2,
    ) -> Result<(), Failure> {
        (*p, *t, *n) = curve.d2(u)?;
        Ok(())
    }
}

impl PCurveTool for Geom2dCurveTool {
    fn get_type(curve: &Curve2d) -> Result<CurveType, Failure> {
        curve.get_type()
    }
    fn line(curve: &Curve2d) -> Result<Line2, Failure> {
        curve.line()
    }
    fn circle(curve: &Curve2d) -> Result<Circle2, Failure> {
        curve.circle()
    }
    fn ellipse(curve: &Curve2d) -> Result<Ellipse2, Failure> {
        curve.ellipse()
    }
    fn parabola(curve: &Curve2d) -> Result<Parabola2, Failure> {
        curve.parabola()
    }
    fn hyperbola(curve: &Curve2d) -> Result<Hyperbola2, Failure> {
        curve.hyperbola()
    }
    fn first_parameter(curve: &Curve2d) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Curve2d) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn d0(curve: &Curve2d, u: f64) -> Result<Point2, Failure> {
        curve.d0(u)
    }
    fn nb_intervals(curve: &Curve2d) -> Result<i32, Failure> {
        Geom2dCurveTool::nb_intervals(curve)
    }
    fn intervals(curve: &Curve2d, tab: &mut Array1<'_, f64>) -> Result<(), Failure> {
        Geom2dCurveTool::intervals(curve, tab)
    }
    fn get_interval(curve: &Curve2d, i: i32, tab: &Array1<'_, f64>) -> Result<(f64, f64), Failure> {
        Geom2dCurveTool::get_interval(curve, i, tab)
    }
}
