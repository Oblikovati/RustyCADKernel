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
use super::{swarm::Swarm, *};
impl Swarm<'_> {
    pub(super) fn stagnated(
        &mut self,
        function: &mut impl MultipleVarFunction,
        previous: f64,
        count: &mut u32,
    ) -> bool {
        if self.stats.iterations <= self.config.min_iterations {
            return false;
        }
        let tolerance: f64 = if self.config.no_improve_tol > 0.0 {
            self.config.no_improve_tol
        } else {
            self.config.solver.tolerance
        };
        let best: f64 = self.best_value.unwrap();
        if (best - previous).abs() >= tolerance * (1.0 + best.abs()) {
            *count = 0;
            return false;
        }
        *count += 1;
        if *count < self.config.no_improve_iters {
            return false;
        }
        self.stats.stagnation_events += 1;
        if self.config.restart_fraction <= 0.0
            || (self.config.max_restarts != 0
                && self.stats.restarts >= self.config.max_restarts as usize)
        {
            return true;
        }
        self.stats.restarts += 1;
        *count = 0;
        let amount: usize = ((self.config.restart_fraction * self.particles.len() as f64).ceil()
            as usize)
            .min(self.particles.len() - 1);
        let worst: Vec<usize> = self.worst_indices();
        for index in worst.into_iter().take(amount) {
            self.random_particle(index);
            self.evaluate_initial(function, index);
        }
        false
    }
    fn worst_indices(&self) -> Vec<usize> {
        let best: usize = (0..self.particles.len())
            .filter(|i| self.particles[*i].best_value.is_some())
            .reduce(|a, b| {
                if self.particles[b].best_value.unwrap() < self.particles[a].best_value.unwrap() {
                    b
                } else {
                    a
                }
            })
            .unwrap();
        let mut indices: Vec<usize> = (0..self.particles.len()).filter(|i| *i != best).collect();
        // OCCT's pairwise swaps determine tie order; a standard sort is not equivalent.
        for outer in 0..indices.len().saturating_sub(1) {
            for inner in outer + 1..indices.len() {
                let a: Option<f64> = self.particles[indices[outer]].best_value;
                let b: Option<f64> = self.particles[indices[inner]].best_value;
                if a.is_some_and(|a| b.is_none_or(|b| a < b)) {
                    indices.swap(outer, inner);
                }
            }
        }
        indices
    }
}
