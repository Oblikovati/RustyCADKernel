// Rust translation of Blend_AppFunction.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The function of a blend approximation: the section equations along a guide line and
//! the shape of the section.
use super::BlendPoint;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::approx::{SweepD1, SweepD2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::{FunctionSetWithDerivatives, Vector};

/// The poles, 2d poles and weights of one section (the `Section(P, Poles, Poles2d,
/// Weigths)` output arrays).
pub struct SectionPoles<'a, 'b> {
    /// `Poles`.
    pub poles: &'a mut Array1<'b, Point3>,
    /// `Poles2d`.
    pub poles2d: &'a mut Array1<'b, Point2>,
    /// `Weigths`.
    pub weights: &'a mut Array1<'b, f64>,
}

/// `Blend_AppFunction`: a blend function seen by the walking and the approximation.
///
/// The vector `X` of `Value`, `Values` and `Derivatives` (the
/// [`FunctionSetWithDerivatives`] members) holds the parameters of the two extremities of
/// a section. A callback that meets a native exception answers false and keeps it;
/// [`BlendAppFunction::take_failure`] returns it after the solver. The other members
/// return the native exceptions as `Failure` values.
pub trait BlendAppFunction: FunctionSetWithDerivatives + Send {
    /// The exception kept by the last solver callbacks, if any (port only: the native
    /// exception unwinds through the solver); `function.take_failure()?`.
    fn take_failure(&mut self) -> Result<(), Failure>;
    /// `Set(Param)`: the parameter on the guide line, which fixes the plane of the section.
    fn set_param(&mut self, param: f64) -> Result<(), Failure>;
    /// `Set(First, Last)`: the bounds of the parametric interval on the guide line.
    fn set_interval(&mut self, first: f64, last: f64) -> Result<(), Failure>;
    /// `GetTolerance(Tolerance, Tol)`: the parametric tolerance of each variable for the
    /// 3d tolerance `tol`.
    fn get_tolerance(&self, tolerance: &mut Vector<'_>, tol: f64) -> Result<(), Failure>;
    /// `GetBounds(InfBound, SupBound)`: the bounds of each variable.
    fn get_bounds(&self, inf: &mut Vector<'_>, sup: &mut Vector<'_>) -> Result<(), Failure>;
    /// `IsSolution(Sol, Tol)`: whether `sol` is a zero at the current guide parameter.
    fn is_solution(&mut self, sol: &Vector<'_>, tol: f64) -> Result<bool, Failure>;
    /// `GetMinimalDistance()`: the minimal distance between the section extremities.
    fn get_minimal_distance(&self) -> Result<f64, Failure>;
    /// `Pnt1()`: the point on the first support.
    fn pnt1(&self) -> Point3;
    /// `Pnt2()`: the point on the second support.
    fn pnt2(&self) -> Point3;
    /// `IsRational()`: whether the section is rational.
    fn is_rational(&self) -> Result<bool, Failure>;
    /// `GetSectionSize()`: the length of the largest section.
    fn get_section_size(&self) -> Result<f64, Failure>;
    /// `GetMinimalWeight(Weigths)`: the minimal weight of each pole over all sections.
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure>;
    /// `NbIntervals(S)`: the number of intervals of continuity `shape`.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure>;
    /// `Intervals(T, S)`: the bounds of those intervals.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure>;
    /// `GetShape(NbPoles, NbKnots, Degree, NbPoles2d)`.
    fn get_shape(&mut self) -> Result<(i32, i32, i32, i32), Failure>;
    /// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d, Tol1D)`: the tolerance of each
    /// pole that the approximation must reach.
    fn get_tolerance_3d(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Vector<'_>,
        tol1d: &mut Vector<'_>,
    ) -> Result<(), Failure>;
    /// `Knots(TKnots)`: the knots of the section.
    fn knots(&mut self, knots: &mut Array1<'_, f64>) -> Result<(), Failure>;
    /// `Mults(TMults)`: the multiplicities of the section.
    fn mults(&mut self, mults: &mut Array1<'_, i32>) -> Result<(), Failure>;
    /// `Section(P, Poles, DPoles, Poles2d, DPoles2d, Weigths, DWeigths)`: the section at
    /// `point` with its first derivatives; false when they are not computed.
    fn section_d1(&mut self, point: &BlendPoint, values: SweepD1<'_, '_>) -> Result<bool, Failure>;
    /// `Section(P, Poles, Poles2d, Weigths)`: the section at `point`.
    fn section(&mut self, point: &BlendPoint, values: SectionPoles<'_, '_>) -> Result<(), Failure>;
    /// `Section(P, ..., D2Weigths)`: the section with its second derivatives; false when
    /// they are not computed.
    fn section_d2(&mut self, point: &BlendPoint, values: SweepD2<'_, '_>) -> Result<bool, Failure>;
    /// `Resolution(IC2d, Tol, TolU, TolV)`: the parametric tolerances of the 2d curve
    /// `index` for the 3d tolerance `tol`.
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure>;
    /// `Parameter(P)`: the parameter of `point`, imposed in the approximation;
    /// `function.parameter(&point)`.
    fn parameter(&self, point: &BlendPoint) -> f64 {
        point.parameter()
    }
}
