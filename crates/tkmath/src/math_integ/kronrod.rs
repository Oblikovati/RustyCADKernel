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
use super::{
    adaptive::{Rule, converged, integrate_adaptive},
    gauss::valid_interval,
    kronrod_rule,
};
use crate::{
    math::Function,
    math_utils::{IntegConfig, IntegResult, Status},
};
/// Native Kronrod options; `KronrodConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct KronrodConfig {
    /// Shared integration options; `config.integration.tolerance`.
    pub integration: IntegConfig,
    /// Embedded Gauss order; `config.nb_gauss_points`.
    pub nb_gauss_points: usize,
    /// Enable subdivision; `config.adaptive`.
    pub adaptive: bool,
}
impl Default for KronrodConfig {
    fn default() -> Self {
        Self::new(1e-10, 100)
    }
}
impl KronrodConfig {
    /// Retains native order seven; `KronrodConfig::new(1e-8, 100)`.
    pub fn new(tolerance: f64, max_iterations: u32) -> Self {
        Self {
            integration: IntegConfig::new(tolerance, max_iterations),
            nb_gauss_points: 7,
            adaptive: true,
        }
    }
    fn valid(&self) -> bool {
        self.nb_gauss_points >= 1
            && self.nb_gauss_points <= 30
            && self.integration.tolerance.is_finite()
            && self.integration.tolerance > 0.0
            && self.integration.max_iterations >= 1
    }
}
/// Adaptive embedded-rule integration; `kronrod(&mut f, 0.0, 1.0, &KronrodConfig::default())`.
pub fn kronrod(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &KronrodConfig,
) -> IntegResult {
    if !valid_interval(lower, upper) || !config.valid() {
        return invalid();
    }
    if lower == upper {
        return empty();
    }
    if lower > upper {
        return reversed(kronrod(f, upper, lower, config));
    }
    if !config.adaptive {
        return kronrod_rule(f, lower, upper, config.nb_gauss_points);
    }
    integrate_adaptive(
        f,
        lower,
        upper,
        &config.integration,
        Rule::Kronrod(config.nb_gauss_points),
    )
}
pub(super) fn invalid() -> IntegResult {
    IntegResult {
        status: Status::InvalidInput,
        ..Default::default()
    }
}
pub(super) fn empty() -> IntegResult {
    IntegResult {
        status: Status::Ok,
        value: Some(0.0),
        absolute_error: Some(0.0),
        ..Default::default()
    }
}
pub(super) fn reversed(mut result: IntegResult) -> IntegResult {
    result.value = result.value.map(|value| -value);
    result
}
/// Escalates orders before adaptive fallback; `kronrod_auto(&mut f, 0.0, 1.0, 1e-10, 30)`.
pub fn kronrod_auto(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    tolerance: f64,
    max_order: usize,
) -> IntegResult {
    if !valid_interval(lower, upper)
        || !tolerance.is_finite()
        || tolerance <= 0.0
        || !(7..=30).contains(&max_order)
    {
        return invalid();
    }
    if lower == upper {
        return empty();
    }
    if lower > upper {
        return reversed(kronrod_auto(f, upper, lower, tolerance, max_order));
    }
    automatic_orders(f, lower, upper, tolerance, max_order)
}
fn automatic_orders(
    f: &mut impl Function,
    lower: f64,
    upper: f64,
    tolerance: f64,
    max_order: usize,
) -> IntegResult {
    let mut points: usize = 0;
    for order in (7..=max_order).step_by(4) {
        let mut result: IntegResult = kronrod_rule(f, lower, upper, order);
        points += result.number_of_points;
        if !result.is_done()
            || converged(
                result.absolute_error.unwrap(),
                result.value.unwrap(),
                tolerance,
            )
        {
            result.number_of_points = points;
            return result;
        }
    }
    let config: KronrodConfig = KronrodConfig::new(tolerance, 50);
    let mut result: IntegResult = kronrod(f, lower, upper, &config);
    result.number_of_points += points;
    result
}
