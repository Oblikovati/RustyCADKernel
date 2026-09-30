// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_TFunction.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_TFunction`: the integrand of the outer (T) integral of the
//! Gauss-Kronrod volume properties, itself the inner U integral.
use super::face::BRepGPropFace;
use super::ufunction::BRepGPropUFunction;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkgeombase::gprop::ValueType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{Function, KronrodSingleIntegration};

/// The minimal U range of an inner integral (`tolU`).
const TOL_U: f64 = 1.0e-9;

/// `BRepGProp_TFunction`: at a parameter of the boundary curve, the Gauss-Kronrod
/// integral of [`BRepGPropUFunction`] from `u_min` to the curve's U, times the curve's
/// V derivative (and the cone factor by point). An exception is kept and `value`
/// answers false (docs/topalgo-design.md section 9).
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_gprop::{BRepGPropFace, BRepGPropTFunction};
/// let face = BRepGPropFace::new(false);
/// let f = BRepGPropTFunction::new(&face, Point3::default(), true, &[0.0; 3], 0.0, 1e-3);
/// assert_eq!(f.error_reached(), 0.0);
/// ```
pub struct BRepGPropTFunction {
    surface: BRepGPropFace,
    u_function: BRepGPropUFunction,
    u_min: f64,
    tolerance: f64,
    tol_reached: f64,
    err_reached: f64,
    abs_error: f64,
    value_type: ValueType,
    is_by_point: bool,
    // Set by SetNbKronrodPoints and read by nothing (its use is commented out natively).
    #[allow(dead_code)]
    nb_pnt_outer: i32,
    failure: Option<Failure>,
}

impl BRepGPropTFunction {
    /// `BRepGProp_TFunction(theSurface, theVertex, IsByPoint, theCoeffs, theUMin,
    /// theTolerance)`: copies of the face tool (one for the U function), value type
    /// GProp_Unknown, three outer Kronrod points;
    /// `BRepGPropTFunction::new(&face, vertex, true, &[0.0; 3], u_min, 1e-4)`.
    pub fn new(
        surface: &BRepGPropFace,
        vertex: Point3,
        is_by_point: bool,
        coeffs: &[f64],
        u_min: f64,
        tolerance: f64,
    ) -> Self {
        let surface = surface.clone();
        let u_function = BRepGPropUFunction::new(&surface, vertex, is_by_point, coeffs);
        Self {
            surface,
            u_function,
            u_min,
            tolerance,
            tol_reached: 0.,
            err_reached: 0.,
            abs_error: 0.,
            value_type: ValueType::Unknown,
            is_by_point,
            nb_pnt_outer: 3,
            failure: None,
        }
    }

    /// `Init()`: resets the reached errors; `f.init()`.
    pub fn init(&mut self) {
        self.tol_reached = 0.;
        self.err_reached = 0.;
        self.abs_error = 0.;
    }

    /// `SetNbKronrodPoints(theNbPoints)`: the outer point count, made odd;
    /// `f.set_nb_kronrod_points(10)`.
    pub fn set_nb_kronrod_points(&mut self, nb_points: i32) {
        self.nb_pnt_outer = if nb_points % 2 == 0 {
            nb_points + 1
        } else {
            nb_points
        };
    }

    /// `SetValueType(aType)`: also the type of the U function; `f.set_value_type(t)`.
    pub fn set_value_type(&mut self, value_type: ValueType) {
        self.value_type = value_type;
        self.u_function.set_value_type(value_type);
    }

    /// `SetTolerance(aTol)`: the tolerance of the inner integrals; `f.set_tolerance(1e-4)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = tolerance;
    }

    /// `ErrorReached()`: the largest relative error since `init`; `f.error_reached()`.
    pub fn error_reached(&self) -> f64 {
        self.err_reached
    }

    /// `AbsolutError()`: the largest absolute error since `init`; `f.absolut_error()`.
    pub fn absolut_error(&self) -> f64 {
        self.abs_error
    }

    /// The exception a `value` call kept (and forgets it); `f.take_failure()`.
    pub fn take_failure(&mut self) -> Option<Failure> {
        self.failure.take()
    }

    /// The factor of the U integral: the V derivative of the curve, divided by 3 (mass),
    /// times 0.25 (centre) or 0.2 (inertia) by point; `None` for GProp_Unknown.
    fn coefficient(&self, dv: f64) -> Option<f64> {
        let by_point = self.is_by_point;
        Some(match self.value_type {
            ValueType::Unknown => return None,
            ValueType::Mass if by_point => dv / 3.,
            ValueType::CenterMassX | ValueType::CenterMassY | ValueType::CenterMassZ
                if by_point =>
            {
                dv * 0.25
            }
            ValueType::Mass
            | ValueType::CenterMassX
            | ValueType::CenterMassY
            | ValueType::CenterMassZ => dv,
            _ if by_point => dv * 0.2,
            _ => dv,
        })
    }

    /// The body of `Value(X, F)`: `None` where the native member returns false.
    fn integral(&mut self, x: f64) -> Result<Option<f64>, Failure> {
        let (p2d, v2d) = self.surface.d12d(x)?;
        let u_max = p2d.x();
        if u_max - self.u_min < TOL_U {
            return Ok(Some(0.));
        }
        let knots = self.surface.get_u_knots(self.u_min, u_max)?;
        self.u_function.set_v_param(p2d.y());
        let Some(coeff) = self.coefficient(v2d.y()) else {
            return Ok(None);
        };
        let abs_coeff = coeff.abs();
        if abs_coeff <= precision::angular() {
            return Ok(Some(0.));
        }
        let knots: Vec<f64> = knots.iter().copied().collect();
        let start = std::cmp::min(
            15,
            self.surface.u_integration_order()? / (knots.len() as i32 - 1) + 1,
        )
        .max(5);
        let Some((value, local_err)) = self.u_integrals(&knots, start)? else {
            return Ok(None);
        };
        Ok(Some(self.account(value * coeff, local_err * abs_coeff)))
    }

    /// The U integrals over the knot spans longer than 1e-9, with `start` Kronrod points
    /// and 1000 iterations at most; `None` when one is not done.
    fn u_integrals(&mut self, knots: &[f64], start: i32) -> Result<Option<(f64, f64)>, Failure> {
        let mut integral = KronrodSingleIntegration::default();
        let (mut value, mut local_err) = (0., 0.);
        for span in knots.windows(2) {
            if span[1] - span[0] < TOL_U {
                continue;
            }
            integral.perform_with_tolerance(
                &mut self.u_function,
                span[0],
                span[1],
                start,
                self.tolerance,
                1000,
            );
            if let Some(failure) = self.u_function.take_failure() {
                return Err(failure);
            }
            if !integral.is_done() {
                return Ok(None);
            }
            value += integral.value()?;
            local_err += integral.absolute_error()?;
        }
        Ok(Some((value, local_err)))
    }

    /// Records the errors of one value (`myAbsError`, `myTolReached`, `myErrReached`).
    fn account(&mut self, value: f64, mut local_err: f64) -> f64 {
        self.abs_error = std_max(self.abs_error, local_err);
        self.tol_reached += local_err;
        if value.abs() > epsilon(1.) {
            local_err /= value.abs();
        }
        self.err_reached = std_max(self.err_reached, local_err);
        value
    }
}

impl Function for BRepGPropTFunction {
    /// `Value(X, F)`: the U integral at the curve parameter `x`; false for GProp_Unknown,
    /// an integral not done, or a failure (kept).
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        match self.integral(x) {
            Ok(Some(value)) => {
                *result = value;
                true
            }
            Ok(None) => false,
            Err(failure) => {
                self.failure = Some(failure);
                false
            }
        }
    }

    /// `GetStateNumber()`: 0 (the error bookkeeping it did is commented out natively).
    fn state_number(&mut self) -> i32 {
        0
    }
}
