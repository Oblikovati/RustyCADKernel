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

//! Owned particle vectors and native first-tie distance selection.
use rustycad_tkernel::standard::Failure;
/// A particle's mutable state; access with `pool.particle_mut(1)?`.
pub struct PsoParticle {
    /// Current position, indexed from zero; `particle.position[0]`.
    pub position: Vec<f64>,
    /// Current velocity; `particle.velocity[0]`.
    pub velocity: Vec<f64>,
    /// Best recorded position; `particle.best_position[0]`.
    pub best_position: Vec<f64>,
    /// Current objective, initially RealLast; `particle.distance`.
    pub distance: f64,
    /// Best objective, initially RealLast; `particle.best_distance`.
    pub best_distance: f64,
}
impl PsoParticle {
    fn new(dimension: usize) -> Self {
        Self {
            position: vec![0.0; dimension],
            velocity: vec![0.0; dimension],
            best_position: vec![0.0; dimension],
            distance: f64::MAX,
            best_distance: f64::MAX,
        }
    }
}
/// One-based particle pool with zeroed vectors; `PsoParticlesPool::new(32,2)?`.
pub struct PsoParticlesPool {
    pub(super) particles: Vec<PsoParticle>,
}
impl PsoParticlesPool {
    /// Allocates initialized native particle state; `PsoParticlesPool::new(32,2)?`.
    pub fn new(count: i32, dimension: i32) -> Result<Self, Failure> {
        if count < 1 || dimension < 1 {
            return Err(Failure::out_of_range(&format!(
                "pool {count} x {dimension}, expected positive dimensions"
            )));
        }
        Ok(Self {
            particles: (0..count)
                .map(|_| PsoParticle::new(dimension as usize))
                .collect(),
        })
    }
    /// Reads a one-based particle; `pool.particle(1)?`.
    pub fn particle(&self, index: i32) -> Result<&PsoParticle, Failure> {
        self.particles
            .get(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "particle {index}, expected 1..={}",
                    self.particles.len()
                ))
            })
    }
    /// Mutates a one-based particle; `pool.particle_mut(1)?`.
    pub fn particle_mut(&mut self, index: i32) -> Result<&mut PsoParticle, Failure> {
        let count: usize = self.particles.len();
        self.particles
            .get_mut(index.wrapping_sub(1) as usize)
            .ok_or_else(|| {
                Failure::out_of_range(&format!("particle {index}, expected 1..={count}"))
            })
    }
    pub(super) fn best_index(&self) -> usize {
        self.extreme(false)
    }
    pub(super) fn worst_index(&self) -> usize {
        self.extreme(true)
    }
    fn extreme(&self, worst: bool) -> usize {
        let mut chosen: usize = 0;
        for index in 1..self.particles.len() {
            let replace: bool = if worst {
                self.particles[chosen].distance < self.particles[index].distance
            } else {
                self.particles[index].distance < self.particles[chosen].distance
            };
            if replace {
                chosen = index;
            }
        }
        chosen
    }
    /// Returns the first particle with minimal distance; `pool.best_particle()`.
    pub fn best_particle(&mut self) -> &mut PsoParticle {
        let index: usize = self.best_index();
        &mut self.particles[index]
    }
    /// Returns the first particle with maximal distance; `pool.worst_particle()`.
    pub fn worst_particle(&mut self) -> &mut PsoParticle {
        let index: usize = self.worst_index();
        &mut self.particles[index]
    }
}
