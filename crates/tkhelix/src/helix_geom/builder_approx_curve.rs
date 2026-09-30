// Rust translation of ModelingAlgorithms/TKHelix/HelixGeom/HelixGeom_BuilderApproxCurve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `HelixGeom_BuilderApproxCurve`: the abstract root of the helix builders, holding
//! the approximation settings, the status codes and the resulting curves
//! (docs/helix-design.md section 2).
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::geom_abs::Shape;

/// The protected data of `HelixGeom_BuilderApproxCurve`; its `Default` is the
/// protected constructor (tolerance 1e-4, C2, degree 8, 150 segments, reached
/// tolerance 99, no curve, both statuses 0).
#[derive(Clone)]
pub struct HelixGeomBuilderApproxCurveBase {
    /// `myErrorStatus`.
    pub error_status: i32,
    /// `myWarningStatus`.
    pub warning_status: i32,
    /// `myTolerance`.
    pub tolerance: f64,
    /// `myCont`.
    pub cont: Shape,
    /// `myMaxDegree`.
    pub max_degree: i32,
    /// `myMaxSeg`.
    pub max_seg: i32,
    /// `myTolReached`.
    pub tol_reached: f64,
    /// `myCurves`.
    pub curves: Sequence<Handle<dyn GeomCurve>>,
}

impl Default for HelixGeomBuilderApproxCurveBase {
    fn default() -> Self {
        Self {
            error_status: 0,
            warning_status: 0,
            tolerance: 0.0001,
            cont: Shape::C2,
            max_degree: 8,
            max_seg: 150,
            tol_reached: 99.0,
            curves: Sequence::default(),
        }
    }
}

/// `HelixGeom_BuilderApproxCurve`: the settings and results shared by the helix
/// builders; the provided methods are the native non-virtual members.
///
/// ```
/// use rustycad_tkhelix::helix_geom::{HelixGeomBuilderApproxCurve, HelixGeomBuilderHelixCoil};
/// let mut builder = HelixGeomBuilderHelixCoil::new();
/// builder.set_tolerance(1e-5);
/// builder.perform()?;
/// assert_eq!(builder.error_status(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait HelixGeomBuilderApproxCurve {
    /// The protected data; `builder.approx_curve_base().tolerance`.
    fn approx_curve_base(&self) -> &HelixGeomBuilderApproxCurveBase;
    /// The protected data, mutable; `builder.approx_curve_base_mut().max_seg = 10`.
    fn approx_curve_base_mut(&mut self) -> &mut HelixGeomBuilderApproxCurveBase;

    /// `Perform()`: builds the curves (pure virtual natively); `builder.perform()?`.
    fn perform(&mut self) -> Result<(), Failure>;

    /// `SetApproxParameters(aCont, aMaxDegree, aMaxSeg)`;
    /// `builder.set_approx_parameters(Shape::C1, 6, 50)`.
    fn set_approx_parameters(&mut self, cont: Shape, max_degree: i32, max_seg: i32) {
        let base = self.approx_curve_base_mut();
        (base.cont, base.max_degree, base.max_seg) = (cont, max_degree, max_seg);
    }
    /// `ApproxParameters(aCont, aMaxDegree, aMaxSeg)`: the three settings;
    /// `let (cont, degree, segments) = builder.approx_parameters();`.
    fn approx_parameters(&self) -> (Shape, i32, i32) {
        let base = self.approx_curve_base();
        (base.cont, base.max_degree, base.max_seg)
    }
    /// `SetTolerance(aTolerance)`; `builder.set_tolerance(1e-6)`.
    fn set_tolerance(&mut self, tolerance: f64) {
        self.approx_curve_base_mut().tolerance = tolerance;
    }
    /// `Tolerance()`; `builder.tolerance()`.
    fn tolerance(&self) -> f64 {
        self.approx_curve_base().tolerance
    }
    /// `ToleranceReached()`: the approximation error reached; `builder.tolerance_reached()`.
    fn tolerance_reached(&self) -> f64 {
        self.approx_curve_base().tol_reached
    }
    /// `Curves()`: the B-spline curves of the coils; `builder.curves().length()`.
    fn curves(&self) -> &Sequence<Handle<dyn GeomCurve>> {
        &self.approx_curve_base().curves
    }
    /// `ErrorStatus()`: 0 on success, 2 when an approximation failed; `builder.error_status()`.
    fn error_status(&self) -> i32 {
        self.approx_curve_base().error_status
    }
    /// `WarningStatus()`; `builder.warning_status()`.
    fn warning_status(&self) -> i32 {
        self.approx_curve_base().warning_status
    }
}
