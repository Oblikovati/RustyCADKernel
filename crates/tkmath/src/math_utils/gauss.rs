// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! MathUtils_Gauss.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::{
    math::{Vector, ordered_gauss_points_and_weights},
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
    if order <= 61 {
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
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (i, (point, weight)) in pairs.into_iter().enumerate() {
        *points.change_at(i).unwrap() = point;
        *weights.change_at(i).unwrap() = weight;
    }
    true
}
