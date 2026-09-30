use super::{is_finite_vector, perturbed_pair};
use crate::math::{FunctionSet, Matrix, Vector};
fn sample(
    f: &mut impl FunctionSet,
    point: &mut Vector<'_>,
    index: usize,
    x: f64,
    residual: &mut Vector<'_>,
) -> bool {
    let saved: f64 = *point.at(index).unwrap();
    *point.change_at(index).unwrap() = x;
    let success: bool = f.value(point, residual) && is_finite_vector(residual);
    *point.change_at(index).unwrap() = saved;
    success
}
fn fill_column(
    matrix: &mut Matrix<'_>,
    column: usize,
    plus: &Vector<'_>,
    minus: &Vector<'_>,
    delta: f64,
) -> bool {
    for row in 0..matrix.row_number() as usize {
        let value: f64 = (plus.at(row).unwrap() - minus.at(row).unwrap()) / delta;
        if !value.is_finite() {
            return false;
        }
        *matrix.change_at(row, column).unwrap() = value;
    }
    true
}
fn jacobian_column(
    f: &mut impl FunctionSet,
    point: &mut Vector<'_>,
    matrix: &mut Matrix<'_>,
    column: usize,
    step: f64,
    samples: &mut [Vector<'_>; 2],
) -> bool {
    let x: f64 = *point.at(column).unwrap();
    let Some((minus, plus)) = perturbed_pair(x, step * x.abs().max(1.0)) else {
        return false;
    };
    if !sample(f, point, column, plus, &mut samples[0])
        || !sample(f, point, column, minus, &mut samples[1])
    {
        return false;
    }
    fill_column(matrix, column, &samples[0], &samples[1], plus - minus)
}
/// Preserves completed columns on failure; `numerical_jacobian(&mut f,&mut x,&mut jac,1e-8)`.
pub fn numerical_jacobian(
    f: &mut impl FunctionSet,
    point: &mut Vector<'_>,
    matrix: &mut Matrix<'_>,
    step: f64,
) -> bool {
    if point.length() != matrix.col_number()
        || matrix.row_number() == 0
        || !step.is_finite()
        || step <= 0.0
        || !is_finite_vector(point)
    {
        return false;
    }
    let mut samples: [Vector<'_>; 2] =
        std::array::from_fn(|_| Vector::with_size(matrix.row_number() as usize));
    for column in 0..matrix.col_number() as usize {
        if !jacobian_column(f, point, matrix, column, step, &mut samples) {
            return false;
        }
    }
    true
}
