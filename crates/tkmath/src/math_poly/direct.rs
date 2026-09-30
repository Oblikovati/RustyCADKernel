//! `MathPoly_Quadratic.hxx`, `MathPoly_Cubic.hxx` and `MathPoly_Quartic.hxx`: thin wrappers
//! that pass ascending coefficients to `Utils::SolveDirect`.
use super::PolyResult;
use super::utils::solve_direct;
use crate::math_utils::Status;
/// `Utils::SolveDirect` for degree <= 4. Such a polynomial has at most four distinct real
/// roots, so the four-root buffer cannot overflow and the failure arm is unreachable; it
/// would report the precondition as `InvalidInput`.
fn solve_low_degree(coefficients: &[f64]) -> PolyResult {
    let degree: i32 = coefficients.len() as i32 - 1;
    solve_direct(coefficients, degree).unwrap_or_else(|_| PolyResult::failed(Status::InvalidInput))
}
/// Solves a linear equation with native degenerate statuses; `linear(2.0,-4.0)`.
pub fn linear(a: f64, b: f64) -> PolyResult {
    solve_low_degree(&[b, a])
}
/// Uses the compensated native discriminant; `quadratic(1.0,-5.0,6.0)`.
pub fn quadratic(a: f64, b: f64, c: f64) -> PolyResult {
    solve_low_degree(&[c, b, a])
}
/// Isolates real roots recursively with x87 precision; `cubic(1.0,-6.0,11.0,-6.0)`.
pub fn cubic(a: f64, b: f64, c: f64, d: f64) -> PolyResult {
    solve_low_degree(&[d, c, b, a])
}
/// Preserves native real-root multiplicities; `quartic(1.0,-10.0,35.0,-50.0,24.0)`.
pub fn quartic(a: f64, b: f64, c: f64, d: f64, e: f64) -> PolyResult {
    solve_low_degree(&[e, d, c, b, a])
}
