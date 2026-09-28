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

//! MathInteg_Set.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::gauss::valid_interval;
use crate::{
    math::{FunctionSet, Vector, ordered_gauss_points_and_weights},
    math_utils::Status,
};
/// Highest Gauss order `gauss_set` uses; larger requests are clamped
/// (`MathInteg::THE_SET_GAUSS_MAX_ORDER`); `assert_eq!(SET_GAUSS_MAX_ORDER, 61)`.
pub const SET_GAUSS_MAX_ORDER: usize = 61;
/// Native vector-integral result, including callback count; `SetResult::default()`.
#[derive(Default)]
pub struct SetResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Computed component integrals on success; `result.values`.
    pub values: Option<Vector<'static>>,
    /// Validated equation count; `result.nb_equations`.
    pub nb_equations: usize,
    /// Attempted callback count; `result.nb_points`.
    pub nb_points: usize,
}
impl SetResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl From<&SetResult> for bool {
    fn from(result: &SetResult) -> bool {
        result.is_done()
    }
}
struct SetRule {
    points: Vector<'static>,
    weights: Vector<'static>,
    sums: Vector<'static>,
    point: Vector<'static>,
    left: Vector<'static>,
    right: Vector<'static>,
}
/// Integrates all components with native symmetric callback ordering; `gauss_set(&mut f, 0.0, 1.0, 7)`.
pub fn gauss_set(f: &mut impl FunctionSet, lower: f64, upper: f64, order: usize) -> SetResult {
    if !valid_interval(lower, upper) || order == 0 {
        return invalid_set();
    }
    let equations: i32 = f.equations();
    if equations <= 0 {
        return invalid_set();
    }
    let mut result: SetResult = SetResult {
        nb_equations: equations as usize,
        ..Default::default()
    };
    if lower == upper {
        result.values = Some(Vector::with_size(equations as usize));
        result.status = Status::Ok;
        return result;
    }
    let Some(mut rule) = SetRule::new(equations as usize, order.min(SET_GAUSS_MAX_ORDER)) else {
        result.status = Status::InvalidInput;
        return result;
    };
    let half: f64 = 0.5 * (upper - lower);
    match rule.integrate(f, [lower + half, half], &mut result) {
        Ok(()) => {
            result.status = Status::Ok;
            result.values = Some(rule.sums);
        }
        Err(status) => result.status = status,
    }
    result
}
fn invalid_set() -> SetResult {
    SetResult {
        status: Status::InvalidInput,
        ..Default::default()
    }
}
impl SetRule {
    fn new(equations: usize, order: usize) -> Option<Self> {
        let mut points: Vector<'static> = Vector::with_size(order);
        let mut weights: Vector<'static> = Vector::with_size(order);
        if !ordered_gauss_points_and_weights(order as i32, &mut points, &mut weights) {
            return None;
        }
        Some(Self {
            points,
            weights,
            sums: Vector::with_size(equations),
            point: Vector::with_size(1),
            left: Vector::with_size(equations),
            right: Vector::with_size(equations),
        })
    }
    fn integrate(
        &mut self,
        f: &mut impl FunctionSet,
        transform: [f64; 2],
        result: &mut SetResult,
    ) -> Result<(), Status> {
        let half_count: usize = self.points.length() as usize / 2;
        if self.points.length() % 2 != 0 {
            self.center(f, transform[0], half_count, result)?;
        }
        for i in 0..half_count {
            self.pair(f, transform, i, result)?;
        }
        for j in 0..self.sums.length() as usize {
            *self.sums.change_at(j).unwrap() *= transform[1];
            if !self.sums.at(j).unwrap().is_finite() {
                return Err(Status::NumericalError);
            }
        }
        Ok(())
    }
    fn center(
        &mut self,
        f: &mut impl FunctionSet,
        mid: f64,
        index: usize,
        result: &mut SetResult,
    ) -> Result<(), Status> {
        *self.point.change_at(0).unwrap() = mid;
        result.nb_points += 1;
        if !f.value(&self.point, &mut self.sums) {
            return Err(Status::CallbackError);
        }
        for j in 0..self.sums.length() as usize {
            if !self.sums.at(j).unwrap().is_finite() {
                return Err(Status::NumericalError);
            }
            *self.sums.change_at(j).unwrap() *= self.weights.at(index).unwrap();
        }
        Ok(())
    }
    fn pair(
        &mut self,
        f: &mut impl FunctionSet,
        transform: [f64; 2],
        i: usize,
        result: &mut SetResult,
    ) -> Result<(), Status> {
        let dx: f64 = transform[1] * self.points.at(i).unwrap();
        *self.point.change_at(0).unwrap() = transform[0] + dx;
        self.left.init(0.0);
        result.nb_points += 1;
        if !f.value(&self.point, &mut self.left) {
            return Err(Status::CallbackError);
        }
        *self.point.change_at(0).unwrap() = transform[0] - dx;
        self.right.init(0.0);
        result.nb_points += 1;
        if !f.value(&self.point, &mut self.right) {
            return Err(Status::CallbackError);
        }
        self.add_pair(*self.weights.at(i).unwrap())
    }
    fn add_pair(&mut self, weight: f64) -> Result<(), Status> {
        for j in 0..self.sums.length() as usize {
            let left: f64 = *self.left.at(j).unwrap();
            let right: f64 = *self.right.at(j).unwrap();
            if !left.is_finite() || !right.is_finite() {
                return Err(Status::NumericalError);
            }
            *self.sums.change_at(j).unwrap() += (left + right) * weight;
            if !self.sums.at(j).unwrap().is_finite() {
                return Err(Status::NumericalError);
            }
        }
        Ok(())
    }
}
/// Uses the first coordinate of arbitrarily bounded vectors; `gauss_set_bounds(&mut f, &lower, &upper, 7)`.
pub fn gauss_set_bounds(
    f: &mut impl FunctionSet,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    order: usize,
) -> SetResult {
    if lower.length() < 1 || upper.length() < 1 {
        return invalid_set();
    }
    gauss_set(f, *lower.at(0).unwrap(), *upper.at(0).unwrap(), order)
}
