//! MathUtils_Gauss.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{
        Vector, gauss_points_max, kronrod_points_and_weights, ordered_gauss_points_and_weights,
    },
    math_lin::{TridiagonalResult, eigen_tridiagonal},
};
/// Native ordered Gauss nodes, with modern eigen fallback; `get_gauss_points_and_weights(65, &mut points, &mut weights)`.
pub fn get_gauss_points_and_weights(
    order: i32,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    if order < 1 || points.length() != order || weights.length() != order {
        return false;
    }
    if order <= gauss_points_max() {
        return ordered_gauss_points_and_weights(order, points, weights);
    }
    let diagonal: Vector<'static> = Vector::with_size(order as usize);
    let mut subdiagonal: Vector<'static> = Vector::with_size(order as usize);
    for i in 1..order as usize {
        let square: usize = i * i;
        *subdiagonal.change_at(i).unwrap() = (square as f64 / (4.0 * square as f64 - 1.0)).sqrt();
    }
    let result: TridiagonalResult = eigen_tridiagonal(&diagonal, &subdiagonal, 30);
    if !result.is_done() {
        return false;
    }
    copy_eigen_quadrature(result, points, weights)
}
fn copy_eigen_quadrature(
    result: TridiagonalResult,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    let (Some(values), Some(vectors)) = (result.eigen_values, result.eigen_vectors) else {
        return false;
    };
    let mut pairs: Vec<(f64, f64)> = (0..values.length() as usize)
        .map(|i| {
            let first: f64 = *vectors.at(0, i).unwrap();
            (*values.at(i).unwrap(), 2.0 * first * first)
        })
        .collect();
    // std::sort by `Value <`; Legendre nodes are distinct, and an unordered NaN pair is kept
    // in place instead of panicking.
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (i, (point, weight)) in pairs.into_iter().enumerate() {
        *points.change_at(i).unwrap() = point;
        *weights.change_at(i).unwrap() = weight;
    }
    true
}
/// Native tabulated or computed Gauss-Kronrod nodes (odd order >= 3), written only on success;
/// `get_kronrod_points_and_weights(15, &mut points, &mut weights)`.
pub fn get_kronrod_points_and_weights(
    order: i32,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    if order < 1 || points.length() != order || weights.length() != order {
        return false;
    }
    // The native copy through temporaries is unobservable: math::KronrodPointsAndWeights
    // validates before it writes, and indexes relative to each vector's lower bound.
    kronrod_points_and_weights(order, points, weights)
}
/// Validated alias of `get_gauss_points_and_weights` (ascending nodes);
/// `get_ordered_gauss_points_and_weights(7, &mut points, &mut weights)`.
pub fn get_ordered_gauss_points_and_weights(
    order: i32,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    if order < 1 || points.length() != order || weights.length() != order {
        return false;
    }
    get_gauss_points_and_weights(order, points, weights)
}
