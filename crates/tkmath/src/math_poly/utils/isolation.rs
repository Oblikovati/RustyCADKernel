// Copyright (c) 2026 OPEN CASCADE SAS
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

//! `AppendUnique` and `SolveDistinctReal`: real-root isolation between critical points.
use super::{
    DISTINCT_ROOT_BISECTION_ITERATIONS, LongCoefficients, derivative,
    distinct_root_residual_tolerance, evaluate, has_small_residual, root_bound,
};
use crate::math_poly::extended::LongDouble;
use rustycad_tkernel::stl;
/// `AppendUnique`: appends `root` unless a stored root lies within
/// `128 * LDBL_EPSILON * max(1, |root|)`. A fifth distinct root overruns the native array
/// and panics here; `utils::append_unique(&mut roots, &mut count, root)`.
pub fn append_unique(roots: &mut [LongDouble; 4], count: &mut usize, root: LongDouble) {
    let tolerance: LongDouble =
        LongDouble::new(128.0) * LongDouble::epsilon() * LongDouble::new(1.0).max(root.abs());
    if roots[..*count]
        .iter()
        .any(|&other| (other - root).abs() <= tolerance)
    {
        return;
    }
    roots[*count] = root;
    *count += 1;
}
/// `SolveDistinctReal`: appends the distinct real roots of a degree >= 1 polynomial to
/// `roots[count..]`, recursing on the derivative to bracket them between critical points
/// and the Cauchy bound, then bisecting 200 times. Returns false only for a non-finite
/// linear root. A degree below 1 never terminates natively and returns false here;
/// `utils::solve_distinct_real(&c, 3, &mut roots, &mut count)`.
pub fn solve_distinct_real(
    coefficients: &LongCoefficients,
    degree: i32,
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) -> bool {
    if degree < 1 {
        return false;
    }
    if degree == 1 {
        let root: LongDouble = -coefficients[0] / coefficients[1];
        if !root.is_finite() {
            return false;
        }
        append_unique(roots, count, root);
        return true;
    }
    let mut critical: [LongDouble; 4] = [LongDouble::ZERO; 4];
    let mut critical_count: usize = 0;
    let derived: LongCoefficients = derivative(coefficients, degree);
    if !solve_distinct_real(&derived, degree - 1, &mut critical, &mut critical_count) {
        return false;
    }
    stl::sort_by(&mut critical[..critical_count], |a, b| a < b);
    let (points, point_count) = interval_points(
        coefficients,
        degree,
        &critical[..critical_count],
        roots,
        count,
    );
    bracketed_roots(coefficients, degree, &points[..point_count], roots, count);
    true
}
/// `-bound`, the distinct critical points and `bound`; critical points that are roots
/// themselves are appended on the way.
fn interval_points(
    c: &LongCoefficients,
    degree: i32,
    critical: &[LongDouble],
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) -> ([LongDouble; 6], usize) {
    let mut points: [LongDouble; 6] = [LongDouble::ZERO; 6];
    let bound: LongDouble = root_bound(c, degree);
    points[0] = -bound;
    let mut point_count: usize = 1;
    for (i, &point) in critical.iter().enumerate() {
        if i == 0 || point != critical[i - 1] {
            points[point_count] = point;
            point_count += 1;
        }
        if has_small_residual(c, degree, point, distinct_root_residual_tolerance()) {
            append_unique(roots, count, point);
        }
    }
    points[point_count] = bound;
    (points, point_count + 1)
}
/// Bisects every sign-changing interval whose inner end points are not roots already.
fn bracketed_roots(
    c: &LongCoefficients,
    degree: i32,
    points: &[LongDouble],
    roots: &mut [LongDouble; 4],
    count: &mut usize,
) {
    let small =
        |x: LongDouble| has_small_residual(c, degree, x, distinct_root_residual_tolerance());
    for i in 1..points.len() {
        let (left, right) = (points[i - 1], points[i]);
        let left_value: LongDouble = evaluate(c, degree, left).0;
        let right_value: LongDouble = evaluate(c, degree, right).0;
        if (i > 1 && small(left)) || (i + 1 < points.len() && small(right)) {
            continue;
        }
        if (left_value < LongDouble::ZERO) == (right_value < LongDouble::ZERO) {
            continue;
        }
        append_unique(roots, count, bisected(c, degree, left, right, left_value));
    }
}
fn bisected(
    c: &LongCoefficients,
    degree: i32,
    mut left: LongDouble,
    mut right: LongDouble,
    mut left_value: LongDouble,
) -> LongDouble {
    let two: LongDouble = LongDouble::new(2.0);
    for _ in 0..DISTINCT_ROOT_BISECTION_ITERATIONS {
        let middle: LongDouble = (left + right) / two;
        let value: LongDouble = evaluate(c, degree, middle).0;
        if (left_value < LongDouble::ZERO) == (value < LongDouble::ZERO) {
            left = middle;
            left_value = value;
        } else {
            right = middle;
        }
    }
    (left + right) / two
}
