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

//! MathInteg_Multiple.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::gauss::valid_interval;
use crate::{
    math::{MultipleVarFunction, Vector, ordered_gauss_points_and_weights},
    math_utils::{IntegResult, Status},
};
use rustycad_tkernel::ncollection::Array1;
/// Native per-axis order cap; `MultipleConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct MultipleConfig {
    /// Maximum permitted quadrature order; `config.max_order`.
    pub max_order: usize,
}
impl Default for MultipleConfig {
    fn default() -> Self {
        Self { max_order: 61 }
    }
}
struct AxisRule {
    mid: f64,
    half: f64,
    points: Vector<'static>,
    weights: Vector<'static>,
}
/// Tensor-product Gauss integration in native axis order; `gauss_multiple(&mut f, 2, &lower, &upper, &orders, &MultipleConfig::default())`.
pub fn gauss_multiple(
    f: &mut impl MultipleVarFunction,
    dimensions: usize,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    orders: &Array1<'_, i32>,
    config: &MultipleConfig,
) -> IntegResult {
    let Some((evaluations, zero_width)) =
        validate_dimensions(dimensions, lower, upper, orders.as_slice(), config)
    else {
        return super::kronrod::invalid();
    };
    if zero_width {
        return IntegResult {
            status: Status::Ok,
            value: Some(0.0),
            ..Default::default()
        };
    }
    let Some(axes) = create_axes(lower, upper, orders.as_slice()) else {
        return super::kronrod::invalid();
    };
    let mut result: IntegResult = IntegResult::default();
    match tensor_product(f, &axes, evaluations, &mut result) {
        Ok(value) => {
            result.value = Some(value);
            result.status = Status::Ok;
            result.iterations = 1;
        }
        Err(status) => result.status = status,
    }
    result
}
fn validate_dimensions(
    dimensions: usize,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    orders: &[i32],
    config: &MultipleConfig,
) -> Option<(usize, bool)> {
    if dimensions == 0
        || lower.length() as usize != dimensions
        || upper.length() as usize != dimensions
        || orders.len() != dimensions
        || !(1..=61).contains(&config.max_order)
    {
        return None;
    }
    let mut evaluations: usize = 1;
    let mut total_nodes: usize = 0;
    let mut zero: bool = false;
    for (i, &order) in orders.iter().enumerate() {
        let a: f64 = *lower.at(i).unwrap();
        let b: f64 = *upper.at(i).unwrap();
        if !valid_interval(a, b) || order < 1 || order as usize > config.max_order {
            return None;
        }
        zero |= a == b;
        evaluations = evaluations.checked_mul(order as usize)?;
        total_nodes = total_nodes.checked_add(order as usize)?;
    }
    Some((evaluations, zero))
}
fn create_axes(lower: &Vector<'_>, upper: &Vector<'_>, orders: &[i32]) -> Option<Vec<AxisRule>> {
    orders
        .iter()
        .enumerate()
        .map(|(i, &order)| {
            let half: f64 = 0.5 * (upper.at(i).unwrap() - lower.at(i).unwrap());
            let mut points: Vector<'static> = Vector::with_size(order as usize);
            let mut weights: Vector<'static> = Vector::with_size(order as usize);
            if !ordered_gauss_points_and_weights(order, &mut points, &mut weights) {
                return None;
            }
            Some(AxisRule {
                mid: lower.at(i).unwrap() + half,
                half,
                points,
                weights,
            })
        })
        .collect()
}
fn tensor_product(
    f: &mut impl MultipleVarFunction,
    axes: &[AxisRule],
    evaluations: usize,
    result: &mut IntegResult,
) -> Result<f64, Status> {
    let mut point: Vector<'static> = Vector::with_size(axes.len());
    let mut indices: Vec<usize> = vec![0; axes.len()];
    let mut sum: f64 = 0.0;
    for _ in 0..evaluations {
        let weight: f64 = tensor_point(axes, &indices, &mut point);
        let mut value: f64 = 0.0;
        result.number_of_points += 1;
        if !f.value(&point, &mut value) {
            return Err(Status::CallbackError);
        }
        if !value.is_finite() {
            return Err(Status::NumericalError);
        }
        sum += weight * value;
        if !sum.is_finite() {
            return Err(Status::NumericalError);
        }
        next_indices(axes, &mut indices);
    }
    scale_axes(sum, axes)
}
fn tensor_point(axes: &[AxisRule], indices: &[usize], point: &mut Vector<'_>) -> f64 {
    let mut weight: f64 = 1.0;
    for (i, axis) in axes.iter().enumerate() {
        *point.change_at(i).unwrap() = axis.mid + axis.half * axis.points.at(indices[i]).unwrap();
        weight *= axis.weights.at(indices[i]).unwrap();
    }
    weight
}
fn next_indices(axes: &[AxisRule], indices: &mut [usize]) {
    for i in (0..axes.len()).rev() {
        indices[i] += 1;
        if indices[i] < axes[i].points.length() as usize {
            break;
        }
        indices[i] = 0;
    }
}
fn scale_axes(mut sum: f64, axes: &[AxisRule]) -> Result<f64, Status> {
    for axis in axes {
        sum *= axis.half;
        if !sum.is_finite() {
            return Err(Status::NumericalError);
        }
    }
    Ok(sum)
}
/// Uses a uniform order without querying callback dimensions; `gauss_multiple_uniform(&mut f, 2, &lower, &upper, 7)`.
pub fn gauss_multiple_uniform(
    f: &mut impl MultipleVarFunction,
    dimensions: usize,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    order: usize,
) -> IntegResult {
    if dimensions == 0 || !(1..=61).contains(&order) {
        return super::kronrod::invalid();
    }
    let mut orders: Array1<'static, i32> = Array1::with_size(dimensions);
    orders.as_mut_slice().fill(order as i32);
    gauss_multiple(
        f,
        dimensions,
        lower,
        upper,
        &orders,
        &MultipleConfig::default(),
    )
}
