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

use rustycad_tkernel::precision;
/// Native solver constant; `NEWTON_FTOL_SQ`.
pub const NEWTON_FTOL_SQ: f64 = 1.0e-32;
/// Native solver constant; `NEWTON_STEP_TOL_FACTOR`.
pub const NEWTON_STEP_TOL_FACTOR: f64 = 1.0e-16;
/// Native solver constant; `NEWTON_MAX_ITER`.
pub const NEWTON_MAX_ITER: usize = 100;
/// Native solver constant; `NEWTON2D_SINGULAR_DET`.
pub const NEWTON2D_SINGULAR_DET: f64 = 1.0e-25;
/// Native solver constant; `NEWTON2D_CRITICAL_GRAD_SQ`.
pub const NEWTON2D_CRITICAL_GRAD_SQ: f64 = 1.0e-60;
/// Native solver constant; `NEWTON2D_MAX_STEP_RATIO`.
pub const NEWTON2D_MAX_STEP_RATIO: f64 = 0.5;
/// Native solver constant; `NEWTON2D_DOMAIN_EXT`.
pub const NEWTON2D_DOMAIN_EXT: f64 = 1.0e-4;
/// Native solver constant; `NEWTON2D_STAGNATION_RATIO`.
pub const NEWTON2D_STAGNATION_RATIO: f64 = 0.999;
/// Native solver constant; `NEWTON2D_STAGNATION_COUNT`.
pub const NEWTON2D_STAGNATION_COUNT: usize = 3;
/// Native solver constant; `NEWTON2D_LINE_SEARCH_MAX`.
pub const NEWTON2D_LINE_SEARCH_MAX: usize = 8;
/// Native solver constant; `NEWTON2D_SKIP_LINESEARCH_SQ`.
pub const NEWTON2D_SKIP_LINESEARCH_SQ: f64 = 0.01;
/// Native solver constant; `NEWTON2D_STAGNATION_RELAX`.
pub const NEWTON2D_STAGNATION_RELAX: f64 = 10.0;
/// Native solver constant; `NEWTON2D_BACKTRACK_TRIGGER`.
pub const NEWTON2D_BACKTRACK_TRIGGER: f64 = 1.5;
/// Native solver constant; `NEWTON2D_BACKTRACK_ACCEPT`.
pub const NEWTON2D_BACKTRACK_ACCEPT: f64 = 1.2;
/// Native solver constant; `NEWTON2D_MAXITER_RELAX`.
pub const NEWTON2D_MAXITER_RELAX: f64 = 10.0;
/// Native solver constant; `HESSIAN_DEGENERACY_REL`.
pub const HESSIAN_DEGENERACY_REL: f64 = 1.0e-8;
/// Native solver constant; `HESSIAN_DEGENERACY_ABS`.
pub const HESSIAN_DEGENERACY_ABS: f64 = 1.0e-20;
/// Native solver constant; `ARMIJO_C1`.
pub const ARMIJO_C1: f64 = 1.0e-4;
/// Shared iterative solver settings; `Config::default()`.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Iteration limit; `config.max_iterations`.
    pub max_iterations: u32,
    /// General tolerance; `config.tolerance`.
    pub tolerance: f64,
    /// Absolute parameter tolerance; `config.x_tolerance`.
    pub x_tolerance: f64,
    /// Absolute functional tolerance; `config.f_tolerance`.
    pub f_tolerance: f64,
    /// Relative tolerance; `config.relative_tolerance`.
    pub relative_tolerance: f64,
    /// Minimum representable solver step; `config.step_min`.
    pub step_min: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self::new(1e-10, 100)
    }
}
impl Config {
    /// Sets all tolerances together; `Config::new(1e-8,100)`.
    pub fn new(tolerance: f64, max_iterations: u32) -> Self {
        Self {
            max_iterations,
            tolerance,
            x_tolerance: tolerance,
            f_tolerance: tolerance,
            relative_tolerance: tolerance,
            // Precision::Computational() is the native default StepMin.
            step_min: precision::computational(),
        }
    }
    /// Validates every native setting; `config.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.max_iterations > 0
            && [
                self.tolerance,
                self.x_tolerance,
                self.f_tolerance,
                self.relative_tolerance,
            ]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.0)
            && self.step_min.is_finite()
            && self.step_min > 0.0
    }
}
/// Iteration settings and scalar bounds; `BoundedConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct BoundedConfig {
    /// Common settings; `config.base`.
    pub base: Config,
    /// Lower parameter; `config.lower_bound`.
    pub lower_bound: f64,
    /// Upper parameter; `config.upper_bound`.
    pub upper_bound: f64,
}
impl Default for BoundedConfig {
    fn default() -> Self {
        Self::new(-precision::infinite(), precision::infinite(), 1e-10, 100)
    }
}
impl BoundedConfig {
    /// Keeps bounds verbatim; `BoundedConfig::new(0.0,1.0,1e-8,100)`.
    pub fn new(lower_bound: f64, upper_bound: f64, tolerance: f64, limit: u32) -> Self {
        Self {
            base: Config::new(tolerance, limit),
            lower_bound,
            upper_bound,
        }
    }
    /// Requires finite ordered bounds and valid base configuration; `config.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.base.is_valid()
            && self.lower_bound.is_finite()
            && self.upper_bound.is_finite()
            && self.lower_bound <= self.upper_bound
    }
}
/// Optional bounds for multiple-variable solvers; `NDimConfig::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NDimConfig {
    /// Common settings; `config.base`.
    pub base: Config,
    /// Whether bounds apply; `config.use_bounds`.
    pub use_bounds: bool,
}
impl NDimConfig {
    /// Builds common settings and bound policy; `NDimConfig::new(1e-8,100,true)`.
    pub fn new(tolerance: f64, limit: u32, use_bounds: bool) -> Self {
        Self {
            base: Config::new(tolerance, limit),
            use_bounds,
        }
    }
    /// Uses native inherited validation; `config.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.base.is_valid()
    }
}
/// Adaptive quadrature configuration; `IntegConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct IntegConfig {
    /// Initial point count; `config.initial_order`.
    pub initial_order: u32,
    /// Maximum point count; `config.max_order`.
    pub max_order: u32,
    /// Maximum adaptive iterations; `config.max_iterations`.
    pub max_iterations: u32,
    /// Relative tolerance; `config.tolerance`.
    pub tolerance: f64,
}
impl Default for IntegConfig {
    fn default() -> Self {
        Self::new(1e-10, 100)
    }
}
impl IntegConfig {
    /// Uses native point defaults 15 and 61; `IntegConfig::new(1e-8,100)`.
    pub fn new(tolerance: f64, max_iterations: u32) -> Self {
        Self {
            initial_order: 15,
            max_order: 61,
            max_iterations,
            tolerance,
        }
    }
    /// Validates order, limits and tolerance; `config.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.initial_order > 0
            && self.max_order >= self.initial_order
            && self.max_iterations > 0
            && self.tolerance.is_finite()
            && self.tolerance >= 0.0
    }
}
/// Linear solver thresholds; `LinConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct LinConfig {
    /// Singularity threshold; `config.singularity_tolerance`.
    pub singularity_tolerance: f64,
    /// Whether pivoting applies; `config.use_pivoting`.
    pub use_pivoting: bool,
}
impl Default for LinConfig {
    fn default() -> Self {
        Self {
            singularity_tolerance: 1e-15,
            use_pivoting: true,
        }
    }
}
impl LinConfig {
    /// Requires a finite nonnegative singularity tolerance; `config.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.singularity_tolerance.is_finite() && self.singularity_tolerance >= 0.0
    }
}
