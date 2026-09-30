// Rust translation of the `TheCurveTool`/`ThePCurveTool` parameter of the IntCurve curve
// templates (IntCurve_UserIntConicCurveGen.gxx, IntCurve_DistBetweenPCurvesGen.gxx,
// IntCurve_IntCurveCurveGen.gxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The curve tool of the IntCurve curve templates, and [`GpConic`], one gp conic of
//! any kind.
use crate::int_imp_par_gen::ParTool;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2};

/// The members of `TheCurveTool` (Geom2dInt_Geom2dCurveTool) that the IntCurve curve
/// templates call beyond those of IntImpParGen's [`ParTool`]; the curve's exceptions
/// are returned as `Err`. `T::get_type(&curve)?`.
pub trait PCurveTool: ParTool {
    /// `GetType(C)`.
    fn get_type(curve: &Self::Curve) -> Result<CurveType, Failure>;
    /// `Line(C)`.
    fn line(curve: &Self::Curve) -> Result<Line2, Failure>;
    /// `Circle(C)`.
    fn circle(curve: &Self::Curve) -> Result<Circle2, Failure>;
    /// `Ellipse(C)`.
    fn ellipse(curve: &Self::Curve) -> Result<Ellipse2, Failure>;
    /// `Parabola(C)`.
    fn parabola(curve: &Self::Curve) -> Result<Parabola2, Failure>;
    /// `Hyperbola(C)`.
    fn hyperbola(curve: &Self::Curve) -> Result<Hyperbola2, Failure>;
    /// `FirstParameter(C)`.
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `LastParameter(C)`.
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `D0(C, U, P)`.
    fn d0(curve: &Self::Curve, u: f64) -> Result<Point2, Failure>;
    /// `NbIntervals(C)`.
    fn nb_intervals(curve: &Self::Curve) -> Result<i32, Failure>;
    /// `Intervals(C, Tab)`.
    fn intervals(curve: &Self::Curve, tab: &mut Array1<'_, f64>) -> Result<(), Failure>;
    /// `GetInterval(C, Index, Tab, U1, U2)`.
    fn get_interval(
        curve: &Self::Curve,
        i: i32,
        tab: &Array1<'_, f64>,
    ) -> Result<(f64, f64), Failure>;
}

/// One gp conic of any kind: the first curve of IntCurve_UserIntConicCurveGen, or
/// the conic a curve tool returns for its type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GpConic {
    /// `gp_Lin2d`.
    Line(Line2),
    /// `gp_Circ2d`.
    Circle(Circle2),
    /// `gp_Elips2d`.
    Ellipse(Ellipse2),
    /// `gp_Parab2d`.
    Parabola(Parabola2),
    /// `gp_Hypr2d`.
    Hyperbola(Hyperbola2),
}

impl GpConic {
    /// The rank of the kind in the pair order of IntCurve_IntConicConic (line,
    /// circle, ellipse, parabola, hyperbola). `GpConic::Line(line).rank() == 0`.
    pub fn rank(&self) -> i32 {
        match self {
            GpConic::Line(_) => 0,
            GpConic::Circle(_) => 1,
            GpConic::Ellipse(_) => 2,
            GpConic::Parabola(_) => 3,
            GpConic::Hyperbola(_) => 4,
        }
    }
    /// The conic of a curve of conic type (`None` for any other type), read through
    /// the tool. `GpConic::of_curve::<Geom2dCurveTool>(&adaptor)?`.
    pub fn of_curve<T: PCurveTool>(curve: &T::Curve) -> Result<Option<GpConic>, Failure> {
        Ok(Some(match T::get_type(curve)? {
            CurveType::Line => GpConic::Line(T::line(curve)?),
            CurveType::Circle => GpConic::Circle(T::circle(curve)?),
            CurveType::Ellipse => GpConic::Ellipse(T::ellipse(curve)?),
            CurveType::Parabola => GpConic::Parabola(T::parabola(curve)?),
            CurveType::Hyperbola => GpConic::Hyperbola(T::hyperbola(curve)?),
            _ => return Ok(None),
        }))
    }
}
