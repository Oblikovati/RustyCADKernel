//! Mixed equality/inequality Uzawa: projected dual ascent with step `1 / (|C|_F^2 sqrt 2)`.
use super::{Problem, UzawaConfig, UzawaResult, finish};
use crate::math::Vector;
use crate::math_opt::utils::zero_based;
use crate::math_utils::Status;

/// Native `std::max(a, b)`: `b` only when `a < b`, so a NaN `b` keeps `a`.
fn native_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

/// Primal/dual iterate of the native loop (`aErruza`, `aVardua`).
struct DualState {
    shift: Vector<'static>,
    dual: Vector<'static>,
}

pub(super) fn solve(
    problem: &Problem<'_, '_, '_>,
    initial_error: Vector<'static>,
    config: &UzawaConfig,
) -> UzawaResult {
    let mut result: UzawaResult = UzawaResult {
        initial_error: Some(zero_based(&initial_error)),
        ..Default::default()
    };
    let rho: f64 = match step_size(problem) {
        Ok(rho) => rho,
        Err(status) => {
            result.status = status;
            return result;
        }
    };
    let mut state: DualState = DualState::new(problem);
    for iteration in 0..config.max_iterations {
        result.iterations = iteration + 1;
        let primal_change: f64 = state.update_primal(problem, iteration > 0);
        let dual_change: f64 = state.update_dual(problem, &initial_error, rho);
        // Primal and dual changes are independent convergence requirements.
        if iteration > 0 && primal_change <= config.eps_lix && dual_change <= config.eps_lic {
            let solution: Vector<'static> = problem.solution(&state.shift);
            finish(&mut result, solution, state.dual, state.shift);
            return result;
        }
    }
    result.solution = Some(problem.solution(&state.shift));
    result.dual = Some(state.dual);
    result.error = Some(state.shift);
    result.status = Status::MaxIterations;
    result
}

/// `1 / norm / norm / sqrt(2)` with the Frobenius norm accumulated by `hypot` row by row.
fn step_size(problem: &Problem<'_, '_, '_>) -> Result<f64, Status> {
    let mut norm: f64 = 0.0;
    for row in 0..problem.rows {
        for col in 0..problem.cols {
            norm = norm.hypot(problem.c(row, col));
        }
    }
    if !norm.is_finite() || norm <= f64::MIN_POSITIVE.sqrt() {
        return Err(Status::Singular);
    }
    let rho: f64 = 1.0 / norm / norm / 2.0_f64.sqrt();
    if !rho.is_finite() || rho <= 0.0 {
        return Err(Status::NumericalError);
    }
    Ok(rho)
}

impl DualState {
    /// Zero primal shift; equality multipliers start at 0, inequality ones at 1.
    fn new(problem: &Problem<'_, '_, '_>) -> Self {
        let mut dual: Vector<'static> = Vector::with_size(problem.rows);
        for row in 0..problem.rows {
            *dual.change_at(row).unwrap() = if row < problem.nce { 0.0 } else { 1.0 };
        }
        Self {
            shift: Vector::with_size(problem.cols),
            dual,
        }
    }

    /// `E = -C^T dual`; returns the largest component change once `track` (iteration > 0).
    fn update_primal(&mut self, problem: &Problem<'_, '_, '_>, track: bool) -> f64 {
        let mut change: f64 = 0.0;
        for col in 0..problem.cols {
            let previous: f64 = *self.shift.at(col).unwrap();
            let next: f64 = problem.primal_shift(&self.dual, col);
            *self.shift.change_at(col).unwrap() = next;
            if track {
                change = native_max(change, (next - previous).abs());
            }
        }
        change
    }

    /// Dual ascent on `C*X - S`, projecting inequality multipliers onto `>= 0`; returns the
    /// largest multiplier change.
    fn update_dual(
        &mut self,
        problem: &Problem<'_, '_, '_>,
        initial_error: &Vector<'_>,
        rho: f64,
    ) -> f64 {
        let mut change: f64 = 0.0;
        for row in 0..problem.rows {
            let mut violation: f64 = *initial_error.at(row).unwrap();
            for col in 0..problem.cols {
                violation += problem.c(row, col) * self.shift.at(col).unwrap();
            }
            let multiplier: &mut f64 = self.dual.change_at(row).unwrap();
            let row_change: f64 = if row < problem.nce {
                *multiplier += rho * violation;
                (rho * violation).abs()
            } else {
                let previous: f64 = *multiplier;
                *multiplier = native_max(0.0, *multiplier + rho * violation);
                (*multiplier - previous).abs()
            };
            change = native_max(change, row_change);
        }
        change
    }
}
