use super::{Config, is_within_tolerance, is_zero, vector_inf_norm, vector_norm};
use crate::math::Vector;
/// Uses the tolerance for both absolute and relative change; `is_x_converged(a,b,1e-8)`.
pub fn is_x_converged(old: f64, new: f64, tolerance: f64) -> bool {
    is_within_tolerance(old, new, tolerance, tolerance)
}
/// Checks finite functional magnitude; `is_f_converged(value,1e-8)`.
pub fn is_f_converged(value: f64, tolerance: f64) -> bool {
    is_zero(value, tolerance)
}
/// Accepts parameter or function convergence with valid settings; `is_converged(a,b,f,&config)`.
pub fn is_converged(old: f64, new: f64, value: f64, config: &Config) -> bool {
    config.is_valid()
        && value.is_finite()
        && (is_within_tolerance(old, new, config.x_tolerance, config.relative_tolerance)
            || is_f_converged(value, config.f_tolerance))
}
/// Accepts parameter or functional change; `is_min_converged(x0,x1,f0,f1,&config)`.
pub fn is_min_converged(old_x: f64, new_x: f64, old_f: f64, new_f: f64, config: &Config) -> bool {
    config.is_valid()
        && old_f.is_finite()
        && new_f.is_finite()
        && (is_within_tolerance(old_x, new_x, config.x_tolerance, config.relative_tolerance)
            || is_within_tolerance(old_f, new_f, config.f_tolerance, config.relative_tolerance))
}
/// Compares matching vector positions independently of lower bounds; `is_vector_converged(&old,&new,1e-8)`.
pub fn is_vector_converged(old: &Vector<'_>, new: &Vector<'_>, tolerance: f64) -> bool {
    old.length() == new.length()
        && tolerance.is_finite()
        && tolerance >= 0.0
        && old
            .values()
            .iter()
            .zip(new.values())
            .all(|(&a, &b)| is_within_tolerance(a, b, tolerance, tolerance))
}
/// Applies the scaled gradient norm; `is_gradient_converged(&gradient,1e-8)`.
pub fn is_gradient_converged(gradient: &Vector<'_>, tolerance: f64) -> bool {
    tolerance.is_finite() && tolerance >= 0.0 && vector_norm(gradient) <= tolerance
}
/// Native infinity-norm alias; `infinity_norm(&vector)`.
pub fn infinity_norm(vector: &Vector<'_>) -> f64 {
    vector_inf_norm(vector)
}
/// Native Euclidean-norm alias; `euclidean_norm(&vector)`.
pub fn euclidean_norm(vector: &Vector<'_>) -> f64 {
    vector_norm(vector)
}
