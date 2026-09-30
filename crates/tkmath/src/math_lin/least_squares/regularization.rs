//! Native ridge filtering and generalized cross-validation grid.
use super::*;
fn filtered_projection(factors: &SvdResult, b: &Vector<'_>, lambda: f64) -> Vector<'static> {
    let u: &Matrix<'_> = factors.u.as_ref().unwrap();
    let s: &Vector<'_> = factors.singular_values.as_ref().unwrap();
    let mut filtered: Vector<'static> = Vector::with_size(s.length() as usize);
    for j in 0..s.length() {
        let mut projection: f64 = 0.0;
        for i in 0..u.row_number() {
            projection += u[(i, j)] * b.at(i as usize).unwrap();
        }
        let filter: f64 = if j as usize >= factors.rank {
            0.0
        } else {
            1.0 / (s[j] + lambda / s[j])
        };
        filtered[j] = filter * projection;
    }
    filtered
}
fn filtered_solution(factors: &SvdResult, b: &Vector<'_>, lambda: f64) -> Vector<'static> {
    let filtered: Vector<'static> = filtered_projection(factors, b, lambda);
    let v: &Matrix<'_> = factors.v.as_ref().unwrap();
    let mut x: Vector<'static> = Vector::with_size(filtered.length() as usize);
    for i in 0..x.length() {
        for j in 0..x.length() {
            x[i] += v[(i, j)] * filtered[j];
        }
    }
    x
}
/// Applies native Tikhonov filtering; `regularized_least_squares(&a,&b,0.01,1e-15)`.
pub fn regularized_least_squares(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    lambda: f64,
    tolerance: f64,
) -> LeastSquaresResult {
    if !valid_system(a, b) || !lambda.is_finite() || lambda < 0.0 {
        return LeastSquaresResult::failed(Status::InvalidInput);
    }
    let factors: SvdResult = svd(a, tolerance);
    if !factors.is_done() {
        return LeastSquaresResult::failed(factors.status);
    }
    let mut out: LeastSquaresResult = LeastSquaresResult {
        solution: Some(filtered_solution(&factors, b, lambda)),
        rank: factors.rank,
        condition_number: factors.condition_number,
        ..Default::default()
    };
    out.finish(a, b);
    out
}
fn normalized_projection(u: &Matrix<'_>, b: &Vector<'_>) -> Option<(Vector<'static>, f64)> {
    let mut scale: f64 = 0.0;
    let mut sum: f64 = 1.0;
    for i in 0..b.length() {
        if !accumulate_norm(*b.at(i as usize).unwrap(), &mut scale, &mut sum) {
            return None;
        }
    }
    let mut projection: Vector<'static> = Vector::with_size(u.col_number() as usize);
    if scale != 0.0 {
        project_scaled(u, b, scale, &mut projection);
    }
    let mut orthogonal: f64 = if scale == 0.0 { 0.0 } else { sum };
    for j in 0..projection.length() {
        orthogonal -= projection[j] * projection[j];
    }
    Some((projection, 0.0_f64.max(orthogonal)))
}
fn project_scaled(u: &Matrix<'_>, b: &Vector<'_>, scale: f64, projection: &mut Vector<'_>) {
    for i in 0..u.row_number() {
        let normalized: f64 = b.at(i as usize).unwrap() / scale;
        for j in 0..u.col_number() {
            projection[j] += u[(i, j)] * normalized;
        }
    }
}
fn gcv_score(
    s: &Vector<'_>,
    projection: &Vector<'_>,
    mut residual: f64,
    rows: i32,
    lambda: f64,
) -> f64 {
    let mut trace: f64 = 0.0;
    for j in 0..s.length() {
        let sigma: f64 = s[j];
        let filter: f64 = if sigma == 0.0 {
            0.0
        } else {
            1.0 / (1.0 + (lambda / sigma) / sigma)
        };
        let projected: f64 = (1.0 - filter) * projection[j];
        residual += projected * projected;
        trace += filter;
    }
    let denominator: f64 = rows as f64 - trace;
    if denominator > 0.0 {
        residual / (denominator * denominator)
    } else {
        f64::INFINITY
    }
}
fn grid_search(
    factors: &SvdResult,
    b: &Vector<'_>,
    minimum: f64,
    maximum: f64,
    points: usize,
) -> Option<f64> {
    let u: &Matrix<'_> = factors.u.as_ref().unwrap();
    let s: &Vector<'_> = factors.singular_values.as_ref().unwrap();
    let (projection, orthogonal) = normalized_projection(u, b)?;
    let log_min: f64 = minimum.log10();
    let log_step: f64 = (maximum.log10() - log_min) / (points - 1) as f64;
    let mut best_score: f64 = f64::INFINITY;
    let mut best_lambda: Option<f64> = None;
    for k in 0..points {
        let lambda: f64 = 10.0_f64.powf(log_min + k as f64 * log_step);
        let score: f64 = gcv_score(s, &projection, orthogonal, u.row_number(), lambda);
        if score.is_finite() && score < best_score {
            best_score = score;
            best_lambda = Some(lambda);
        }
    }
    best_lambda
}
/// Chooses the native logarithmic GCV grid minimum; `optimal_regularization(&a,&b,1e-10,1e2,20)`.
pub fn optimal_regularization(
    a: &Matrix<'_>,
    b: &Vector<'_>,
    minimum: f64,
    maximum: f64,
    points: usize,
) -> Option<f64> {
    if a.row_number() < a.col_number()
        || !valid_system(a, b)
        || !minimum.is_finite()
        || !maximum.is_finite()
        || minimum <= 0.0
        || maximum < minimum
        || points < 2
    {
        return None;
    }
    let factors: SvdResult = svd(a, 1e-15);
    if !factors.is_done() {
        return None;
    }
    grid_search(&factors, b, minimum, maximum, points)
}
