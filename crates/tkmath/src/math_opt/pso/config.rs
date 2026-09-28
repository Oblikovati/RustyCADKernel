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

//! MathOpt_PSO.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::Vector;
use crate::math_utils::Config;
/// Native seed selection policy; `PSOInitMode::RandomOnly`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PSOInitMode {
    #[default]
    /// Native RandomOnly policy.
    RandomOnly,
    /// Native SeededOnly policy.
    SeededOnly,
    /// Native SeededPlusRandom policy.
    SeededPlusRandom,
}
/// Native boundary correction policy; `PSOBoundaryMode::Clamp`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PSOBoundaryMode {
    #[default]
    /// Native Clamp policy.
    Clamp,
    /// Native Reflect policy.
    Reflect,
    /// Native Wrap policy.
    Wrap,
}
/// Native inertia policy; `PSOInertiaSchedule::Constant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PSOInertiaSchedule {
    #[default]
    /// Native Constant policy.
    Constant,
    /// Native LinearDecay policy.
    LinearDecay,
}
/// Seeds retain advisory fitness, but always evaluate the callback; `PSOSeedParticle::new(&position)`.
pub struct PSOSeedParticle {
    /// Native position field; `value.position`.
    pub position: Vector<'static>,
    /// Native value field; `value.value`.
    pub value: Option<f64>,
    /// Native velocity field; `value.velocity`.
    pub velocity: Option<Vector<'static>>,
}
impl PSOSeedParticle {
    /// Copies the input position with its bounds; `PSOSeedParticle::new(&position)`.
    pub fn new(position: &Vector<'_>) -> Self {
        {
            let mut copied: Vector<'static> =
                Vector::with_bounds(position.lower(), position.upper()).unwrap();
            copied.initialized(position).unwrap();
            Self {
                position: copied,
                value: None,
                velocity: None,
            }
        }
    }
    /// Stores an advisory fitness; `PSOSeedParticle::with_value(&position, 1.0)`.
    pub fn with_value(position: &Vector<'_>, value: f64) -> Self {
        Self {
            value: Some(value),
            ..Self::new(position)
        }
    }
    pub(super) fn valid(&self, n: usize) -> bool {
        self.position.length() as usize == n
            && self.position.values().iter().all(|v| v.is_finite())
            && self.value.is_none_or(f64::is_finite)
            && self.velocity.as_ref().is_none_or(|v| {
                v.length() as usize == n && v.values().iter().all(|x| x.is_finite())
            })
    }
}
/// Native execution counters; `PSOStats::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PSOStats {
    /// Native function_evals field; `value.function_evals`.
    pub function_evals: usize,
    /// Native iterations field; `value.iterations`.
    pub iterations: u32,
    /// Native boundary_corrections field; `value.boundary_corrections`.
    pub boundary_corrections: usize,

    /// Native stagnation_events field; `value.stagnation_events`.
    pub stagnation_events: usize,
    /// Native restarts field; `value.restarts`.
    pub restarts: usize,
    /// Native initial_best field; `value.initial_best`.
    pub initial_best: Option<f64>,
    /// Native final_best field; `value.final_best`.
    pub final_best: Option<f64>,
}
/// Native particle swarm settings; `PSOConfig::default()`.
#[derive(Clone, Copy, Debug)]
pub struct PSOConfig {
    /// Native solver field; `value.solver`.
    pub solver: Config,
    /// Native use_bounds field; `value.use_bounds`.
    pub use_bounds: bool,
    /// Native particles field; `value.particles`.
    pub particles: usize,
    /// Native omega field; `value.omega`.
    pub omega: f64,

    /// Native phi_personal field; `value.phi_personal`.
    pub phi_personal: f64,
    /// Native phi_global field; `value.phi_global`.
    pub phi_global: f64,
    /// Native velocity_clamp field; `value.velocity_clamp`.
    pub velocity_clamp: f64,
    /// Native seed field; `value.seed`.
    pub seed: u32,

    /// Native init_mode field; `value.init_mode`.
    pub init_mode: PSOInitMode,
    /// Native boundary_mode field; `value.boundary_mode`.
    pub boundary_mode: PSOBoundaryMode,
    /// Native inertia_schedule field; `value.inertia_schedule`.
    pub inertia_schedule: PSOInertiaSchedule,

    /// Native omega_min field; `value.omega_min`.
    pub omega_min: f64,
    /// Native min_iterations field; `value.min_iterations`.
    pub min_iterations: u32,
    /// Native target_value field; `value.target_value`.
    pub target_value: Option<f64>,

    /// Native no_improve_tol field; `value.no_improve_tol`.
    pub no_improve_tol: f64,
    /// Native no_improve_iters field; `value.no_improve_iters`.
    pub no_improve_iters: u32,
    /// Native restart_fraction field; `value.restart_fraction`.
    pub restart_fraction: f64,

    /// Native max_restarts field; `value.max_restarts`.
    pub max_restarts: u32,
    /// Native polish_budget_per_dim field; `value.polish_budget_per_dim`.
    pub polish_budget_per_dim: u32,
}
impl Default for PSOConfig {
    fn default() -> Self {
        Self::new(40, 100, 1e-8)
    }
}
impl PSOConfig {
    /// Sets native population, limit and tolerances; `PSOConfig::new(40,100,1e-8)`.
    pub fn new(particles: usize, limit: u32, tolerance: f64) -> Self {
        Self {
            solver: Config::new(tolerance, limit),
            use_bounds: true,
            particles,
            omega: 0.7,
            phi_personal: 1.5,
            phi_global: 1.5,
            velocity_clamp: 0.5,
            seed: 6,
            init_mode: PSOInitMode::RandomOnly,
            boundary_mode: PSOBoundaryMode::Clamp,
            inertia_schedule: PSOInertiaSchedule::Constant,
            omega_min: 0.4,
            min_iterations: 0,
            target_value: None,
            no_improve_tol: 0.0,
            no_improve_iters: 10,
            restart_fraction: 0.0,
            max_restarts: 0,
            polish_budget_per_dim: 50,
        }
    }
    pub(super) fn valid(&self) -> bool {
        super::super::utils::is_valid_config(&self.solver)
            && self.particles > 0
            && self.omega.is_finite()
            && self.omega_min.is_finite()
            && [
                self.phi_personal,
                self.phi_global,
                self.velocity_clamp,
                self.no_improve_tol,
                self.restart_fraction,
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.0)
            && self.min_iterations <= self.solver.max_iterations
            && self.no_improve_iters > 0
            && self.restart_fraction <= 1.0
            && self.target_value.is_none_or(f64::is_finite)
    }
    pub(super) fn inertia(&self, iteration: u32) -> f64 {
        if self.inertia_schedule == PSOInertiaSchedule::LinearDecay {
            self.omega
                - (self.omega - self.omega_min) * iteration as f64
                    / self.solver.max_iterations as f64
        } else {
            self.omega
        }
    }
}
