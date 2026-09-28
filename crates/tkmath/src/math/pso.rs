// Created on: 2014-07-18
// Created by: Alexander Malyshev
// Copyright (c) 2014-2014 OPEN CASCADE SAS
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

//! Particle swarm optimization with native grid ordering and fixed-seed velocities.
use super::{BullardGenerator, MultipleVarFunction, PsoParticle, PsoParticlesPool, Vector};
use rustycad_tkernel::standard::Failure;
mod grid;
mod iterate;
mod velocity;
/// Objective borrowed for the optimizer's lifetime; `Pso::new(&mut f,&lower,&upper,&steps)?`.
pub struct Pso<'a, F: MultipleVarFunction> {
    function: &'a mut F,
    lower: Vector<'static>,
    upper: Vector<'static>,
    steps: Vector<'static>,
    dimension: i32,
    count: i32,
    limit: i32,
}
impl<'a, F: MultipleVarFunction> Pso<'a, F> {
    /// Uses 32 particles and 100 constructor iterations; `Pso::new(&mut f,&lower,&upper,&steps)?`.
    pub fn new(
        function: &'a mut F,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        steps: &Vector<'_>,
    ) -> Result<Self, Failure> {
        Self::with_options(function, lower, upper, steps, 32, 100)
    }
    /// Copies the bounds and grid spacing; `Pso::with_options(&mut f,&lower,&upper,&steps,20,50)?`.
    pub fn with_options(
        function: &'a mut F,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        steps: &Vector<'_>,
        count: i32,
        limit: i32,
    ) -> Result<Self, Failure> {
        let dimension: i32 = function.variables();
        let mut result: Self = Self {
            function,
            lower: Vector::with_bounds(1, dimension)?,
            upper: Vector::with_bounds(1, dimension)?,
            steps: Vector::with_bounds(1, dimension)?,
            dimension,
            count,
            limit,
        };
        result.lower.initialized(lower)?;
        result.upper.initialized(upper)?;
        result.steps.initialized(steps)?;
        Ok(result)
    }
    fn interior(&self) -> Result<(Vector<'static>, Vector<'static>), Failure> {
        let mut lower: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        let mut upper: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for i in 1..=self.dimension {
            let margin: f64 = (self.upper[i] - self.lower[i]) * (1.0 / 1e4);
            lower[i] = self.lower[i] + margin;
            upper[i] = self.upper[i] - margin;
        }
        Ok((lower, upper))
    }
    /// Generates particles and runs 100 iterations, independently of constructor limit; `pso.perform(&steps,&mut value,&mut point)?`.
    pub fn perform(
        &mut self,
        steps: &Vector<'_>,
        value: &mut f64,
        point: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        self.perform_with_iterations(steps, value, point, 100)
    }
    /// Replaces grid steps and iteration count on each call; `pso.perform_with_iterations(&steps,&mut value,&mut point,30)?`.
    pub fn perform_with_iterations(
        &mut self,
        steps: &Vector<'_>,
        value: &mut f64,
        point: &mut Vector<'_>,
        limit: i32,
    ) -> Result<(), Failure> {
        self.limit = limit;
        self.steps.initialized(steps)?;
        let mut pool: PsoParticlesPool = PsoParticlesPool::new(self.count, self.dimension)?;
        self.populate(&mut pool)?;
        self.perform_particles_with_iterations(&mut pool, self.count, value, point, limit)
    }
    /// Runs an existing pool with the native default iteration count; `pso.perform_particles(&mut pool,n,&mut value,&mut point)?`.
    pub fn perform_particles(
        &mut self,
        pool: &mut PsoParticlesPool,
        count: i32,
        value: &mut f64,
        point: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        self.perform_particles_with_iterations(pool, count, value, point, 100)
    }
}
