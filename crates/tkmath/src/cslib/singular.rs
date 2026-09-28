// Copyright (c) 1991-1999 Matra Datavision
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

//! The higher-order CSLib::Normal overload for singular points.
mod sign_analysis;

use super::derivatives::derivative_at;
use super::{NormalResult, NormalStatus};
use crate::gp::{Direction3, Vector3};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Failure;
use sign_analysis::{Domain, polynomial_sign};

/// Angular tolerance for parallel/opposite derivative detection (radians).
const PARALLEL_ANGULAR_TOL: f64 = 1e-6;

/// The evaluated parameter and the surface bounds that pick the angular domain.
///
/// `ParametricPosition { u: 0.0, v: 0.5, u_min: 0.0, u_max: 1.0, v_min: 0.0, v_max: 1.0 }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParametricPosition {
    /// Current U parameter.
    pub u: f64,
    /// Current V parameter.
    pub v: f64,
    /// First U bound.
    pub u_min: f64,
    /// Last U bound.
    pub u_max: f64,
    /// First V bound.
    pub v_min: f64,
    /// Last V bound.
    pub v_max: f64,
}

/// Result of the singular-point normal: status, optional normal and the orders used.
///
/// `order_u`/`order_v` are always written, as the native reference outputs are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SingularNormal {
    /// Status and normal.
    pub result: NormalResult<NormalStatus>,
    /// U order of the first non-null derivative of N.
    pub order_u: i32,
    /// V order of the first non-null derivative of N.
    pub order_v: i32,
}

/// The first non-null derivative of N found by the native diagonal scan.
struct FirstDerivative {
    found: bool,
    order: i32,
    order_u: i32,
    order_v: i32,
    vector: Vector3,
}

/// CSLib::Normal(MaxOrder, DerNUV, SinTol, U, V, Umin, Umax, Vmin, Vmax, ...).
///
/// Finds the first order k0 whose derivatives of `N = D1U ^ D1V` are non-null,
/// then decides from the sign of [`NormalPolyDef`](super::NormalPolyDef) over
/// the angular domain allowed at `position` whether the normal is unique.
/// Null reference vectors (reachable only with `sin_tol <= 0`) fail like
/// gp_Vec::Normalized/Angle. `normal_at_singularity(2, &der_nuv, 1e-9, position)?`.
pub fn normal_at_singularity(
    max_order: i32,
    der_nuv: &Array2<'_, Vector3>,
    sin_tol: f64,
    position: ParametricPosition,
) -> Result<SingularNormal, Failure> {
    let first: FirstDerivative = first_non_null(max_order, der_nuv, sin_tol)?;
    let result: NormalResult<NormalStatus> = classify(&first, der_nuv, sin_tol, position)?;
    Ok(SingularNormal {
        result,
        order_u: first.order_u,
        order_v: first.order_v,
    })
}

// Scans orders 0..=max_order, each from (order,0) down to (0,order), for |d| >= sin_tol.
fn first_non_null(
    max_order: i32,
    der_nuv: &Array2<'_, Vector3>,
    sin_tol: f64,
) -> Result<FirstDerivative, Failure> {
    let mut order: i32 = -1;
    let mut u_index: i32 = 0;
    let mut found: bool = false;
    let mut vector: Vector3 = Vector3::new(0.0, 0.0, 0.0);
    while !found && order < max_order {
        order += 1;
        u_index = order;
        while u_index >= 0 && !found {
            vector = derivative_at(der_nuv, u_index, order - u_index)?;
            found = vector.magnitude() >= sin_tol;
            u_index -= 1;
        }
    }
    let order_u: i32 = u_index + 1;
    Ok(FirstDerivative {
        found,
        order,
        order_u,
        order_v: order - order_u,
        vector,
    })
}

fn classify(
    first: &FirstDerivative,
    der_nuv: &Array2<'_, Vector3>,
    sin_tol: f64,
    position: ParametricPosition,
) -> Result<NormalResult<NormalStatus>, Failure> {
    if !first.found {
        return Ok(NormalResult::untouched(NormalStatus::Singular));
    }
    if first.order == 0 {
        return defined_along(first.vector, 1.0);
    }
    let vk0: Vector3 = derivative_at(der_nuv, first.order_u, first.order_v)?;
    let Some(ratios) = collinear_ratios(der_nuv, first.order, vk0, sin_tol)? else {
        return defined_along(first.vector, 1.0);
    };
    let sign: Option<f64> = polynomial_sign(first.order, &ratios, Domain::at(position))?;
    match sign {
        None => Ok(NormalResult::untouched(NormalStatus::InfinityOfSolutions)),
        Some(value) => defined_along(vk0, if value > 0.0 { 1.0 } else { -1.0 }),
    }
}

// Native writes gp_Dir(sign * V.Normalized()): normalize, scale, then normalize again.
fn defined_along(vector: Vector3, sign: f64) -> Result<NormalResult<NormalStatus>, Failure> {
    let direction: Direction3 = Direction3::from_vector(vector.normalized()? * sign)?;
    Ok(NormalResult::written(NormalStatus::Defined, direction))
}

/// Signed ratios `lambda_i = +-|d(i,k0-i)| / |Vk0|`, or `None` when some
/// non-null derivative of order k0 is not parallel to Vk0 (normal defined).
fn collinear_ratios(
    der_nuv: &Array2<'_, Vector3>,
    order: i32,
    vk0: Vector3,
    sin_tol: f64,
) -> Result<Option<Vec<f64>>, Failure> {
    let mut ratios: Vec<f64> = Vec::with_capacity(order as usize + 1);
    for index in 0..=order {
        let derivative: Vector3 = derivative_at(der_nuv, index, order - index)?;
        match collinear_ratio(derivative, vk0, sin_tol)? {
            Some(ratio) => ratios.push(ratio),
            None => return Ok(None),
        }
    }
    Ok(Some(ratios))
}

fn collinear_ratio(
    derivative: Vector3,
    vk0: Vector3,
    sin_tol: f64,
) -> Result<Option<f64>, Failure> {
    if derivative.magnitude() <= sin_tol {
        return Ok(Some(0.0));
    }
    if !derivative.is_parallel(vk0, PARALLEL_ANGULAR_TOL)? {
        return Ok(None);
    }
    let ratio: f64 = derivative.magnitude() / vk0.magnitude();
    if derivative.is_opposite(vk0, PARALLEL_ANGULAR_TOL)? {
        return Ok(Some(-ratio));
    }
    Ok(Some(ratio))
}
