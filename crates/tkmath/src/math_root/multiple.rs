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

//! Multiple root search preserving MathRoot_MultipleUtils.hxx evaluation order.
use super::utils;
use crate::math::{Function, FunctionWithDerivative, Vector};
use crate::math_utils::Status;
use std::cell::RefCell;
/// Native multi-root configuration; `MultipleConfig::default()`.
#[derive(Clone, Copy)]
pub struct MultipleConfig {
    /// Initial sampling request, doubled internally; `config.nb_samples`.
    pub nb_samples: usize,
    /// Root-position tolerance; `config.x_tolerance`.
    pub x_tolerance: f64,
    /// Function residual tolerance; `config.f_tolerance`.
    pub f_tolerance: f64,
    /// All-null classification threshold; `config.null_tolerance`.
    pub null_tolerance: f64,
    /// Brent iterations per refinement; `config.max_iterations`.
    pub max_iterations: u32,
    /// Requested function level; `config.offset`.
    pub offset: f64,
}
impl Default for MultipleConfig {
    fn default() -> Self {
        Self {
            nb_samples: 100,
            x_tolerance: 1e-10,
            f_tolerance: 1e-10,
            null_tolerance: 1e-12,
            max_iterations: 100,
            offset: 0.0,
        }
    }
}
/// Found roots retain native insertion order on early failure; `MultipleResult::default()`.
#[derive(Default)]
pub struct MultipleResult {
    /// Native completion status; `result.status`.
    pub status: Status,
    /// Total Brent iterations; `result.iterations`.
    pub iterations: u32,
    /// Root locations; `result.roots`.
    pub roots: Vec<f64>,
    /// Original function values at each root; `result.values`.
    pub values: Vec<f64>,
    /// All samples satisfy null tolerance; `result.is_all_null`.
    pub is_all_null: bool,
}
impl MultipleResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    /// Counts stored roots; `result.nb_roots()`.
    pub fn nb_roots(&self) -> usize {
        self.roots.len()
    }
}
impl From<&MultipleResult> for bool {
    fn from(result: &MultipleResult) -> bool {
        result.is_done()
    }
}
impl std::ops::Index<usize> for MultipleResult {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        &self.roots[index]
    }
}
/// Finds crossings and sampled zeros (`FindAllRoots`): wires the native Value-only adapters
/// into [`utils::find_all_roots_impl`]; `find_all_roots(&mut function,0.0,1.0,&MultipleConfig::default())`.
pub fn find_all_roots(
    function: &mut impl Function,
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
) -> MultipleResult {
    if !utils::is_valid_multiple_input(lower, upper, config) {
        return MultipleResult {
            status: Status::InvalidInput,
            ..Default::default()
        };
    }
    let count: usize = utils::multiple_sample_count(config.nb_samples);
    let samples: RefCell<Vector<'static>> = RefCell::new(Vector::with_size(count + 1));
    let shared = RefCell::new(function);
    let mut wrapper = utils::MultipleBrentValueWrapper::new(&shared, config.offset);
    utils::find_all_roots_impl(
        lower,
        upper,
        config,
        utils::MultipleSampleValueFn::new(&shared, &samples, config.offset),
        utils::MultipleGetValueFn::new(&samples),
        &mut wrapper,
        utils::MultipleGetRootValueFn::new(&shared),
        utils::MultipleNoExtraHandler,
    )
}
/// Uses the native convenience sampling count; `find_all_roots_samples(&mut function,0.0,1.0,50)`.
pub fn find_all_roots_samples(
    function: &mut impl Function,
    lower: f64,
    upper: f64,
    samples: usize,
) -> MultipleResult {
    find_all_roots(
        function,
        lower,
        upper,
        &MultipleConfig {
            nb_samples: samples,
            ..Default::default()
        },
    )
}
/// Includes native derivative extrema heuristics (`FindAllRootsWithDerivative`);
/// `find_all_roots_with_derivative(&mut function,0.0,1.0,&MultipleConfig::default())`.
pub fn find_all_roots_with_derivative(
    function: &mut impl FunctionWithDerivative,
    lower: f64,
    upper: f64,
    config: &MultipleConfig,
) -> MultipleResult {
    utils::find_all_roots_with_derivative_impl(function, lower, upper, config)
}
