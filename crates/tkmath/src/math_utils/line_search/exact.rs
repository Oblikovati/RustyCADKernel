use super::brent::BrentState;
use super::*;
fn evaluate_best(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    alpha: f64,
    candidate: &mut Vector<'_>,
    result: &mut LineSearchResult,
) -> Option<f64> {
    if !alpha.is_finite() || !new_point(point, direction, alpha, candidate) {
        result.status = Status::NumericalError;
        return None;
    }
    let value: f64 = result.evaluate(f, candidate)?;
    if !result.finite(value) {
        return None;
    }
    if !result.f_new.is_finite() || value < result.f_new {
        result.alpha = alpha;
        result.f_new = value;
    }
    Some(value)
}
fn valid(
    point: &Vector<'_>,
    direction: &Vector<'_>,
    limit: f64,
    tolerance: f64,
    iterations: u32,
) -> bool {
    point.length() == direction.length()
        && limit.is_finite()
        && limit > 0.0
        && tolerance.is_finite()
        && tolerance > 0.0
        && iterations > 0
        && point
            .values()
            .iter()
            .chain(direction.values())
            .all(|v| v.is_finite())
}
/// Searches [-limit,limit] with Brent's method; `exact_line_search(&mut f,&x,&d,10.0,1e-6,100)`.
pub fn exact_line_search(
    f: &mut impl MultipleVarFunction,
    point: &Vector<'_>,
    direction: &Vector<'_>,
    limit: f64,
    tolerance: f64,
    iterations: u32,
) -> LineSearchResult {
    let mut result: LineSearchResult = LineSearchResult::default();
    if !valid(point, direction, limit, tolerance, iterations) {
        result.status = Status::InvalidInput;
        return result;
    }
    let mut candidate: Vector<'_> = Vector::with_size(point.length() as usize);
    let Some(fx) = evaluate_best(f, point, direction, 0.0, &mut candidate, &mut result) else {
        return result;
    };
    let mut state: BrentState = BrentState::new(-limit, limit, 0.0, fx);
    for _ in 0..iterations {
        let mid: f64 = 0.5 * state.a + 0.5 * state.b;
        let tol: f64 = tolerance * state.x.abs() + ZERO_TOL / 10.0;
        if !result.finite(tol) || !result.finite(2.0 * tol) {
            return result;
        }
        if state.converged(mid, 2.0 * tol) {
            result.success();
            result.alpha = state.x;
            result.f_new = state.fx;
            return result;
        }
        let u: f64 = state.candidate(mid, tol, true);
        let Some(fu) = evaluate_best(f, point, direction, u, &mut candidate, &mut result) else {
            return result;
        };
        state.update(u, fu);
    }
    result.status = Status::MaxIterations;
    result
}
