// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Pinned Gauss/Kronrod decimal tables, with native computed fallback.
use super::{GaussPointsAndWeights, KronrodPointsAndWeights, Vector};
mod gauss;
mod kronrod;
/// Largest tabulated Gauss order; `gauss_points_max()` is 61.
pub fn gauss_points_max() -> i32 {
    61
}
/// Largest tabulated Kronrod order; `kronrod_points_max()` is 123.
pub fn kronrod_points_max() -> i32 {
    123
}
/// Copies positive-first native Gauss nodes; `gauss_points(5,&mut points)`.
pub fn gauss_points(order: i32, points: &mut Vector<'_>) {
    let start: usize = (1..order).map(|i| ((i + 1) >> 1) as usize).sum();
    let half: i32 = (order + 1) >> 1;
    for i in 1..=half {
        points[i] = gauss::point(start + i as usize);
        if i + half <= order {
            points[i + half] = -points[i];
        }
    }
}
/// Copies native Gauss weights; `gauss_weights(5,&mut weights)`.
pub fn gauss_weights(order: i32, weights: &mut Vector<'_>) {
    let start: usize = (1..order).map(|i| ((i + 1) >> 1) as usize).sum();
    let half: i32 = (order + 1) >> 1;
    for i in 1..=half {
        weights[i] = gauss::weight(start + i as usize);
        if i + half <= order {
            weights[i + half] = weights[i];
        }
    }
}
/// Returns ordered Gauss nodes with native validation; `ordered_gauss_points_and_weights(n,&mut p,&mut w)`.
pub fn ordered_gauss_points_and_weights(
    order: i32,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    if order < 1 || points.length() != order || weights.length() != order {
        return false;
    }
    if order > 61 {
        let Ok(computed) = GaussPointsAndWeights::new(order) else {
            return false;
        };
        if !computed.is_done() {
            return false;
        }
        points.initialized(&computed.points()).unwrap();
        weights.initialized(&computed.weights()).unwrap();
        return true;
    }
    let start: usize = 1 + (1..order).map(|i| ((i + 1) / 2) as usize).sum::<usize>();
    copy_table(order, start, points, weights, gauss::point, gauss::weight);
    true
}
/// Returns ordered Kronrod nodes, computing orders above 123; `kronrod_points_and_weights(n,&mut p,&mut w)`.
pub fn kronrod_points_and_weights(
    order: i32,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
) -> bool {
    if order < 3 || order % 2 == 0 || points.length() != order || weights.length() != order {
        return false;
    }
    if order > 123 {
        let Ok(computed) = KronrodPointsAndWeights::new((order - 1) / 2) else {
            return false;
        };
        if !computed.is_done() {
            return false;
        }
        points.initialized(&computed.points()).unwrap();
        weights.initialized(&computed.weights()).unwrap();
        return true;
    }
    let start: usize = (3..order).step_by(2).map(|i| ((i + 1) / 2) as usize).sum();
    copy_table(
        order,
        start,
        points,
        weights,
        kronrod::point,
        kronrod::weight,
    );
    true
}
fn copy_table(
    order: i32,
    start: usize,
    points: &mut Vector<'_>,
    weights: &mut Vector<'_>,
    point: fn(usize) -> f64,
    weight: fn(usize) -> f64,
) {
    let lower: i32 = points.lower();
    let upper: i32 = points.upper();
    for i in 0..order / 2 {
        points[lower + i] = -point(start + i as usize);
        points[upper - i] = point(start + i as usize);
        weights[lower + i] = weight(start + i as usize);
        weights[upper - i] = weight(start + i as usize);
    }
    if order % 2 == 1 {
        points[lower + order / 2] = point(start + (order / 2) as usize);
        weights[lower + order / 2] = weight(start + (order / 2) as usize);
    }
}
