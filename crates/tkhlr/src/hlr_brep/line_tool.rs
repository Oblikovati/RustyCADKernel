//! HLRBRep_LineTool.hxx/.lxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve};
use rustycad_tkgeomalgo::int_curve_surface::InterCurveTool;
use rustycad_tkgeomalgo::int_imp::{TheCurve3dTool, TheCurveTool};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// HLRBRep_LineTool: a `gp_Lin` (the sight line through a point of the view) as
/// the curve of the line-surface intersection templates: unbounded, one CN
/// interval, three samples; `LineTool::value(&line, 0.5)`.
pub struct LineTool;

impl LineTool {
    /// FirstParameter(C): RealFirst; `LineTool::first_parameter(&line)`.
    pub fn first_parameter(_c: &Line3) -> f64 {
        real_first()
    }
    /// LastParameter(C): RealLast; `LineTool::last_parameter(&line)`.
    pub fn last_parameter(_c: &Line3) -> f64 {
        real_last()
    }
    /// Continuity(C): CN; `LineTool::continuity(&line)`.
    pub fn continuity(_c: &Line3) -> Shape {
        Shape::CN
    }
    /// NbIntervals(C, S): 1; `LineTool::nb_intervals(&line, Shape::C2)`.
    pub fn nb_intervals(_c: &Line3, _s: Shape) -> i32 {
        1
    }
    /// Intervals(C, T, Sh): leaves the table unchanged, as natively;
    /// `LineTool::intervals(&line, &mut tab, Shape::C2)`.
    pub fn intervals(_c: &Line3, _t: &mut Array1<'_, f64>, _sh: Shape) {}
    /// IntervalFirst(C): RealFirst; `LineTool::interval_first(&line)`.
    pub fn interval_first(_c: &Line3) -> f64 {
        real_first()
    }
    /// IntervalLast(C): RealLast; `LineTool::interval_last(&line)`.
    pub fn interval_last(_c: &Line3) -> f64 {
        real_last()
    }
    /// IntervalContinuity(C): CN; `LineTool::interval_continuity(&line)`.
    pub fn interval_continuity(_c: &Line3) -> Shape {
        Shape::CN
    }
    /// IsClosed(C): false; `LineTool::is_closed(&line)`.
    pub fn is_closed(_c: &Line3) -> bool {
        false
    }
    /// IsPeriodic(C): false; `LineTool::is_periodic(&line)`.
    pub fn is_periodic(_c: &Line3) -> bool {
        false
    }
    /// Period(C): 0; `LineTool::period(&line)`.
    pub fn period(_c: &Line3) -> f64 {
        0.
    }
    /// Value(C, U): ElCLib's line point; `LineTool::value(&line, 0.5)`.
    pub fn value(c: &Line3, u: f64) -> Point3 {
        elclib::line_value(u, c.position())
    }
    /// D0(C, U, P); `LineTool::d0(&line, 0.5)`.
    pub fn d0(c: &Line3, u: f64) -> Point3 {
        elclib::line_value(u, c.position())
    }
    /// D1(C, U, P, V): the point and the unit direction; `LineTool::d1(&line, 0.5)`.
    pub fn d1(c: &Line3, u: f64) -> (Point3, Vector3) {
        elclib::line_d1(u, c.position())
    }
    /// D2(C, U, P, V1, V2): V2 null; `LineTool::d2(&line, 0.5)`.
    pub fn d2(c: &Line3, u: f64) -> (Point3, Vector3, Vector3) {
        let (p, v1) = elclib::line_d1(u, c.position());
        (p, v1, Vector3::new(0., 0., 0.))
    }
    /// D3(C, U, P, V1, V2, V3): V2 and V3 null; `LineTool::d3(&line, 0.5)`.
    pub fn d3(c: &Line3, u: f64) -> (Point3, Vector3, Vector3, Vector3) {
        let (p, v1) = elclib::line_d1(u, c.position());
        let zero = Vector3::new(0., 0., 0.);
        (p, v1, zero, zero)
    }
    /// DN(C, U, N): ElCLib's line derivative; `LineTool::dn(&line, 0.5, 1)`.
    pub fn dn(c: &Line3, u: f64, n: i32) -> Vector3 {
        elclib::line_dn(u, c.position(), n)
    }
    /// Resolution(C, R3d): `r3d` itself; `LineTool::resolution(&line, 1e-7)`.
    pub fn resolution(_c: &Line3, r3d: f64) -> f64 {
        r3d
    }
    /// GetType(C): Line; `LineTool::get_type(&line)`.
    pub fn get_type(_c: &Line3) -> CurveType {
        CurveType::Line
    }
    /// Line(C): the line itself; `LineTool::line(&line)`.
    pub fn line(c: &Line3) -> Line3 {
        *c
    }
    /// Circle(C): a default circle; `LineTool::circle(&line)`.
    pub fn circle(_c: &Line3) -> Circle3 {
        Circle3::default()
    }
    /// Ellipse(C): a default ellipse; `LineTool::ellipse(&line)`.
    pub fn ellipse(_c: &Line3) -> Ellipse3 {
        Ellipse3::default()
    }
    /// Hyperbola(C): a default hyperbola; `LineTool::hyperbola(&line)`.
    pub fn hyperbola(_c: &Line3) -> Hyperbola3 {
        Hyperbola3::default()
    }
    /// Parabola(C): a default parabola; `LineTool::parabola(&line)`.
    pub fn parabola(_c: &Line3) -> Parabola3 {
        Parabola3::default()
    }
    /// Bezier(C): a null handle; `LineTool::bezier(&line)`.
    pub fn bezier(_c: &Line3) -> Handle<GeomBezierCurve> {
        Handle::default()
    }
    /// BSpline(C): a null handle; `LineTool::bspline(&line)`.
    pub fn bspline(_c: &Line3) -> Handle<GeomBSplineCurve> {
        Handle::default()
    }
    /// Degree(C): 1; `LineTool::degree(&line)`.
    pub fn degree(_c: &Line3) -> i32 {
        1
    }
    /// NbPoles(C): 0; `LineTool::nb_poles(&line)`.
    pub fn nb_poles(_c: &Line3) -> i32 {
        0
    }
    /// Poles(C, TP): leaves the array unchanged; `LineTool::poles(&line, &mut tp)`.
    pub fn poles(_c: &Line3, _tp: &mut Array1<'_, Point3>) {}
    /// IsRational(C): false; `LineTool::is_rational(&line)`.
    pub fn is_rational(_c: &Line3) -> bool {
        false
    }
    /// PolesAndWeights(C, TP, TW): leaves both arrays unchanged;
    /// `LineTool::poles_and_weights(&line, &mut tp, &mut tw)`.
    pub fn poles_and_weights(_c: &Line3, _tp: &mut Array1<'_, Point3>, _tw: &mut Array1<'_, f64>) {}
    /// NbKnots(C): 0; `LineTool::nb_knots(&line)`.
    pub fn nb_knots(_c: &Line3) -> i32 {
        0
    }
    /// KnotsAndMultiplicities(C, TK, TM): leaves both arrays unchanged;
    /// `LineTool::knots_and_multiplicities(&line, &mut tk, &mut tm)`.
    pub fn knots_and_multiplicities(
        _c: &Line3,
        _tk: &mut Array1<'_, f64>,
        _tm: &mut Array1<'_, i32>,
    ) {
    }
    /// NbSamples(C, U0, U1): 3; `LineTool::nb_samples(&line, 0.0, 1.0)`.
    pub fn nb_samples(_c: &Line3, _u0: f64, _u1: f64) -> i32 {
        3
    }
    /// SamplePars(C, U0, U1, Defl, NbMin): the ends and their mean;
    /// `LineTool::sample_pars(&line, 0.0, 1.0, 0.1, 5)`.
    pub fn sample_pars(
        _c: &Line3,
        u0: f64,
        u1: f64,
        _defl: f64,
        _nb_min: i32,
    ) -> Handle<HArray1<f64>> {
        Handle::new(HArray1::from_owned(three_pars(u0, u1)))
    }
    /// SamplePars(C, U0, U1, Defl, NbMin, Pars): stores a new array in `pars`;
    /// `LineTool::sample_pars_into(&line, 0.0, 1.0, 0.1, 5, &mut pars)`.
    pub fn sample_pars_into(
        c: &Line3,
        u0: f64,
        u1: f64,
        defl: f64,
        nb_min: i32,
        pars: &mut Handle<HArray1<f64>>,
    ) {
        *pars = Self::sample_pars(c, u0, u1, defl, nb_min);
    }
}

// `U0, (U0 + U1) * 0.5, U1` from index 1.
fn three_pars(u0: f64, u1: f64) -> Array1<'static, f64> {
    // Bounds 1..=3 are always valid; the fallback is never taken.
    let mut pars = Array1::with_bounds(1, 3).unwrap_or_else(|_| Array1::with_size(3));
    pars[1] = u0;
    pars[2] = (u0 + u1) * 0.5;
    pars[3] = u1;
    pars
}

impl TheCurveTool for LineTool {
    type Curve = Line3;
    fn first_parameter(curve: &Line3) -> Result<f64, Failure> {
        Ok(LineTool::first_parameter(curve))
    }
    fn last_parameter(curve: &Line3) -> Result<f64, Failure> {
        Ok(LineTool::last_parameter(curve))
    }
    fn resolution(curve: &Line3, r3d: f64) -> Result<f64, Failure> {
        Ok(LineTool::resolution(curve, r3d))
    }
}

impl TheCurve3dTool for LineTool {
    fn value(curve: &Line3, w: f64) -> Result<Point3, Failure> {
        Ok(LineTool::value(curve, w))
    }
    fn d1(curve: &Line3, w: f64) -> Result<(Point3, Vector3), Failure> {
        Ok(LineTool::d1(curve, w))
    }
}

impl InterCurveTool for LineTool {
    fn get_type(curve: &Line3) -> Result<CurveType, Failure> {
        Ok(LineTool::get_type(curve))
    }
    fn line(curve: &Line3) -> Result<Line3, Failure> {
        Ok(*curve)
    }
    fn circle(curve: &Line3) -> Result<Circle3, Failure> {
        Ok(LineTool::circle(curve))
    }
    fn ellipse(curve: &Line3) -> Result<Ellipse3, Failure> {
        Ok(LineTool::ellipse(curve))
    }
    fn parabola(curve: &Line3) -> Result<Parabola3, Failure> {
        Ok(LineTool::parabola(curve))
    }
    fn hyperbola(curve: &Line3) -> Result<Hyperbola3, Failure> {
        Ok(LineTool::hyperbola(curve))
    }
    fn nb_intervals(curve: &Line3, shape: Shape) -> Result<i32, Failure> {
        Ok(LineTool::nb_intervals(curve, shape))
    }
    fn intervals(curve: &Line3, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        LineTool::intervals(curve, bounds, shape);
        Ok(())
    }
    fn is_periodic(curve: &Line3) -> Result<bool, Failure> {
        Ok(LineTool::is_periodic(curve))
    }
    fn period(curve: &Line3) -> Result<f64, Failure> {
        Ok(LineTool::period(curve))
    }
    fn d0(curve: &Line3, u: f64) -> Result<Point3, Failure> {
        Ok(LineTool::d0(curve, u))
    }
    fn nb_samples(curve: &Line3, u0: f64, u1: f64) -> Result<i32, Failure> {
        Ok(LineTool::nb_samples(curve, u0, u1))
    }
    fn sample_pars(
        _curve: &Line3,
        u0: f64,
        u1: f64,
        _defl: f64,
        _nb_min: i32,
    ) -> Result<Array1<'static, f64>, Failure> {
        Ok(three_pars(u0, u1))
    }
}
