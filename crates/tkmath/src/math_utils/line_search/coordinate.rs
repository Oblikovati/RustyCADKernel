use super::brent::BrentState;
use super::*;
fn valid(point: &Vector<'_>, index: usize, fx: f64, parameters: [f64; 3], iterations: u32) -> bool {
    let [lower, upper, tolerance]: [f64; 3] = parameters;
    if index >= point.length() as usize
        || !lower.is_finite()
        || !upper.is_finite()
        || lower > upper
        || !tolerance.is_finite()
        || tolerance <= 0.0
        || iterations == 0
    {
        return false;
    }
    let x: f64 = *point.at(index).unwrap();
    x.is_finite() && x >= lower && x <= upper && fx.is_finite()
}
fn iterate(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    index: usize,
    state: &mut BrentState,
    parameters: [f64; 2],
    iterations: u32,
    evaluations: &mut usize,
) -> bool {
    let [tolerance, original]: [f64; 2] = parameters;
    for _ in 0..iterations {
        let mid: f64 = 0.5 * (state.a + state.b);
        let tol: f64 = tolerance * state.x.abs() + ZERO_TOL / 10.0;
        if state.converged(mid, 2.0 * tol) {
            break;
        }
        let u: f64 = state.candidate(mid, tol, false);
        *point.change_at(index).unwrap() = u;
        let mut value: f64 = 0.0;
        if !f.value(point, &mut value) || !value.is_finite() {
            *point.change_at(index).unwrap() = original;
            return false;
        }
        *evaluations += 1;
        state.update(u, value);
    }
    true
}
/// Optimizes one coordinate using `[lower,upper,tolerance]`; `brent_along_coordinate(&mut f,&mut x,0,&mut fx,[-2.0,2.0,1e-6],100,&mut evaluations)`.
pub fn brent_along_coordinate(
    f: &mut impl MultipleVarFunction,
    point: &mut Vector<'_>,
    index: usize,
    fx: &mut f64,
    parameters: [f64; 3],
    iterations: u32,
    evaluations: &mut usize,
) -> bool {
    if !valid(point, index, *fx, parameters, iterations) {
        return false;
    }
    let [lower, upper, tolerance]: [f64; 3] = parameters;
    let original: f64 = *point.at(index).unwrap();
    let mut state: BrentState = BrentState::new(lower, upper, original, *fx);
    if !iterate(
        f,
        point,
        index,
        &mut state,
        [tolerance, original],
        iterations,
        evaluations,
    ) {
        return false;
    }
    if state.fx < *fx {
        *point.change_at(index).unwrap() = state.x;
        *fx = state.fx;
        return true;
    }
    *point.change_at(index).unwrap() = original;
    false
}
