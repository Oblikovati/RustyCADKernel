use super::{Matrix, Vector};
use crate::math::{Gauss, GaussLeastSquares, Svd};
pub(super) fn search(
    jacobian: &Matrix<'_>,
    gradient: &Vector<'_>,
    residual: &Vector<'_>,
    mut change: bool,
    inverse_lengths: &Vector<'_>,
    direction: &mut Vector<'_>,
) -> f64 {
    if !change {
        change = !newton_direction(jacobian, residual, direction);
    }
    limit_direction(direction, inverse_lengths);
    let mut slope: f64 = direction.dot_unchecked(gradient);
    if slope >= -1e-32 {
        change = true;
    }
    if change {
        for index in 1..=direction.length() {
            direction[index] = -gradient[index];
        }
        slope = -gradient.norm2_unchecked();
    }
    slope
}
fn newton_direction(
    jacobian: &Matrix<'_>,
    residual: &Vector<'_>,
    direction: &mut Vector<'_>,
) -> bool {
    let negative: Vector<'_> = residual.multiplied(-1.0);
    if jacobian.col_number() == jacobian.row_number() {
        direction.initialized(&negative).unwrap();
        let gaussian: Gauss = Gauss::with_min_pivot(jacobian, 1e-9).unwrap();
        if gaussian.is_done() {
            gaussian.solve(direction).unwrap();
            return true;
        }
    }
    if jacobian.col_number() >= jacobian.row_number() {
        return solve_svd(jacobian, &negative, direction);
    }
    let least: GaussLeastSquares = GaussLeastSquares::new(jacobian).unwrap();
    if !least.is_done() {
        return false;
    }
    least.solve_into(&negative, direction).unwrap();
    true
}
fn solve_svd(jacobian: &Matrix<'_>, negative: &Vector<'_>, direction: &mut Vector<'_>) -> bool {
    let mut singular: Svd = Svd::new(jacobian).unwrap();
    if !singular.is_done() {
        return false;
    }
    singular.solve(negative, direction, 1e-6).unwrap();
    true
}
fn limit_direction(direction: &mut Vector<'_>, inverse_lengths: &Vector<'_>) {
    let mut ratio: f64 = (direction[1] * inverse_lengths[1]).abs();
    for index in 2..=direction.length() {
        let candidate: f64 = (direction[index] * inverse_lengths[index]).abs();
        if ratio < candidate {
            ratio = candidate;
        }
    }
    if ratio > 1.0 {
        direction.divide(ratio).unwrap();
    }
}
pub(super) fn constrained(
    jacobian: &Matrix<'_>,
    gradient: &Vector<'_>,
    residual: &Vector<'_>,
    constraints: &[bool],
    change: bool,
    inverse_lengths: &Vector<'_>,
    direction: &mut Vector<'_>,
) -> f64 {
    let free: Vec<i32> = (1..=direction.length())
        .filter(|index| !constraints[(*index - 1) as usize])
        .collect();
    if free.len() == direction.length() as usize {
        return search(
            jacobian,
            gradient,
            residual,
            change,
            inverse_lengths,
            direction,
        );
    }
    if free.is_empty() {
        direction.init(0.0);
        return 0.0;
    }
    let n: i32 = free.len() as i32;
    let mut reduced: Matrix<'static> = Matrix::with_bounds(1, jacobian.row_number(), 1, n).unwrap();
    let mut g: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    let mut d: Vector<'static> = Vector::with_bounds(1, n).unwrap();
    let mut lengths: Vector<'static> = Vector::with_bounds(1, direction.length()).unwrap();
    fill_reduced(
        jacobian,
        gradient,
        inverse_lengths,
        direction,
        &free,
        &mut reduced,
        &mut g,
        &mut d,
        &mut lengths,
    );
    let slope: f64 = search(&reduced, &g, residual, change, &lengths, &mut d);
    direction.init(0.0);
    for (offset, index) in free.into_iter().enumerate() {
        direction[index] = if change {
            -gradient[index]
        } else {
            d[offset as i32 + 1]
        };
    }
    slope
}
#[allow(clippy::too_many_arguments)]
fn fill_reduced(
    jacobian: &Matrix<'_>,
    gradient: &Vector<'_>,
    inverse_lengths: &Vector<'_>,
    direction: &Vector<'_>,
    free: &[i32],
    reduced: &mut Matrix<'_>,
    g: &mut Vector<'_>,
    d: &mut Vector<'_>,
    lengths: &mut Vector<'_>,
) {
    for (offset, index) in free.iter().copied().enumerate() {
        let col: i32 = offset as i32 + 1;
        g[col] = gradient[index];
        d[col] = direction[index];
        lengths[col] = inverse_lengths[index];
        for row in 1..=jacobian.row_number() {
            reduced[(row, col)] = jacobian[(row, index)];
        }
    }
}
