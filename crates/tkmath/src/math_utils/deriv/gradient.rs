use super::{is_finite_vector, perturbed_pair, save_finite};
use crate::math::{MultipleVarFunction, Vector};
fn valid_input(point: &Vector<'_>, gradient: &Vector<'_>, step: f64) -> bool {
    point.length() == gradient.length() && step.is_finite() && step > 0.0 && is_finite_vector(point)
}
pub(super) fn perturbed_value(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    index: usize,
    x: f64,
) -> Option<f64> {
    let saved: f64 = *point.at(index).unwrap();
    *point.change_at(index).unwrap() = x;
    let mut value: f64 = 0.0;
    let success: bool = f.value(point, &mut value);
    *point.change_at(index).unwrap() = saved;
    (success && value.is_finite()).then_some(value)
}
fn gradient_component(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    index: usize,
    output: &mut f64,
    step: f64,
) -> bool {
    let x: f64 = *point.at(index).unwrap();
    let Some((minus, plus)) = perturbed_pair(x, step) else {
        return false;
    };
    let Some(f_plus) = perturbed_value(f, point, index, plus) else {
        return false;
    };
    let Some(f_minus) = perturbed_value(f, point, index, minus) else {
        return false;
    };
    save_finite((f_plus - f_minus) / (plus - minus), output)
}
fn gradient_impl(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    gradient: &mut Vector<'_>,
    step: f64,
    adaptive: bool,
) -> bool {
    if !valid_input(point, gradient, step) {
        return false;
    }
    for index in 0..point.length() as usize {
        let increment: f64 = if adaptive {
            step * point.at(index).unwrap().abs().max(1.0)
        } else {
            step
        };
        if !gradient_component(
            f,
            point,
            index,
            gradient.change_at(index).unwrap(),
            increment,
        ) {
            return false;
        }
    }
    true
}
/// Computes central differences while restoring the input on failure; `numerical_gradient(&mut f,&mut x,&mut g,1e-8)`.
pub fn numerical_gradient(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    gradient: &mut Vector<'_>,
    step: f64,
) -> bool {
    gradient_impl(f, point, gradient, step, false)
}
/// Scales each variable's step independently; `numerical_gradient_adaptive(&mut f,&mut x,&mut g,1e-8)`.
pub fn numerical_gradient_adaptive(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    gradient: &mut Vector<'_>,
    step: f64,
) -> bool {
    gradient_impl(f, point, gradient, step, true)
}
