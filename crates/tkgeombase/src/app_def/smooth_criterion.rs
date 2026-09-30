// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_SmoothCriterion.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_SmoothCriterion`: the abstract criterion AppDef_Variational minimizes.
use crate::fem_tool::{AssemblyTable, Curve};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::{Array1, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkmath::math::{Matrix, Vector};

/// The quality of a smoothing (`QualityValues`): the ICDANA code (0 when the
/// estimations are right, 1 or 2 when they were corrected) and the tension (J1),
/// flexion (J2) and jerk (J3) criteria of the curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QualityValues {
    /// The native return value: 0, 1 or 2.
    pub icdana: i32,
    /// `J1`, `J2`, `J3`.
    pub criteria: [f64; 3],
}

/// The fit errors of a smoothing (`ErrorValues`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmoothErrors {
    /// `MaxError`: the largest point distance.
    pub max_error: f64,
    /// `QuadraticError`: the sum of the squared distances.
    pub quadratic_error: f64,
    /// `AverageError`: the sum of the distances.
    pub average_error: f64,
}

/// `AppDef_SmoothCriterion`: the criterion of the variational smoothing, shared
/// through handles, so every member takes `&self` (an implementation keeps its
/// state behind a lock). The native `double& EstLength()` is the pair
/// `est_length` / `set_est_length`; the output references are returned values.
///
/// ```
/// use rustycad_tkgeombase::app_def::{LinearCriteria, MultiLine, SmoothCriterion};
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point2;
/// let p = [Point2::new(0.0, 0.0), Point2::new(1.0, 0.5), Point2::new(2.0, 0.0)];
/// let criterion = LinearCriteria::new(&MultiLine::from_points2d(&Array1::from_shared(&p, 1)), 1, 3)?;
/// criterion.set_estimation(1.0, 2.0, 3.0);
/// assert_eq!(criterion.get_estimation(), [1.0, 2.0, 3.0]);
/// assert!(criterion.curve().is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait SmoothCriterion: Transient {
    /// `SetParameters(Parameters)`: the parameters of the points (shared, so later
    /// changes of the array are seen); invalidates the element cache.
    fn set_parameters(&self, parameters: &Handle<HArray1<f64>>);

    /// `SetCurve(C)`: the curve the criterion measures.
    fn set_curve(&self, curve: &Handle<Curve>) -> Result<(), Failure>;

    /// `GetCurve(C)`: the current curve (null before `SetCurve`).
    fn get_curve(&self) -> Handle<Curve>;

    /// `Curve()`: the non-virtual wrapper of `GetCurve`.
    fn curve(&self) -> Handle<Curve> {
        self.get_curve()
    }

    /// `SetEstimation(E1, E2, E3)`: the estimations of the three criteria.
    fn set_estimation(&self, e1: f64, e2: f64, e3: f64);

    /// `EstLength()` read: the estimated curve length.
    fn est_length(&self) -> f64;

    /// `EstLength() = value`: the native reference written.
    fn set_est_length(&self, length: f64);

    /// `GetEstimation(E1, E2, E3)`.
    fn get_estimation(&self) -> [f64; 3];

    /// `AssemblyTable()`: the global index of every element coefficient.
    fn assembly_table(&self) -> Result<AssemblyTable, Failure>;

    /// `DependenceTable()`: which dimension pairs are coupled.
    fn dependence_table(&self) -> Result<Handle<HArray2<i32>>, Failure>;

    /// `QualityValues(J1min, J2min, J3min, J1, J2, J3)`: the criteria of the curve,
    /// updating the estimations.
    fn quality_values(&self, j1min: f64, j2min: f64, j3min: f64) -> Result<QualityValues, Failure>;

    /// `ErrorValues(MaxError, QuadraticError, AverageError)`.
    fn error_values(&self) -> Result<SmoothErrors, Failure>;

    /// `Hessian(Element, Dimension1, Dimension2, H)`.
    fn hessian(
        &self,
        element: i32,
        dimension1: i32,
        dimension2: i32,
        h: &mut Matrix<'_>,
    ) -> Result<(), Failure>;

    /// `Gradient(Element, Dimension, G)`.
    fn gradient(&self, element: i32, dimension: i32, g: &mut Vector<'_>) -> Result<(), Failure>;

    /// `InputVector(X, AssTable)`: the solution vector written into the curve.
    fn input_vector(&self, x: &Vector<'_>, ass_table: &AssemblyTable) -> Result<(), Failure>;

    /// `SetWeight(QuadraticWeight, QualityWeight, percentJ1, percentJ2, percentJ3)`.
    fn set_weight(
        &self,
        quadratic_weight: f64,
        quality_weight: f64,
        percents: [f64; 3],
    ) -> Result<(), Failure>;

    /// `GetWeight(QuadraticWeight, QualityWeight)`.
    fn get_weight(&self) -> (f64, f64);

    /// `SetWeight(Weight)`: the weight of each point (the array's bounds replace
    /// the current ones, as the native assignment).
    fn set_point_weights(&self, weight: &Array1<'_, f64>);
}
impl_transient_interface!(
    dyn SmoothCriterion,
    "AppDef_SmoothCriterion",
    StandardTransient
);
