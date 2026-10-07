// Rust translation of BRepBlend_HCurveTool.hxx/.lxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBlend_HCurveTool`: the static access of the blend walkings to the guide curves
//! (`Adaptor3d_Curve` handles), and their sample count.
use super::live_curve_of;
use crate::blend_func::package::x86_int;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dD3};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3};

/// `BRepBlend_HCurveTool`: every member forwards to the adaptor, except `NbSamples`. A
/// null curve gives Standard_NullObject (a null dereference natively).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkfillet::brep_blend::BRepBlendHCurveTool;
/// assert!(BRepBlendHCurveTool::first_parameter(&Handle::default()).is_err());
/// ```
pub struct BRepBlendHCurveTool;

type Curve = Handle<dyn Adaptor3dCurve>;

fn curve(c: &Curve) -> Result<&dyn Adaptor3dCurve, Failure> {
    live_curve_of(c, "BRepBlend_HCurveTool")
}

impl BRepBlendHCurveTool {
    /// `FirstParameter(C)`. `BRepBlendHCurveTool::first_parameter(&c)?`.
    pub fn first_parameter(c: &Curve) -> Result<f64, Failure> {
        curve(c)?.first_parameter()
    }
    /// `LastParameter(C)`. `BRepBlendHCurveTool::last_parameter(&c)?`.
    pub fn last_parameter(c: &Curve) -> Result<f64, Failure> {
        curve(c)?.last_parameter()
    }
    /// `Continuity(C)`. `BRepBlendHCurveTool::continuity(&c)?`.
    pub fn continuity(c: &Curve) -> Result<Shape, Failure> {
        curve(c)?.continuity()
    }
    /// `NbIntervals(C, S)`. `BRepBlendHCurveTool::nb_intervals(&c, Shape::C2)?`.
    pub fn nb_intervals(c: &Curve, s: Shape) -> Result<i32, Failure> {
        curve(c)?.nb_intervals(s)
    }
    /// `Intervals(C, T, S)`. `BRepBlendHCurveTool::intervals(&c, &mut t, Shape::C2)?`.
    pub fn intervals(c: &Curve, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        curve(c)?.intervals(t, s)
    }
    /// `IsClosed(C)`. `BRepBlendHCurveTool::is_closed(&c)?`.
    pub fn is_closed(c: &Curve) -> Result<bool, Failure> {
        curve(c)?.is_closed()
    }
    /// `IsPeriodic(C)`. `BRepBlendHCurveTool::is_periodic(&c)?`.
    pub fn is_periodic(c: &Curve) -> Result<bool, Failure> {
        curve(c)?.is_periodic()
    }
    /// `Period(C)`. `BRepBlendHCurveTool::period(&c)?`.
    pub fn period(c: &Curve) -> Result<f64, Failure> {
        curve(c)?.period()
    }
    /// `Value(C, U)`. `BRepBlendHCurveTool::value(&c, 0.5)?`.
    pub fn value(c: &Curve, u: f64) -> Result<Point3, Failure> {
        curve(c)?.value(u)
    }
    /// `D0(C, U, P)`. `BRepBlendHCurveTool::d0(&c, 0.5)?`.
    pub fn d0(c: &Curve, u: f64) -> Result<Point3, Failure> {
        curve(c)?.d0(u)
    }
    /// `D1(C, U, P, T)`. `BRepBlendHCurveTool::d1(&c, 0.5)?`.
    pub fn d1(c: &Curve, u: f64) -> Result<(Point3, Vector3), Failure> {
        curve(c)?.d1(u)
    }
    /// `D2(C, U, P, T, N)`. `BRepBlendHCurveTool::d2(&c, 0.5)?`.
    pub fn d2(c: &Curve, u: f64) -> Result<(Point3, Vector3, Vector3), Failure> {
        curve(c)?.d2(u)
    }
    /// `D3(C, U, P, V1, V2, V3)`. `BRepBlendHCurveTool::d3(&c, 0.5)?`.
    pub fn d3(c: &Curve, u: f64) -> Result<Adaptor3dD3, Failure> {
        curve(c)?.d3(u)
    }
    /// `DN(C, U, N)`. `BRepBlendHCurveTool::dn(&c, 0.5, 2)?`.
    pub fn dn(c: &Curve, u: f64, n: i32) -> Result<Vector3, Failure> {
        curve(c)?.dn(u, n)
    }
    /// `Resolution(C, R3d)`. `BRepBlendHCurveTool::resolution(&c, 1e-7)?`.
    pub fn resolution(c: &Curve, r3d: f64) -> Result<f64, Failure> {
        curve(c)?.resolution(r3d)
    }
    /// `GetType(C)`. `BRepBlendHCurveTool::get_type(&c)?`.
    pub fn get_type(c: &Curve) -> Result<CurveType, Failure> {
        curve(c)?.get_type()
    }
    /// `Line(C)`. `BRepBlendHCurveTool::line(&c)?`.
    pub fn line(c: &Curve) -> Result<Line3, Failure> {
        curve(c)?.line()
    }
    /// `Circle(C)`. `BRepBlendHCurveTool::circle(&c)?`.
    pub fn circle(c: &Curve) -> Result<Circle3, Failure> {
        curve(c)?.circle()
    }
    /// `Ellipse(C)`. `BRepBlendHCurveTool::ellipse(&c)?`.
    pub fn ellipse(c: &Curve) -> Result<Ellipse3, Failure> {
        curve(c)?.ellipse()
    }
    /// `Hyperbola(C)`. `BRepBlendHCurveTool::hyperbola(&c)?`.
    pub fn hyperbola(c: &Curve) -> Result<Hyperbola3, Failure> {
        curve(c)?.hyperbola()
    }
    /// `Parabola(C)`. `BRepBlendHCurveTool::parabola(&c)?`.
    pub fn parabola(c: &Curve) -> Result<Parabola3, Failure> {
        curve(c)?.parabola()
    }
    /// `Bezier(C)`. `BRepBlendHCurveTool::bezier(&c)?`.
    pub fn bezier(c: &Curve) -> Result<Handle<GeomBezierCurve>, Failure> {
        curve(c)?.bezier()
    }
    /// `BSpline(C)`. `BRepBlendHCurveTool::bspline(&c)?`.
    pub fn bspline(c: &Curve) -> Result<Handle<GeomBSplineCurve>, Failure> {
        curve(c)?.bspline()
    }

    /// `NbSamples(C, U0, U1)`: 2 on a line, `3 + NbPoles` on a Bezier curve, the knots
    /// times the degree scaled from the curve range to `[u0, u1]` on a B-spline (at least
    /// 2), 10 otherwise; at most 50. `BRepBlendHCurveTool::nb_samples(&c, 0.0, 1.0)?`.
    pub fn nb_samples(c: &Curve, u0: f64, u1: f64) -> Result<i32, Failure> {
        let curve = curve(c)?;
        let mut nbs = match curve.get_type()? {
            CurveType::Line => 2.0,
            CurveType::BezierCurve => f64::from(3 + live(&curve.bezier()?)?.nb_poles()),
            CurveType::BSplineCurve => bspline_samples(live(&curve.bspline()?)?, u0, u1),
            _ => 10.0,
        };
        if nbs > 50. {
            nbs = 50.;
        }
        Ok(x86_int(nbs))
    }
}

fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepBlend_HCurveTool::NbSamples: null curve",
        )
    })
}

/// The B-spline count: `NbKnots * Degree * (Last - First) / (U1 - U0)`, at least 2.
fn bspline_samples(bsc: &GeomBSplineCurve, u0: f64, u1: f64) -> f64 {
    let mut nbs = f64::from(bsc.nb_knots());
    nbs *= f64::from(bsc.degree());
    nbs *= bsc.last_parameter() - bsc.first_parameter();
    nbs /= u1 - u0;
    if nbs < 2.0 { 2.0 } else { nbs }
}
