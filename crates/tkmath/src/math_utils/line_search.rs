//! MathUtils_LineSearch.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{GOLDEN_SECTION, Status, ZERO_TOL, sign_transfer};
use crate::math::{MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
mod armijo;
mod brent;
mod exact;
mod wolfe;
pub use armijo::armijo_backtrack;
pub use exact::exact_line_search;
pub use wolfe::wolfe_search;
/// Native default magnitude bound; `EXACT_LINE_SEARCH_ALPHA_MAX` is 10.
pub const EXACT_LINE_SEARCH_ALPHA_MAX: f64 = 10.0;
/// Keeps the native last or best evaluation on failure; `LineSearchResult::default()`.
#[derive(Debug, Clone)]
pub struct LineSearchResult {
    /// Detailed outcome; `result.status`.
    pub status: Status,
    /// Whether convergence conditions held; `result.is_valid`.
    pub is_valid: bool,
    /// Step associated with the stored value; `result.alpha`.
    pub alpha: f64,
    /// Last or best objective, depending on algorithm; `result.f_new`.
    pub f_new: f64,
    /// Value and gradient callback count; `result.evaluations`.
    pub evaluations: usize,
}
impl Default for LineSearchResult {
    fn default() -> Self {
        Self {
            status: Status::NotConverged,
            is_valid: false,
            alpha: 0.0,
            f_new: f64::NAN,
            evaluations: 0,
        }
    }
}
impl LineSearchResult {
    fn success(&mut self) {
        self.status = Status::Ok;
        self.is_valid = true;
    }
    fn evaluate(&mut self, f: &mut impl MultipleVarFunction, point: &Vector<'_>) -> Option<f64> {
        self.evaluations += 1;
        let mut value: f64 = 0.0;
        if !f.value(point, &mut value) {
            self.status = Status::CallbackError;
            return None;
        }
        Some(value)
    }
    fn finite(&mut self, value: f64) -> bool {
        if value.is_finite() {
            return true;
        }
        self.status = Status::NumericalError;
        false
    }
}
fn new_point(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    alpha: f64,
    output: &mut Vector<'_>,
) -> bool {
    for index in 0..point.length() as usize {
        let value: f64 = point.at(index).unwrap() + alpha * direction.at(index).unwrap();
        *output.change_at(index).unwrap() = value;
        if !value.is_finite() {
            return false;
        }
    }
    true
}
fn directional(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
) -> Result<f64, Status> {
    let mut derivative: f64 = 0.0;
    for index in 0..point.length() as usize {
        let (x, d, g): (f64, f64, f64) = (
            *point.at(index).unwrap(),
            *direction.at(index).unwrap(),
            *gradient.at(index).unwrap(),
        );
        if !x.is_finite() || !d.is_finite() || !g.is_finite() {
            return Err(Status::InvalidInput);
        }
        derivative += g * d;
    }
    if !derivative.is_finite() {
        return Err(Status::NumericalError);
    }
    if derivative >= 0.0 {
        return Err(Status::NonDescentDirection);
    }
    Ok(derivative)
}
fn common_valid(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    gradient: &Vector<'_>,
    fx: f64,
    alpha: f64,
    iterations: u32,
) -> bool {
    direction.length() == point.length()
        && gradient.length() == point.length()
        && fx.is_finite()
        && alpha.is_finite()
        && alpha > 0.0
        && iterations > 0
}
/// Safeguards the native quadratic interpolation step; `quadratic_interpolation(1.0,-2.0,1.0,2.0)`.
pub fn quadratic_interpolation(phi0: f64, derivative: f64, alpha: f64, phi: f64) -> f64 {
    let numerator: f64 = derivative * alpha * alpha;
    let denominator: f64 = 2.0 * (phi - phi0 - derivative * alpha);
    if denominator.abs() < ZERO_TOL {
        return 0.5 * alpha;
    }
    let candidate: f64 = -numerator / denominator;
    if candidate < 0.1 * alpha {
        return 0.1 * alpha;
    }
    if candidate > 0.9 * alpha {
        return 0.5 * alpha;
    }
    candidate
}

mod coordinate;
pub use coordinate::brent_along_coordinate;

pub(crate) use brent::BrentState;
