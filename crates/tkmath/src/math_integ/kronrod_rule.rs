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

//! MathInteg_Kronrod.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::gauss::{finite_value, valid_interval};
use crate::{
    math::{Function, Vector, kronrod_points_and_weights, ordered_gauss_points_and_weights},
    math_utils::{IntegResult, Status},
};
struct EmbeddedRule {
    weights: Vector<'static>,
    nodes: Vector<'static>,
    gauss: Vector<'static>,
    left: Vec<f64>,
    right: Vec<f64>,
    kronrod_sum: f64,
    gauss_sum: f64,
    absolute_sum: f64,
    center: f64,
}
/// Highest embedded Gauss order with tabulated Kronrod extensions
/// (`MathInteg::THE_KRONROD_MAX_GAUSS_ORDER`); `assert_eq!(KRONROD_MAX_GAUSS_ORDER, 30)`.
pub const KRONROD_MAX_GAUSS_ORDER: usize = 30;

/// Embedded Gauss-Kronrod estimates with QUADPACK error floor; `kronrod_rule(&mut f, 0.0, 1.0, 7)`.
pub fn kronrod_rule(f: &mut impl Function, lower: f64, upper: f64, order: usize) -> IntegResult {
    if order == 0 || order > KRONROD_MAX_GAUSS_ORDER || !valid_interval(lower, upper) {
        return IntegResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    if lower == upper {
        return IntegResult {
            status: Status::Ok,
            value: Some(0.0),
            absolute_error: Some(0.0),
            ..Default::default()
        };
    }
    if lower > upper {
        let mut result: IntegResult = kronrod_rule(f, upper, lower, order);
        result.value = result.value.map(|value| -value);
        return result;
    }
    let Some(mut rule) = EmbeddedRule::new(order) else {
        return IntegResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    };
    let half: f64 = 0.5 * (upper - lower);
    let mut result: IntegResult = IntegResult::default();
    if let Err(status) = rule.evaluate(f, [lower + half, half], &mut result) {
        result.status = status;
        return result;
    }
    rule.finish(half, result)
}
impl EmbeddedRule {
    fn new(order: usize) -> Option<Self> {
        let mut nodes: Vector<'static> = Vector::with_size(2 * order + 1);
        let mut weights: Vector<'static> = Vector::with_size(2 * order + 1);
        let mut gauss: Vector<'static> = Vector::with_size(order);
        let mut unused: Vector<'static> = Vector::with_size(order);
        if !kronrod_points_and_weights((2 * order + 1) as i32, &mut nodes, &mut weights)
            || !ordered_gauss_points_and_weights(order as i32, &mut unused, &mut gauss)
        {
            return None;
        }
        Some(Self {
            nodes,
            weights,
            gauss,
            left: vec![0.0; order + 1],
            right: vec![0.0; order + 1],
            kronrod_sum: 0.0,
            gauss_sum: 0.0,
            absolute_sum: 0.0,
            center: 0.0,
        })
    }
    fn evaluate(
        &mut self,
        f: &mut impl Function,
        transform: [f64; 2],
        result: &mut IntegResult,
    ) -> Result<(), Status> {
        let half_count: usize = self.left.len();
        for i in (1..half_count - 1).step_by(2) {
            self.pair(f, transform, i, true, result)?;
        }
        result.number_of_points += 1;
        self.center = finite_value(f, transform[0])?;
        self.kronrod_sum += self.center * self.weights.at(half_count - 1).unwrap();
        self.absolute_sum += self.center.abs() * self.weights.at(half_count - 1).unwrap();
        if half_count.is_multiple_of(2) {
            self.gauss_sum += self.center * self.gauss.at(half_count / 2 - 1).unwrap();
        }
        for i in (0..half_count - 1).step_by(2) {
            self.pair(f, transform, i, false, result)?;
        }
        Ok(())
    }
    fn pair(
        &mut self,
        f: &mut impl Function,
        transform: [f64; 2],
        i: usize,
        embedded: bool,
        result: &mut IntegResult,
    ) -> Result<(), Status> {
        let dx: f64 = transform[1] * self.nodes.at(i).unwrap();
        result.number_of_points += 1;
        let left: f64 = finite_value(f, transform[0] + dx)?;
        result.number_of_points += 1;
        let right: f64 = finite_value(f, transform[0] - dx)?;
        self.left[i] = left;
        self.right[i] = right;
        if embedded {
            self.gauss_sum += (left + right) * self.gauss.at(i / 2).unwrap();
        }
        self.kronrod_sum += (left + right) * self.weights.at(i).unwrap();
        self.absolute_sum += (left.abs() + right.abs()) * self.weights.at(i).unwrap();
        Ok(())
    }
    fn variability(&self) -> f64 {
        let mean: f64 = 0.5 * self.kronrod_sum;
        let mut variability: f64 =
            (self.center - mean).abs() * self.weights.at(self.left.len() - 1).unwrap();
        for i in 0..self.left.len() - 1 {
            variability += self.weights.at(i).unwrap()
                * ((self.left[i] - mean).abs() + (self.right[i] - mean).abs());
        }
        variability
    }
    fn finish(self, half: f64, mut result: IntegResult) -> IntegResult {
        let variability: f64 = self.variability() * half;
        let absolute: f64 = self.absolute_sum * half;
        let kronrod: f64 = self.kronrod_sum * half;
        let gauss: f64 = self.gauss_sum * half;
        if ![variability, absolute, kronrod, gauss]
            .iter()
            .all(|v| v.is_finite())
        {
            result.status = Status::NumericalError;
            return result;
        }
        let error: f64 = roundoff_error((kronrod - gauss).abs(), variability, absolute);
        result.status = Status::Ok;
        result.value = Some(kronrod);
        result.absolute_error = Some(error);
        if kronrod.abs() > f64::EPSILON {
            result.relative_error = Some(error / kronrod.abs());
        }
        result.iterations = 1;
        result
    }
}
fn roundoff_error(mut error: f64, variability: f64, absolute: f64) -> f64 {
    if variability != 0.0 && error != 0.0 {
        let scale: f64 = (200.0 * error / variability).powf(1.5);
        error = variability * scale.min(1.0);
    }
    if absolute > f64::MIN_POSITIVE / (50.0 * f64::EPSILON) {
        error = error.max(50.0 * f64::EPSILON * absolute);
    }
    error
}
