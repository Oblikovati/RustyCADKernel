//! MathSys native behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{Matrix, Vector},
    math_utils::Status,
};
/// Native generic nonlinear solver result; `SystemResult::default()`.
#[derive(Default)]
pub struct SystemResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Completed iteration count; `result.iterations`.
    pub iterations: u32,
    /// Last solution when available; `result.solution`.
    pub solution: Option<Vector<'static>>,
    /// Squared residual norm if representable; `result.value`.
    pub value: Option<f64>,
    /// Final gradient when computed; `result.gradient`.
    pub gradient: Option<Vector<'static>>,
    /// Final Jacobian when computed; `result.jacobian`.
    pub jacobian: Option<Matrix<'static>>,
    /// Final Euclidean residual norm; `result.residual_norm`.
    pub residual_norm: f64,
    /// Last accepted displacement norm; `result.step_norm`.
    pub step_norm: f64,
    /// Equilibrated Jacobian rank; `result.rank`.
    pub rank: usize,
    /// Meets function tolerance; `result.is_root`.
    pub is_root: bool,
    /// Meets least-squares stationarity criterion; `result.is_stationary`.
    pub is_stationary: bool,
}
impl SystemResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&SystemResult> for bool {
    fn from(result: &SystemResult) -> bool {
        result.is_done()
    }
}
