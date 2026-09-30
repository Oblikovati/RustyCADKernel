// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepLib/BRepLib_ValidateEdge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_ValidateEdge`: the largest distance between the 3d curve of an edge and one
//! of its curves on surface (docs/topalgo-design.md section 6).
use crate::brep_check::BRepCheck;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface;
use rustycad_tkmath::gp::Point3;

/// `BRepLib_ValidateEdge`: computes the largest distance between a reference 3d curve
/// and a curve on surface, at a finite number of points (the default) or exactly with
/// `GeomLib_CheckCurveOnSurface` (only for a same-parameter edge).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tktopalgo::brep_lib::BRepLibValidateEdge;
/// let mut validate = BRepLibValidateEdge::new(&Handle::default(), &Handle::default(), true);
/// validate.set_control_points_number(10);
/// assert!(!validate.is_exact_method() && !validate.is_done());
/// ```
pub struct BRepLibValidateEdge {
    my_reference_curve: Handle<dyn Adaptor3dCurve>,
    my_other_curve: Handle<Adaptor3dCurveOnSurface>,
    my_same_parameter: bool,
    my_control_points_number: i32,
    my_tolerance_for_checking: f64,
    my_calculated_distance: f64,
    my_exit_if_tolerance_exceeded: bool,
    my_is_done: bool,
    my_is_exact_method: bool,
    my_is_multi_thread: bool,
}

/// The running largest squared distance of `processApprox` and its early exit.
struct ApproxScan {
    max_square: f64,
    square_tolerance: f64,
    exit_if_exceeded: bool,
}

impl ApproxScan {
    /// Takes `square` into the largest one; true when the scan must stop there.
    fn take(&mut self, square: f64) -> bool {
        if square > self.max_square {
            self.max_square = square;
        }
        self.exit_if_exceeded && self.max_square > self.square_tolerance
    }
}

impl BRepLibValidateEdge {
    /// `BRepLib_ValidateEdge(theReferenceCurve, theOtherCurve, theSameParameter)`: 22
    /// control points, the approximate method, no early exit;
    /// `BRepLibValidateEdge::new(&curve3d, &curve_on_surface, true)`.
    pub fn new(
        reference_curve: &Handle<dyn Adaptor3dCurve>,
        other_curve: &Handle<Adaptor3dCurveOnSurface>,
        same_parameter: bool,
    ) -> Self {
        Self {
            my_reference_curve: reference_curve.clone(),
            my_other_curve: other_curve.clone(),
            my_same_parameter: same_parameter,
            my_control_points_number: 22,
            my_tolerance_for_checking: 0.0,
            my_calculated_distance: 0.0,
            my_exit_if_tolerance_exceeded: false,
            my_is_done: false,
            my_is_exact_method: false,
            my_is_multi_thread: false,
        }
    }

    /// `SetExactMethod(theIsExact)`: the exact method (used only for a same-parameter
    /// edge); `validate.set_exact_method(true)`.
    pub fn set_exact_method(&mut self, is_exact: bool) {
        self.my_is_exact_method = is_exact;
    }

    /// `IsExactMethod()`; `validate.is_exact_method()`.
    pub fn is_exact_method(&self) -> bool {
        self.my_is_exact_method
    }

    /// `SetParallel(theIsMultiThread)`; `validate.set_parallel(true)`.
    pub fn set_parallel(&mut self, is_multi_thread: bool) {
        self.my_is_multi_thread = is_multi_thread;
    }

    /// `IsParallel()`; `validate.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.my_is_multi_thread
    }

    /// `SetControlPointsNumber(theControlPointsNumber)`: the number of intervals of the
    /// approximate method (below 1 counts as 1); `validate.set_control_points_number(40)`.
    pub fn set_control_points_number(&mut self, control_points_number: i32) {
        self.my_control_points_number = control_points_number;
    }

    /// `SetExitIfToleranceExceeded(theToleranceForChecking)`: the approximate method
    /// stops at the first distance beyond the corrected tolerance;
    /// `validate.set_exit_if_tolerance_exceeded(1e-7)?`.
    pub fn set_exit_if_tolerance_exceeded(&mut self, tolerance: f64) -> Result<(), Failure> {
        self.my_exit_if_tolerance_exceeded = true;
        self.my_tolerance_for_checking = self.correct_tolerance(tolerance)?;
        Ok(())
    }

    /// `Process()`: the exact method for an exact same-parameter check, else the
    /// approximate one; `validate.process()?`.
    pub fn process(&mut self) -> Result<(), Failure> {
        if self.my_is_exact_method && self.my_same_parameter {
            self.process_exact();
            return Ok(());
        }
        self.process_approx()
    }

    /// `IsDone()`: the distance was found at every point; `validate.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `CheckTolerance(theToleranceToCheck)`: the corrected tolerance is above the
    /// computed distance; `validate.check_tolerance(1e-7)?`.
    pub fn check_tolerance(&self, tolerance: f64) -> Result<bool, Failure> {
        Ok(self.correct_tolerance(tolerance)? > self.my_calculated_distance)
    }

    /// `GetMaxDistance()`: the computed distance times 1.00001;
    /// `validate.get_max_distance()`.
    pub fn get_max_distance(&self) -> f64 {
        self.my_calculated_distance * 1.00001
    }

    /// `UpdateTolerance(theToleranceToUpdate)`: raises `tolerance` to the computed
    /// distance times 1.00001; `validate.update_tolerance(&mut tol)`.
    pub fn update_tolerance(&self, tolerance: &mut f64) {
        let corrected: f64 = self.my_calculated_distance * 1.00001;
        if corrected > *tolerance {
            *tolerance = corrected;
        }
    }

    /// `correctTolerance`: `tolerance` plus the larger precision of the reference
    /// curve and of the surface; Standard_NullObject for missing curves (a crash
    /// natively).
    fn correct_tolerance(&self, tolerance: f64) -> Result<f64, Failure> {
        let surface = self.other()?.get_surface();
        let curve_precision: f64 = BRepCheck::prec_curve(self.reference()?)?;
        let surface_precision: f64 = BRepCheck::prec_surface(&surface)?;
        let delta: f64 = if curve_precision > surface_precision {
            curve_precision
        } else {
            surface_precision
        };
        Ok(tolerance + delta)
    }

    fn reference(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.my_reference_curve
            .get()
            .ok_or_else(|| null_curve("the reference curve"))
    }

    fn other(&self) -> Result<&Adaptor3dCurveOnSurface, Failure> {
        self.my_other_curve
            .get()
            .ok_or_else(|| null_curve("the curve on surface"))
    }

    /// `processApprox`: the distances at the control parameters (the same parameter on
    /// both curves), or at the ends and by projection of each curve's points on the
    /// other (a different parametrization).
    fn process_approx(&mut self) -> Result<(), Failure> {
        self.my_is_done = true;
        let reference = self.reference()?;
        let other: &dyn Adaptor3dCurve = self.other()?;
        let (rf, rl) = (reference.first_parameter()?, reference.last_parameter()?);
        let (of, ol) = (other.first_parameter()?, other.last_parameter()?);
        let count: i32 = self.my_control_points_number.max(1);
        let is_projection: bool = !self.my_same_parameter
            || (of - rf).abs() > precision::p_confusion()
            || (ol - rl).abs() > precision::p_confusion();
        let mut scan = ApproxScan {
            max_square: 0.0,
            square_tolerance: self.my_tolerance_for_checking * self.my_tolerance_for_checking,
            exit_if_exceeded: self.my_exit_if_tolerance_exceeded,
        };
        let done: bool = if is_projection {
            approx_projected(reference, other, ((rf, rl), (of, ol)), count, &mut scan)?
        } else {
            approx_same_parameter(reference, other, (rf, rl), count, &mut scan)?;
            true
        };
        if !done {
            // A failed projection stops the scan and keeps the earlier distance.
            self.my_is_done = false;
            return Ok(());
        }
        self.my_calculated_distance = scan.max_square.sqrt();
        Ok(())
    }

    /// `processExact`: `GeomLib_CheckCurveOnSurface` over the whole reference curve.
    fn process_exact(&mut self) {
        let mut check =
            CheckCurveOnSurface::with_curve(&self.my_reference_curve, precision::p_confusion());
        check.set_parallel(self.my_is_multi_thread);
        check.perform(&self.my_other_curve);
        self.my_is_done = check.is_done();
        if self.my_is_done {
            self.my_calculated_distance = check.max_distance();
        }
    }
}

/// The control parameter `((n - i) * first + i * last) / n`, in the native order.
fn control_parameter(i: i32, count: i32, (first, last): (f64, f64)) -> f64 {
    (f64::from(count - i) * first + f64::from(i) * last) / f64::from(count)
}

/// The same-parameter scan: both curves at the `count + 1` control parameters, until
/// the scan stops.
fn approx_same_parameter(
    reference: &dyn Adaptor3dCurve,
    other: &dyn Adaptor3dCurve,
    range: (f64, f64),
    count: i32,
    scan: &mut ApproxScan,
) -> Result<(), Failure> {
    for i in 0..=count {
        let parameter: f64 = control_parameter(i, count, range);
        let reference_point: Point3 = reference.value(parameter)?;
        let other_point: Point3 = other.value(parameter)?;
        if scan.take(reference_point.square_distance(&other_point)) {
            return Ok(());
        }
    }
    Ok(())
}

/// The scan of different parametrizations: the ends, then each curve's control points
/// projected on the other curve from the matching parameter, until the scan stops.
/// False when a projection fails (the check is then not done).
fn approx_projected(
    reference: &dyn Adaptor3dCurve,
    other: &dyn Adaptor3dCurve,
    ((rf, rl), (of, ol)): ((f64, f64), (f64, f64)),
    count: i32,
    scan: &mut ApproxScan,
) -> Result<bool, Failure> {
    if scan.take(reference.value(rf)?.square_distance(&other.value(of)?)) {
        return Ok(true);
    }
    if scan.take(reference.value(rl)?.square_distance(&other.value(ol)?)) {
        return Ok(true);
    }
    let mut reference_extrema = LocateExtPC::new();
    let mut other_extrema = LocateExtPC::new();
    let reference_tol: f64 = reference.resolution(precision::confusion())?;
    reference_extrema.initialize(reference, (rf, rl), reference_tol)?;
    let other_tol: f64 = other.resolution(precision::confusion())?;
    other_extrema.initialize(other, (of, ol), other_tol)?;
    for i in 1..count {
        let reference_parameter: f64 = control_parameter(i, count, (rf, rl));
        let reference_point: Point3 = reference.value(reference_parameter)?;
        let other_parameter: f64 = control_parameter(i, count, (of, ol));
        let other_point: Point3 = other.value(other_parameter)?;
        reference_extrema.perform(other_point, reference_parameter)?;
        if !reference_extrema.is_done() {
            return Ok(false);
        }
        if scan.take(reference_extrema.square_distance()?) {
            return Ok(true);
        }
        other_extrema.perform(reference_point, other_parameter)?;
        if !other_extrema.is_done() {
            return Ok(false);
        }
        if scan.take(other_extrema.square_distance()?) {
            return Ok(true);
        }
    }
    Ok(true)
}

/// Standard_NullObject for a missing adaptor (a null dereference natively).
fn null_curve(what: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("BRepLib_ValidateEdge: {what} is a null handle; expected an adaptor"),
    )
}
