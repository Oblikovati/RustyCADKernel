//! HLRBRep_CurveTool.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Curve;
use crate::hlr_algo::c_int_truncation;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::int_curve::PCurveTool;
use rustycad_tkgeomalgo::int_imp_par_gen::ParTool;
use rustycad_tkgeombase::extrema::ExtremaCurveAccess;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// HLRBRep_CurveTool: the static access to a projected [`Curve`] that the 2D
/// intersection templates of HLRBRep (CInter and its helpers) and the Extrema
/// locators call (`IS_C2_COMPOSITE` is 0: C1 intervals); `CurveTool::nb_samples(&curve)?`.
pub struct CurveTool;

impl CurveTool {
    /// FirstParameter(C); `CurveTool::first_parameter(&curve)?`.
    pub fn first_parameter(c: &Curve) -> Result<f64, Failure> {
        c.first_parameter()
    }
    /// LastParameter(C); `CurveTool::last_parameter(&curve)?`.
    pub fn last_parameter(c: &Curve) -> Result<f64, Failure> {
        c.last_parameter()
    }
    /// Continuity(C); `CurveTool::continuity(&curve)?`.
    pub fn continuity(c: &Curve) -> Result<Shape, Failure> {
        c.continuity()
    }
    /// NbIntervals(C): the C1 intervals; `CurveTool::nb_intervals(&curve)?`.
    pub fn nb_intervals(c: &Curve) -> Result<i32, Failure> {
        c.nb_intervals(Shape::C1)
    }
    /// Intervals(C, T): the C1 interval bounds; `CurveTool::intervals(&curve, &mut tab)?`.
    pub fn intervals(c: &Curve, tab: &mut Array1<'_, f64>) -> Result<(), Failure> {
        c.intervals(tab, Shape::C1)
    }
    /// GetInterval(C, Index, Tab, U1, U2): `Tab(Index)` and `Tab(Index + 1)`
    /// (Standard_OutOfRange outside the table);
    /// `let (a, b) = CurveTool::get_interval(&curve, 1, &tab)?;`.
    pub fn get_interval(_c: &Curve, i: i32, tab: &Array1<'_, f64>) -> Result<(f64, f64), Failure> {
        let at = |index: i32| -> Result<f64, Failure> {
            tab.value(index)
                .copied()
                .map_err(|e| e.into_failure("HLRBRep_CurveTool::GetInterval"))
        };
        Ok((at(i)?, at(i + 1)?))
    }
    /// IsClosed(C); `CurveTool::is_closed(&curve)?`.
    pub fn is_closed(c: &Curve) -> Result<bool, Failure> {
        c.is_closed()
    }
    /// IsPeriodic(C); `CurveTool::is_periodic(&curve)?`.
    pub fn is_periodic(c: &Curve) -> Result<bool, Failure> {
        c.is_periodic()
    }
    /// Period(C); `CurveTool::period(&curve)?`.
    pub fn period(c: &Curve) -> Result<f64, Failure> {
        c.period()
    }
    /// Value(C, U); `CurveTool::value(&curve, 0.5)?`.
    pub fn value(c: &Curve, u: f64) -> Result<Point2, Failure> {
        c.value(u)
    }
    /// D0(C, U, P); `CurveTool::d0(&curve, 0.5, &mut p)?`.
    pub fn d0(c: &Curve, u: f64, p: &mut Point2) -> Result<(), Failure> {
        c.d0(u, p)
    }
    /// D1(C, U, P, T); `CurveTool::d1(&curve, 0.5, &mut p, &mut t)?`.
    pub fn d1(c: &Curve, u: f64, p: &mut Point2, t: &mut Vector2) -> Result<(), Failure> {
        c.d1(u, p, t)
    }
    /// D2(C, U, P, T, N); `CurveTool::d2(&curve, 0.5, &mut p, &mut t, &mut n)?`.
    pub fn d2(
        c: &Curve,
        u: f64,
        p: &mut Point2,
        t: &mut Vector2,
        n: &mut Vector2,
    ) -> Result<(), Failure> {
        c.d2(u, p, t, n)
    }
    /// D3(C, U, P, V1, V2, V3): empty natively, the outputs stay unchanged;
    /// `CurveTool::d3(&curve, 0.5, &mut p, &mut v1, &mut v2, &mut v3)`.
    pub fn d3(
        c: &Curve,
        u: f64,
        p: &mut Point2,
        v1: &mut Vector2,
        v2: &mut Vector2,
        v3: &mut Vector2,
    ) {
        c.d3(u, p, v1, v2, v3);
    }
    /// DN(C, U, N): a null vector natively; `CurveTool::dn(&curve, 0.5, 2)`.
    pub fn dn(c: &Curve, u: f64, n: i32) -> Vector2 {
        c.dn(u, n)
    }
    /// Resolution(C, R3d); `CurveTool::resolution(&curve, 1e-7)?`.
    pub fn resolution(c: &Curve, r3d: f64) -> Result<f64, Failure> {
        c.resolution(r3d)
    }
    /// GetType(C): the projected type; `CurveTool::get_type(&curve)`.
    pub fn get_type(c: &Curve) -> CurveType {
        c.get_type()
    }
    /// TheType(C): the projected type, as GetType; `CurveTool::the_type(&curve)`.
    pub fn the_type(c: &Curve) -> CurveType {
        c.get_type()
    }
    /// Line(C); `CurveTool::line(&curve)?`.
    pub fn line(c: &Curve) -> Result<Line2, Failure> {
        c.line()
    }
    /// Circle(C); `CurveTool::circle(&curve)?`.
    pub fn circle(c: &Curve) -> Result<Circle2, Failure> {
        c.circle()
    }
    /// Ellipse(C); `CurveTool::ellipse(&curve)?`.
    pub fn ellipse(c: &Curve) -> Result<Ellipse2, Failure> {
        c.ellipse()
    }
    /// Hyperbola(C): a default hyperbola natively; `CurveTool::hyperbola(&curve)`.
    pub fn hyperbola(c: &Curve) -> Hyperbola2 {
        c.hyperbola()
    }
    /// Parabola(C): a default parabola natively; `CurveTool::parabola(&curve)`.
    pub fn parabola(c: &Curve) -> Parabola2 {
        c.parabola()
    }
    /// Bezier(C): not implemented natively, a null handle; `CurveTool::bezier(&curve)`.
    pub fn bezier(_c: &Curve) -> Handle<Geom2dBezierCurve> {
        Handle::default()
    }
    /// BSpline(C): not implemented natively, a null handle; `CurveTool::bspline(&curve)`.
    pub fn bspline(_c: &Curve) -> Handle<Geom2dBSplineCurve> {
        Handle::default()
    }
    /// EpsX(C): 1e-10; `CurveTool::eps_x(&curve)`.
    pub fn eps_x(_c: &Curve) -> f64 {
        1e-10
    }
    /// NbSamples(C): 2 on a line, `3 + NbPoles` on a Bezier, `NbKnots * Degree` on a
    /// B-spline (at least 2), else 10; at most 50; `CurveTool::nb_samples(&curve)?`.
    pub fn nb_samples(c: &Curve) -> Result<i32, Failure> {
        let nbs = match c.get_type() {
            CurveType::BSplineCurve => {
                let nbs = f64::from(c.nb_knots()?) * f64::from(c.degree()?);
                at_least_two(nbs)
            }
            _ => common_samples(c)?,
        };
        Ok(capped(nbs))
    }
    /// NbSamples(C, U0, U1): as [`CurveTool::nb_samples`], but a B-spline counts the
    /// CN intervals of its Geom curve on `[u0, u1]` plus one, times its degree;
    /// `CurveTool::nb_samples_in(&curve, 0.0, 1.0)?`.
    pub fn nb_samples_in(c: &Curve, u1: f64, u2: f64) -> Result<i32, Failure> {
        let nbs = match c.get_type() {
            CurveType::BSplineCurve => {
                let geom = c.curve().transformed_curve().curve().curve();
                let ga_curve = GeomAdaptorCurve::with_range(&geom, u1, u2)?;
                let nbs = f64::from(ga_curve.nb_intervals(Shape::CN)? + 1) * f64::from(c.degree()?);
                at_least_two(nbs)
            }
            _ => common_samples(c)?,
        };
        Ok(capped(nbs))
    }
    /// Degree(C); `CurveTool::degree(&curve)?`.
    pub fn degree(c: &Curve) -> Result<i32, Failure> {
        c.degree()
    }
}

// The line, Bezier and other branches of both NbSamples (`nbsOther` is 10).
fn common_samples(c: &Curve) -> Result<f64, Failure> {
    Ok(match c.get_type() {
        CurveType::Line => 2.,
        CurveType::BezierCurve => f64::from(3 + c.nb_poles()?),
        _ => 10.,
    })
}

fn at_least_two(nbs: f64) -> f64 {
    if nbs < 2.0 { 2. } else { nbs }
}

// `if (nbs > 50) nbs = 50; return (int)nbs;`
fn capped(nbs: f64) -> i32 {
    c_int_truncation(if nbs > 50. { 50. } else { nbs })
}

impl ParTool for CurveTool {
    type Curve = Curve;
    fn eps_x(curve: &Curve) -> f64 {
        CurveTool::eps_x(curve)
    }
    fn nb_samples(curve: &Curve, u0: f64, u1: f64) -> Result<i32, Failure> {
        CurveTool::nb_samples_in(curve, u0, u1)
    }
    fn value(curve: &Curve, x: f64) -> Result<Point2, Failure> {
        curve.value(x)
    }
    fn d1(curve: &Curve, u: f64, p: &mut Point2, t: &mut Vector2) -> Result<(), Failure> {
        curve.d1(u, p, t)
    }
    fn d2(
        curve: &Curve,
        u: f64,
        p: &mut Point2,
        t: &mut Vector2,
        n: &mut Vector2,
    ) -> Result<(), Failure> {
        curve.d2(u, p, t, n)
    }
}

impl PCurveTool for CurveTool {
    fn get_type(curve: &Curve) -> Result<CurveType, Failure> {
        Ok(curve.get_type())
    }
    fn line(curve: &Curve) -> Result<Line2, Failure> {
        curve.line()
    }
    fn circle(curve: &Curve) -> Result<Circle2, Failure> {
        curve.circle()
    }
    fn ellipse(curve: &Curve) -> Result<Ellipse2, Failure> {
        curve.ellipse()
    }
    fn parabola(curve: &Curve) -> Result<Parabola2, Failure> {
        Ok(curve.parabola())
    }
    fn hyperbola(curve: &Curve) -> Result<Hyperbola2, Failure> {
        Ok(curve.hyperbola())
    }
    fn first_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn d0(curve: &Curve, u: f64) -> Result<Point2, Failure> {
        curve.value(u)
    }
    fn nb_intervals(curve: &Curve) -> Result<i32, Failure> {
        CurveTool::nb_intervals(curve)
    }
    fn intervals(curve: &Curve, tab: &mut Array1<'_, f64>) -> Result<(), Failure> {
        CurveTool::intervals(curve, tab)
    }
    fn get_interval(curve: &Curve, i: i32, tab: &Array1<'_, f64>) -> Result<(f64, f64), Failure> {
        CurveTool::get_interval(curve, i, tab)
    }
}

impl ExtremaCurveAccess for CurveTool {
    type Curve = Curve;
    type Point = Point2;
    type Vector = Vector2;
    fn first_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Curve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn get_type(curve: &Curve) -> Result<CurveType, Failure> {
        Ok(curve.get_type())
    }
    fn value(curve: &Curve, u: f64) -> Result<Point2, Failure> {
        curve.value(u)
    }
    fn d0(curve: &Curve, u: f64) -> Result<Point2, Failure> {
        curve.value(u)
    }
    fn d1(curve: &Curve, u: f64) -> Result<(Point2, Vector2), Failure> {
        let (mut p, mut v) = (Point2::default(), Vector2::default());
        curve.d1(u, &mut p, &mut v)?;
        Ok((p, v))
    }
    fn d2(curve: &Curve, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        let (mut p, mut v1, mut v2) = (Point2::default(), Vector2::default(), Vector2::default());
        curve.d2(u, &mut p, &mut v1, &mut v2)?;
        Ok((p, v1, v2))
    }
    fn dn(curve: &Curve, u: f64, n: i32) -> Result<Vector2, Failure> {
        Ok(curve.dn(u, n))
    }
}
