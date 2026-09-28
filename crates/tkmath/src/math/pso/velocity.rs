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

use super::{BullardGenerator, Failure, MultipleVarFunction, Pso, PsoParticlesPool, Vector};
impl<F: MultipleVarFunction> Pso<'_, F> {
    pub(super) fn initial_velocities(
        &self,
        pool: &mut PsoParticlesPool,
        random: &mut BullardGenerator,
    ) -> Result<(), Failure> {
        for index in 1..=self.count {
            let particle: &mut super::PsoParticle = pool.particle_mut(index)?;
            for i in 0..self.dimension as usize {
                let r: f64 = random.next_real();
                particle.velocity[i] = self.steps[i as i32 + 1] * (r - 0.5) * 2.0;
            }
        }
        Ok(())
    }
    pub(super) fn restart_velocities(
        &self,
        pool: &mut PsoParticlesPool,
        lower: &Vector<'_>,
        upper: &Vector<'_>,
        random: &mut BullardGenerator,
    ) -> Result<(), Failure> {
        for index in 1..=self.count {
            let r: f64 = random.next_real();
            let particle: &mut super::PsoParticle = pool.particle_mut(index)?;
            for i in 0..self.dimension as usize {
                let velocity: f64 = if particle.position[i] == lower[i as i32 + 1] {
                    self.steps[i as i32 + 1] * r
                } else if particle.position[i] == upper[i as i32 + 1] {
                    -self.steps[i as i32 + 1] * r
                } else {
                    self.steps[i as i32 + 1] * (r - 0.5) * 2.0
                };
                particle.velocity[i] = velocity;
            }
        }
        Ok(())
    }
}
