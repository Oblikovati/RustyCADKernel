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

//! Critical-point isolation using software x87 long double arithmetic.
use super::{E, exponent};
fn derivative(c: &[E]) -> Vec<E> {
    (1..c.len()).map(|i| E::new(i as f64) * c[i]).collect()
}
fn evaluated(c: &[E], x: E) -> (E, E, E) {
    let mut value: E = *c.last().unwrap();
    let mut derivative: E = E::ZERO;
    let mut magnitude: E = value.abs();
    for coefficient in c[..c.len() - 1].iter().rev() {
        derivative = derivative * x + value;
        value = value * x + *coefficient;
        magnitude = magnitude * x.abs() + coefficient.abs();
    }
    (value, derivative, magnitude)
}
pub(super) fn small_residual(c: &[E], root: E, tolerance: f64) -> bool {
    let (value, _, magnitude) = evaluated(c, root);
    value.finite()
        && value.abs() <= E::new(tolerance * 2.0_f64.powi(-63)) * E::new(1.0).max(magnitude)
}
fn root_bound(c: &[E]) -> E {
    let leading: E = c.last().unwrap().abs();
    let ratio: E = c[..c.len() - 1]
        .iter()
        .fold(E::ZERO, |s, x| s.max(x.abs() / leading));
    E::new(1.0) + ratio
}
fn append_unique(roots: &mut Vec<E>, root: E) {
    let tolerance: E = E::new(128.0 * 2.0_f64.powi(-63)) * E::new(1.0).max(root.abs());
    if roots.iter().any(|&other| (other - root).abs() <= tolerance) {
        return;
    }
    roots.push(root);
}
fn variable_exponent(c: &[f64]) -> i32 {
    let degree: usize = c.len() - 1;
    let leading: i32 = exponent(c[degree].abs());
    let mut scale: E = E::new(-1000.0);
    for (i, &coefficient) in c.iter().enumerate().take(degree) {
        if coefficient == 0.0 {
            continue;
        }
        let difference: i32 = exponent(coefficient.abs()) - leading;
        scale = scale.max(E::new(difference as f64) / E::new((degree - i) as f64));
    }
    scale.ceil_integer().clamp(-1000, 1000)
}
pub(super) fn scaled_polynomial(c: &[f64]) -> (Vec<E>, i32) {
    let power: i32 = variable_exponent(c);
    let mut common: i32 = exponent(c[c.len() - 1].abs()) + (c.len() - 1) as i32 * power;
    for (i, &value) in c.iter().enumerate() {
        if value != 0.0 {
            common = common.max(exponent(value.abs()) + i as i32 * power);
        }
    }
    (
        c.iter()
            .enumerate()
            .map(|(i, &value)| E::new(value).scale(i as i32 * power - common))
            .collect(),
        power,
    )
}
fn interval_root(c: &[E], mut left: E, mut right: E, mut left_value: E) -> E {
    for _ in 0..200 {
        let middle: E = (left + right) / E::new(2.0);
        let (value, _, _) = evaluated(c, middle);
        if (left_value < E::ZERO) == (value < E::ZERO) {
            left = middle;
            left_value = value;
        } else {
            right = middle;
        }
    }
    (left + right) / E::new(2.0)
}
fn interval_roots(c: &[E], points: &[E], roots: &mut Vec<E>) {
    for i in 1..points.len() {
        let left: E = points[i - 1];
        let right: E = points[i];
        let (left_value, _, _) = evaluated(c, left);
        let (right_value, _, _) = evaluated(c, right);
        if (i > 1 && small_residual(c, left, 64.0))
            || (i + 1 < points.len() && small_residual(c, right, 64.0))
        {
            continue;
        }
        if (left_value < E::ZERO) == (right_value < E::ZERO) {
            continue;
        }
        append_unique(roots, interval_root(c, left, right, left_value));
    }
}
pub(super) fn distinct_real(c: &[E]) -> Option<Vec<E>> {
    let mut roots: Vec<E> = Vec::new();
    if c.len() == 2 {
        let root: E = -c[0] / c[1];
        if !root.finite() {
            return None;
        }
        append_unique(&mut roots, root);
        return Some(roots);
    }
    let mut critical: Vec<E> = distinct_real(&derivative(c))?;
    critical.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let bound: E = root_bound(c);
    let mut points: Vec<E> = vec![-bound];
    for (i, &root) in critical.iter().enumerate() {
        if i == 0 || root != critical[i - 1] {
            points.push(root);
        }
        if small_residual(c, root, 64.0) {
            append_unique(&mut roots, root);
        }
    }
    points.push(bound);
    interval_roots(c, &points, &mut roots);
    Some(roots)
}
pub(super) fn multiplicity(c: &[E], root: E) -> usize {
    let mut current: Vec<E> = c.to_vec();
    let mut count: usize = 1;
    while count < c.len() - 1 && current.len() > 1 {
        let next: Vec<E> = derivative(&current);
        if !small_residual(&next, root, 32768.0) {
            break;
        }
        count += 1;
        current = next;
    }
    count
}
