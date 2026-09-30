// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_HCurve2dTool.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_HCurve2dTool`: the static access to the arcs (`Adaptor2d_Curve2d` handles)
//! of the intersection, and their sample count.
use super::h_inter_tool::arc_of;
use crate::int_imp::TheCurveTool;
use crate::int_start::TheArc;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dD3};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

/// `IntPatch_HCurve2dTool`: every member forwards to the adaptor, except `NbSamples`.
/// A null arc gives Standard_NullObject (a null dereference natively).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::int_patch::IntPatchHCurve2dTool;
/// assert!(IntPatchHCurve2dTool::first_parameter(&Handle::default()).is_err());
/// ```
pub struct IntPatchHCurve2dTool;

fn arc(c: &TheArc) -> Result<&dyn Adaptor2dCurve2d, Failure> {
    arc_of(c, "IntPatch_HCurve2dTool")
}

impl IntPatchHCurve2dTool {
    /// `FirstParameter(C)`. `IntPatchHCurve2dTool::first_parameter(&c)?`.
    pub fn first_parameter(c: &TheArc) -> Result<f64, Failure> {
        arc(c)?.first_parameter()
    }
    /// `LastParameter(C)`. `IntPatchHCurve2dTool::last_parameter(&c)?`.
    pub fn last_parameter(c: &TheArc) -> Result<f64, Failure> {
        arc(c)?.last_parameter()
    }
    /// `Continuity(C)`. `IntPatchHCurve2dTool::continuity(&c)?`.
    pub fn continuity(c: &TheArc) -> Result<Shape, Failure> {
        arc(c)?.continuity()
    }
    /// `NbIntervals(C, S)`. `IntPatchHCurve2dTool::nb_intervals(&c, Shape::C2)?`.
    pub fn nb_intervals(c: &TheArc, s: Shape) -> Result<i32, Failure> {
        arc(c)?.nb_intervals(s)
    }
    /// `Intervals(C, T, S)`. `IntPatchHCurve2dTool::intervals(&c, &mut t, Shape::C2)?`.
    pub fn intervals(c: &TheArc, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        arc(c)?.intervals(t, s)
    }
    /// `IsClosed(C)`. `IntPatchHCurve2dTool::is_closed(&c)?`.
    pub fn is_closed(c: &TheArc) -> Result<bool, Failure> {
        arc(c)?.is_closed()
    }
    /// `IsPeriodic(C)`. `IntPatchHCurve2dTool::is_periodic(&c)?`.
    pub fn is_periodic(c: &TheArc) -> Result<bool, Failure> {
        arc(c)?.is_periodic()
    }
    /// `Period(C)`. `IntPatchHCurve2dTool::period(&c)?`.
    pub fn period(c: &TheArc) -> Result<f64, Failure> {
        arc(c)?.period()
    }
    /// `Value(C, U)`. `IntPatchHCurve2dTool::value(&c, 0.5)?`.
    pub fn value(c: &TheArc, u: f64) -> Result<Point2, Failure> {
        arc(c)?.value(u)
    }
    /// `D0(C, U, P)`. `IntPatchHCurve2dTool::d0(&c, 0.5)?`.
    pub fn d0(c: &TheArc, u: f64) -> Result<Point2, Failure> {
        arc(c)?.d0(u)
    }
    /// `D1(C, U, P, T)`. `IntPatchHCurve2dTool::d1(&c, 0.5)?`.
    pub fn d1(c: &TheArc, u: f64) -> Result<(Point2, Vector2), Failure> {
        arc(c)?.d1(u)
    }
    /// `D2(C, U, P, T, N)`. `IntPatchHCurve2dTool::d2(&c, 0.5)?`.
    pub fn d2(c: &TheArc, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        arc(c)?.d2(u)
    }
    /// `D3(C, U, P, V1, V2, V3)`. `IntPatchHCurve2dTool::d3(&c, 0.5)?`.
    pub fn d3(c: &TheArc, u: f64) -> Result<Adaptor2dD3, Failure> {
        arc(c)?.d3(u)
    }
    /// `DN(C, U, N)`. `IntPatchHCurve2dTool::dn(&c, 0.5, 2)?`.
    pub fn dn(c: &TheArc, u: f64, n: i32) -> Result<Vector2, Failure> {
        arc(c)?.dn(u, n)
    }
    /// `Resolution(C, R3d)`. `IntPatchHCurve2dTool::resolution(&c, 1e-7)?`.
    pub fn resolution(c: &TheArc, r3d: f64) -> Result<f64, Failure> {
        arc(c)?.resolution(r3d)
    }
    /// `GetType(C)`. `IntPatchHCurve2dTool::get_type(&c)?`.
    pub fn get_type(c: &TheArc) -> Result<CurveType, Failure> {
        arc(c)?.get_type()
    }
    /// `Line(C)`. `IntPatchHCurve2dTool::line(&c)?`.
    pub fn line(c: &TheArc) -> Result<Line2, Failure> {
        arc(c)?.line()
    }
    /// `Circle(C)`. `IntPatchHCurve2dTool::circle(&c)?`.
    pub fn circle(c: &TheArc) -> Result<Circle2, Failure> {
        arc(c)?.circle()
    }
    /// `Ellipse(C)`. `IntPatchHCurve2dTool::ellipse(&c)?`.
    pub fn ellipse(c: &TheArc) -> Result<Ellipse2, Failure> {
        arc(c)?.ellipse()
    }
    /// `Hyperbola(C)`. `IntPatchHCurve2dTool::hyperbola(&c)?`.
    pub fn hyperbola(c: &TheArc) -> Result<Hyperbola2, Failure> {
        arc(c)?.hyperbola()
    }
    /// `Parabola(C)`. `IntPatchHCurve2dTool::parabola(&c)?`.
    pub fn parabola(c: &TheArc) -> Result<Parabola2, Failure> {
        arc(c)?.parabola()
    }
    /// `Bezier(C)`. `IntPatchHCurve2dTool::bezier(&c)?`.
    pub fn bezier(c: &TheArc) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        arc(c)?.bezier()
    }
    /// `BSpline(C)`. `IntPatchHCurve2dTool::bspline(&c)?`.
    pub fn bspline(c: &TheArc) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        arc(c)?.bspline()
    }

    /// `NbSamples(C, U0, U1)`: 2 on a line, `3 + NbPoles` on a Bezier curve, the knots
    /// times the degree scaled to `[u0, u1]` on a B-spline (at least 2), 25 otherwise;
    /// at most 50. `IntPatchHCurve2dTool::nb_samples(&c, 0.0, 1.0)?`.
    pub fn nb_samples(c: &TheArc, u0: f64, u1: f64) -> Result<i32, Failure> {
        let curve: &dyn Adaptor2dCurve2d = arc(c)?;
        let nbs: f64 = match curve.get_type()? {
            CurveType::Line => return Ok(2),
            CurveType::BezierCurve => f64::from(3 + curve.nb_poles()?),
            CurveType::BSplineCurve => {
                let mut nbs: f64 = f64::from(curve.nb_knots()?);
                nbs *= f64::from(curve.degree()?);
                nbs *= u1 - u0;
                nbs /= curve.last_parameter()? - curve.first_parameter()?;
                if nbs < 2.0 { 2.0 } else { nbs }
            }
            _ => return Ok(25),
        };
        if nbs > 50.0 {
            return Ok(50);
        }
        // `(int)nbs`: truncation toward zero.
        Ok(nbs as i32)
    }
}

/// `TheCurveTool` of `IntPatch_CurvIntSurf` (the IntImp_IntCS instance): the bounds and
/// the resolution of the arc.
impl TheCurveTool for IntPatchHCurve2dTool {
    type Curve = TheArc;
    fn first_parameter(curve: &TheArc) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::first_parameter(curve)
    }
    fn last_parameter(curve: &TheArc) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::last_parameter(curve)
    }
    fn resolution(curve: &TheArc, r3d: f64) -> Result<f64, Failure> {
        IntPatchHCurve2dTool::resolution(curve, r3d)
    }
}
