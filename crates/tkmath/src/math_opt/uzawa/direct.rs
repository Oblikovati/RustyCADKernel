//! Equality-only Uzawa: dual = (C*C^T)^-1 (C*X0 - S), X = X0 - C^T dual.
use super::{Problem, UzawaResult, finish};
use crate::math::{Matrix, Vector};
use crate::math_lin::invert_crout;
use crate::math_opt::utils::{is_finite_matrix, zero_based};
use crate::math_utils::{InverseResult, Status};

/// Native `InvertCrout` default minimum pivot.
const CROUT_MIN_PIVOT: f64 = 1.0e-15;

pub(super) fn solve(problem: &Problem<'_, '_, '_>, initial_error: Vector<'static>) -> UzawaResult {
    let mut result: UzawaResult = UzawaResult {
        iterations: 1,
        initial_error: Some(zero_based(&initial_error)),
        ..Default::default()
    };
    let inverse: Matrix<'static> = match inverse_ctc(problem) {
        Ok(inverse) => inverse,
        Err(status) => {
            result.status = status;
            return result;
        }
    };
    let dual: Vector<'static> = dual_variables(&inverse, &initial_error, problem.rows);
    let mut error: Vector<'static> = Vector::with_size(problem.cols);
    for col in 0..problem.cols {
        *error.change_at(col).unwrap() = problem.primal_shift(&dual, col);
    }
    let solution: Vector<'static> = problem.solution(&error);
    result.inverse_ctc = Some(inverse);
    finish(&mut result, solution, dual, error);
    result
}

/// Symmetric `C*C^T` (lower triangle computed, mirrored), Crout-inverted into zero-based
/// storage as native `aCTCinv = *Inverse` keeps the destination bounds.
fn inverse_ctc(problem: &Problem<'_, '_, '_>) -> Result<Matrix<'static>, Status> {
    let mut ctc: Matrix<'static> = gram_matrix(problem);
    let inverted: InverseResult = invert_crout(&ctc, CROUT_MIN_PIVOT);
    if !inverted.is_done() {
        return Err(Status::Singular);
    }
    ctc.initialized(inverted.inverse.as_ref().unwrap()).unwrap();
    if !is_finite_matrix(&ctc) {
        return Err(Status::NumericalError);
    }
    Ok(ctc)
}

fn gram_matrix(problem: &Problem<'_, '_, '_>) -> Matrix<'static> {
    let n: i32 = problem.rows as i32;
    let mut ctc: Matrix<'static> = Matrix::filled(0, n - 1, 0, n - 1, 0.0).unwrap();
    for i in 0..problem.rows {
        for j in 0..=i {
            let mut sum: f64 = 0.0;
            for k in 0..problem.cols {
                sum += problem.c(i, k) * problem.c(j, k);
            }
            *ctc.change_at(i, j).unwrap() = sum;
            *ctc.change_at(j, i).unwrap() = sum;
        }
    }
    ctc
}

fn dual_variables(
    inverse: &Matrix<'_>,
    initial_error: &Vector<'_>,
    rows: usize,
) -> Vector<'static> {
    let mut dual: Vector<'static> = Vector::with_size(rows);
    for i in 0..rows {
        let mut sum: f64 = 0.0;
        for j in 0..rows {
            sum += inverse.at(i, j).unwrap() * initial_error.at(j).unwrap();
        }
        *dual.change_at(i).unwrap() = sum;
    }
    dual
}
