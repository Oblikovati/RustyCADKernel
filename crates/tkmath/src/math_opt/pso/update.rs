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
use super::{
    swarm::{Particle, Swarm},
    *,
};
use crate::math_utils::clamp;
impl Swarm<'_> {
    pub(super) fn update_all(&mut self, function: &mut impl MultipleVarFunction, iteration: u32) {
        let omega: f64 = self.config.inertia(iteration);
        for index in 0..self.particles.len() {
            self.update_velocity(index, omega);
            self.update_position(index);
            let particle: &mut Particle = &mut self.particles[index];
            let mut value: f64 = 0.0;
            let valid: bool = function.value(&particle.x, &mut value);
            self.stats.function_evals += 1;
            if valid && particle.best_value.is_none_or(|old| value < old) {
                particle.best_value = Some(value);
                particle.best.initialized(&particle.x).unwrap();
            }
            self.improve_global(index);
        }
    }
    fn update_velocity(&mut self, index: usize, omega: f64) {
        let p: &mut Particle = &mut self.particles[index];
        for i in 0..self.range.len() {
            let r1: f64 = self.rng.next_real();
            let r2: f64 = self.rng.next_real();
            let v: f64 = omega * p.velocity[i]
                + self.config.phi_personal * r1 * (p.best.at(i).unwrap() - p.x.at(i).unwrap())
                + self.config.phi_global * r2 * (self.best.at(i).unwrap() - p.x.at(i).unwrap());
            p.velocity[i] = clamp(v, -self.velocity_max[i], self.velocity_max[i]);
        }
    }
    fn update_position(&mut self, index: usize) {
        let p: &mut Particle = &mut self.particles[index];
        for i in 0..self.range.len() {
            let x: f64 = p.x.at(i).unwrap() + p.velocity[i];
            let lo: f64 = *self.bounds.lower.at(i).unwrap();
            let hi: f64 = *self.bounds.upper.at(i).unwrap();
            let outside: bool = x < lo || x > hi;
            if outside {
                self.stats.boundary_corrections += 1;
            }
            *p.x.change_at(i).unwrap() = if outside {
                correct(
                    x,
                    &mut p.velocity[i],
                    lo,
                    hi,
                    self.range[i],
                    self.config.boundary_mode,
                )
            } else {
                x
            };
        }
    }
}
fn correct(
    mut x: f64,
    velocity: &mut f64,
    lo: f64,
    hi: f64,
    range: f64,
    mode: PSOBoundaryMode,
) -> f64 {
    match mode {
        PSOBoundaryMode::Clamp => {
            x = clamp(x, lo, hi);
            *velocity *= -0.5;
        }
        PSOBoundaryMode::Reflect => {
            x = if x < lo { 2.0 * lo - x } else { 2.0 * hi - x };
            x = clamp(x, lo, hi);
            *velocity *= -0.5;
        }
        PSOBoundaryMode::Wrap => {
            x = (x - lo) % range;
            if x < 0.0 {
                x += range;
            }
            x += lo;
        }
    }
    x
}
