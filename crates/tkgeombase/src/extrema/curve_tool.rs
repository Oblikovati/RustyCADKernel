// Rust translation of Extrema_CurveTool.hxx/.cxx and Extrema_Curve2dTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_CurveTool` and `Extrema_Curve2dTool`: the static curve access of the
//! Extrema templates, and the [`ExtremaCurveTool`] trait that makes them (with the
//! point, vector and elementary types of their dimension) one template parameter.
use super::elementary_point_curve::ElementaryPointCurve;
use super::gp_space::{GpPoint, GpVector};
use crate::gcpnts::{PoleCurve, TCurveTypes, TangentialCriteria, TangentialDeflection};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use std::f64::consts::PI;

/// The knot access of `Geom_BSplineCurve` and `Geom2d_BSplineCurve` that
/// Extrema_GGExtPC reads.
pub trait SplineKnots: PoleCurve {
    /// `FirstUKnotIndex()`; `spline.first_knot_index()?`.
    fn first_knot_index(&self) -> Result<i32, Failure>;
    /// `LastUKnotIndex()`; `spline.last_knot_index()?`.
    fn last_knot_index(&self) -> Result<i32, Failure>;
    /// `Knots()`, the 1-based knot values; `spline.knot_values()`.
    fn knot_values(&self) -> Array1<'static, f64>;
}

macro_rules! spline_knots {
    ($($spline:ty),*) => {$(
        impl SplineKnots for $spline {
            fn first_knot_index(&self) -> Result<i32, Failure> {
                self.first_u_knot_index()
            }
            fn last_knot_index(&self) -> Result<i32, Failure> {
                self.last_u_knot_index()
            }
            fn knot_values(&self) -> Array1<'static, f64> {
                self.knots()
            }
        }
    )*};
}
spline_knots!(GeomBSplineCurve, Geom2dBSplineCurve);

/// A point with its first derivative.
pub type ToolD1<T> = (
    <T as ExtremaCurveTool>::Point,
    <T as ExtremaCurveTool>::Vector,
);
/// A point with its first two derivatives.
pub type ToolD2<T> = (
    <T as ExtremaCurveTool>::Point,
    <T as ExtremaCurveTool>::Vector,
    <T as ExtremaCurveTool>::Vector,
);
/// A point with its first three derivatives.
pub type ToolD3<T> = (
    <T as ExtremaCurveTool>::Point,
    <T as ExtremaCurveTool>::Vector,
    <T as ExtremaCurveTool>::Vector,
    <T as ExtremaCurveTool>::Vector,
);

/// The `TheCurve`/`TheCurveTool`/`ThePoint`/`TheVector`/`ThePOnC`/`TheExtPElC`
/// parameters of the Extrema templates, implemented by `CurveTool` (3D) and
/// `Curve2dTool` (2D). Each member is the static tool member of the same name;
/// an adaptor exception is the returned `Failure`;
/// `CurveTool::first_parameter(&adaptor)?`.
pub trait ExtremaCurveTool: 'static {
    /// The adaptor (`Adaptor3d_Curve` or `Adaptor2d_Curve2d`).
    type Curve: ?Sized + TCurveTypes<BSplineCurve = Self::BSpline, BezierCurve = Self::Bezier>;
    /// `gp_Pnt` or `gp_Pnt2d`.
    type Point: GpPoint<Vector = Self::Vector>;
    /// `gp_Vec` or `gp_Vec2d`.
    type Vector: GpVector;
    /// `gp_Lin` or `gp_Lin2d`.
    type Line: Copy;
    /// `gp_Circ` or `gp_Circ2d`.
    type Circle: Copy;
    /// `gp_Elips` or `gp_Elips2d`.
    type Ellipse: Copy;
    /// `gp_Hypr` or `gp_Hypr2d`.
    type Hyperbola: Copy;
    /// `gp_Parab` or `gp_Parab2d`.
    type Parabola: Copy;
    /// `Geom_BezierCurve` or `Geom2d_BezierCurve`.
    type Bezier: PoleCurve + RuntimeTyped;
    /// `Geom_BSplineCurve` or `Geom2d_BSplineCurve`.
    type BSpline: SplineKnots + RuntimeTyped;
    /// `Extrema_ExtPElC` or `Extrema_ExtPElC2d`.
    type ExtPElC: ElementaryPointCurve<Self> + Default;

    /// `FirstParameter(C)`.
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `LastParameter(C)`.
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `Continuity(C)`.
    fn continuity(curve: &Self::Curve) -> Result<Shape, Failure>;
    /// `NbIntervals(C, S)`.
    fn nb_intervals(curve: &Self::Curve, shape: Shape) -> Result<i32, Failure>;
    /// `Intervals(C, T, S)`.
    fn intervals(
        curve: &Self::Curve,
        bounds: &mut Array1<'_, f64>,
        shape: Shape,
    ) -> Result<(), Failure>;
    /// `IsClosed(C)` (a member of Extrema_Curve2dTool; the adaptor's for 3D).
    fn is_closed(curve: &Self::Curve) -> Result<bool, Failure>;
    /// `IsPeriodic(C)`.
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure>;
    /// `Period(C)`.
    fn period(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `Resolution(C, R3d)`.
    fn resolution(curve: &Self::Curve, r3d: f64) -> Result<f64, Failure>;
    /// `GetType(C)`.
    fn get_type(curve: &Self::Curve) -> Result<CurveType, Failure>;
    /// `Value(C, U)`.
    fn value(curve: &Self::Curve, u: f64) -> Result<Self::Point, Failure>;
    /// `D0(C, U, P)`.
    fn d0(curve: &Self::Curve, u: f64) -> Result<Self::Point, Failure>;
    /// `D1(C, U, P, V)`.
    fn d1(curve: &Self::Curve, u: f64) -> Result<ToolD1<Self>, Failure>;
    /// `D2(C, U, P, V1, V2)`.
    fn d2(curve: &Self::Curve, u: f64) -> Result<ToolD2<Self>, Failure>;
    /// `D3(C, U, P, V1, V2, V3)`.
    fn d3(curve: &Self::Curve, u: f64) -> Result<ToolD3<Self>, Failure>;
    /// `DN(C, U, N)`.
    fn dn(curve: &Self::Curve, u: f64, n: i32) -> Result<Self::Vector, Failure>;
    /// `Line(C)`.
    fn line(curve: &Self::Curve) -> Result<Self::Line, Failure>;
    /// `Circle(C)`.
    fn circle(curve: &Self::Curve) -> Result<Self::Circle, Failure>;
    /// `Ellipse(C)`.
    fn ellipse(curve: &Self::Curve) -> Result<Self::Ellipse, Failure>;
    /// `Hyperbola(C)`.
    fn hyperbola(curve: &Self::Curve) -> Result<Self::Hyperbola, Failure>;
    /// `Parabola(C)`.
    fn parabola(curve: &Self::Curve) -> Result<Self::Parabola, Failure>;
    /// `Degree(C)`.
    fn degree(curve: &Self::Curve) -> Result<i32, Failure>;
    /// `IsRational(C)`.
    fn is_rational(curve: &Self::Curve) -> Result<bool, Failure>;
    /// `NbPoles(C)`.
    fn nb_poles(curve: &Self::Curve) -> Result<i32, Failure>;
    /// `NbKnots(C)`.
    fn nb_knots(curve: &Self::Curve) -> Result<i32, Failure>;
    /// `Bezier(C)`.
    fn bezier(curve: &Self::Curve) -> Result<Handle<Self::Bezier>, Failure>;
    /// `BSpline(C)`.
    fn bspline(curve: &Self::Curve) -> Result<Handle<Self::BSpline>, Failure>;
    /// `DeflCurvIntervals(C)`: the parameters of a GCPnts_TangentialDeflection
    /// sampling sized from the curve length, the values of the 1-based native array.
    fn defl_curv_intervals(curve: &Self::Curve) -> Result<Vec<f64>, Failure> {
        let (tf, tl) = (Self::first_parameter(curve)?, Self::last_parameter(curve)?);
        let origin: Self::Point = Self::value(curve, tf)?;
        let mut length: f64 = 0.0;
        for i in 2..=23 {
            let t: f64 = (tf * f64::from(23 - i) + f64::from(i - 1) * tl) / f64::from(23 - 1);
            // The sum accumulates distances from the first point (not a polyline).
            length += origin.distance_to(&Self::value(curve, t)?);
        }
        deflection_parameters(curve, (tf, tl), length)
    }
}

// The deflection-sized sampling of `[tf, tl]`, or its two bounds for a short, slow,
// long-range or strongly deflected curve.
fn deflection_parameters<C: TCurveTypes + ?Sized>(
    curve: &C,
    (tf, tl): (f64, f64),
    length: f64,
) -> Result<Vec<f64>, Failure> {
    let (epsd, maxdefl, mindefl) = (1.0e-3, 1.0e3, 1.0e-3);
    let dl_dt: f64 = length / (tl - tf);
    // (tl - tf) > 10000: to avoid problem with GCPnts
    if length <= precision::confusion() || dl_dt < epsd || (tl - tf) > 10000.0 {
        return Ok(vec![tf, tl]);
    }
    let defl: f64 = std_max(0.01 * length / (2.0 * PI), mindefl);
    if defl > maxdefl {
        return Ok(vec![tf, tl]);
    }
    let criteria = TangentialCriteria {
        angular_deflection: PI / 6.0,
        curvature_deflection: defl,
        minimum_of_points: 2,
        u_tol: std_max(0.00001 * (tl - tf), precision::p_confusion()),
        min_len: std_max(0.00001 * length, precision::confusion()),
    };
    let generator = TangentialDeflection::with_curve(curve, criteria)?;
    (1..=generator.nb_points())
        .map(|i| generator.parameter(i))
        .collect()
}
