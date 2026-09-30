use super::gradient::perturbed_value;
use super::{is_finite_vector, perturbed_pair, save_finite};
use crate::math::{Matrix, MultipleVarFunction, Vector};
fn diagonal(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    index: usize,
    fx: f64,
    step: f64,
) -> Option<f64> {
    let x: f64 = *point.at(index).unwrap();
    let (minus, plus): (f64, f64) = perturbed_pair(x, step * x.abs().max(1.0))?;
    let f_plus: f64 = perturbed_value(f, point, index, plus)?;
    let f_minus: f64 = perturbed_value(f, point, index, minus)?;
    let h_plus: f64 = plus - x;
    let h_minus: f64 = x - minus;
    Some(2.0 * ((f_plus - fx) / h_plus - (fx - f_minus) / h_minus) / (h_plus + h_minus))
}
fn pair_sample(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    indices: [usize; 2],
    x: [f64; 2],
) -> Option<f64> {
    let saved: f64 = *point.at(indices[0]).unwrap();
    *point.change_at(indices[0]).unwrap() = x[0];
    let result: Option<f64> = perturbed_value(f, point, indices[1], x[1]);
    *point.change_at(indices[0]).unwrap() = saved;
    result
}
fn mixed(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    indices: [usize; 2],
    step: f64,
) -> Option<f64> {
    let xi: f64 = *point.at(indices[0]).unwrap();
    let xj: f64 = *point.at(indices[1]).unwrap();
    let (mi, pi): (f64, f64) = perturbed_pair(xi, step * xi.abs().max(1.0))?;
    let (mj, pj): (f64, f64) = perturbed_pair(xj, step * xj.abs().max(1.0))?;
    let pp: f64 = pair_sample(f, point, indices, [pi, pj])?;
    let pm: f64 = pair_sample(f, point, indices, [pi, mj])?;
    let mm: f64 = pair_sample(f, point, indices, [mi, mj])?;
    let mp: f64 = pair_sample(f, point, indices, [mi, pj])?;
    Some((pp - pm - mp + mm) / ((pi - mi) * (pj - mj)))
}
fn fill_diagonal(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    matrix: &mut Matrix<'_>,
    fx: f64,
    step: f64,
) -> bool {
    for index in 0..point.length() as usize {
        let Some(value) = diagonal(f, point, index, fx, step) else {
            return false;
        };
        if !save_finite(value, matrix.change_at(index, index).unwrap()) {
            return false;
        }
    }
    true
}
fn fill_mixed_row(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    matrix: &mut Matrix<'_>,
    row: usize,
    step: f64,
) -> bool {
    for column in row + 1..point.length() as usize {
        let Some(value) = mixed(f, point, [row, column], step) else {
            return false;
        };
        if !save_finite(value, matrix.change_at(row, column).unwrap()) {
            return false;
        }
        *matrix.change_at(column, row).unwrap() = value;
    }
    true
}
/// Samples diagonals first and then symmetric mixed derivatives; `numerical_hessian(&mut f,&mut x,&mut h,1e-5)`.
pub fn numerical_hessian(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    matrix: &mut Matrix<'_>,
    step: f64,
) -> bool {
    if matrix.row_number() != point.length()
        || matrix.col_number() != point.length()
        || !step.is_finite()
        || step <= 0.0
        || !is_finite_vector(point)
    {
        return false;
    }
    let mut fx: f64 = 0.0;
    if !f.value(point, &mut fx) || !fx.is_finite() {
        return false;
    }
    if !fill_diagonal(f, point, matrix, fx, step) {
        return false;
    }
    for row in 0..point.length() as usize {
        if !fill_mixed_row(f, point, matrix, row, step) {
            return false;
        }
    }
    true
}
