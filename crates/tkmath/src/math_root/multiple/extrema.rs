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

//! Derivative and golden-section extrema candidates preserve native callback order.
use super::*;
use crate::math_utils::GOLDEN_SECTION;
struct Candidate {
    root: f64,
    residual: f64,
    derivative: f64,
}
fn derivative_candidate<F: FunctionWithDerivative>(
    search: &mut Search<'_, F>,
    lower: f64,
    upper: f64,
) -> Result<Option<Candidate>, Status> {
    let mut config: Config = search.configuration(search.config.x_tolerance);
    config.f_tolerance = 0.0;
    let mut wrapper = |x: f64, out: &mut f64| {
        let mut value: f64 = 0.0;
        if !search.function.values(x, &mut value, out) {
            return false;
        }
        if !value.is_finite() {
            *out = value;
        }
        true
    };
    let result: ScalarResult = super::super::brent(&mut wrapper, lower, upper, &config);
    search.result.iterations = search.result.iterations.wrapping_add(result.iterations);
    if !result.is_done() || result.root.is_none() {
        return Ok(None);
    }
    accepted_derivative_candidate(search, result.root.unwrap())
}
fn accepted_derivative_candidate<F: FunctionWithDerivative>(
    search: &mut Search<'_, F>,
    root: f64,
) -> Result<Option<Candidate>, Status> {
    let mut value: f64 = search.value(root)?;
    let residual: f64 = (value - search.config.offset).abs();
    if residual >= search.config.f_tolerance {
        return Ok(None);
    }
    let mut derivative: f64 = 0.0;
    search.shifted_values_into(root, &mut value, &mut derivative)?;
    Ok(Some(Candidate {
        root,
        residual,
        derivative,
    }))
}
struct Golden {
    positions: [f64; 4],
    values: [f64; 4],
    minimum: bool,
}
impl Golden {
    fn new<F: FunctionWithDerivative>(
        search: &mut Search<'_, F>,
        i: usize,
        midpoint: f64,
        lower: f64,
        upper: f64,
    ) -> Result<Self, Status> {
        let (first, second): (f64, f64) = if (upper - midpoint).abs() > (lower - midpoint).abs() {
            (midpoint, midpoint + GOLDEN_SECTION * (upper - midpoint))
        } else {
            (midpoint - GOLDEN_SECTION * (midpoint - lower), midpoint)
        };
        let f1: f64 = search.shifted(first)?;
        let f2: f64 = search.shifted(second)?;
        Ok(Self {
            positions: [lower, first, second, upper],
            values: [search.ys[i - 1], f1, f2, search.ys[i + 1]],
            minimum: search.ys[i - 1] > 0.0,
        })
    }
    fn continuing(&self, epsilon: f64) -> bool {
        (self.positions[3] - self.positions[0]).abs()
            > epsilon * 10.0 * (self.positions[1].abs() + self.positions[2].abs())
            && (self.positions[1] - self.positions[2]).abs() > 0.001 * epsilon
    }
    fn advance<F: Function>(&mut self, search: &mut Search<'_, F>) -> Result<(), Status> {
        if (self.minimum && self.values[2] < self.values[1])
            || (!self.minimum && self.values[2] > self.values[1])
        {
            self.positions[0] = self.positions[1];
            self.positions[1] = self.positions[2];
            self.positions[2] =
                (1.0 - GOLDEN_SECTION) * self.positions[1] + GOLDEN_SECTION * self.positions[3];
            self.values[0] = self.values[1];
            self.values[1] = self.values[2];
            // OCCT reuses this output across calls, including callbacks that leave it untouched.
            search.shifted_into(self.positions[2], &mut self.values[2])?;
        } else {
            self.positions[3] = self.positions[2];
            self.positions[2] = self.positions[1];
            self.positions[1] =
                (1.0 - GOLDEN_SECTION) * self.positions[2] + GOLDEN_SECTION * self.positions[0];
            self.values[3] = self.values[2];
            self.values[2] = self.values[1];
            search.shifted_into(self.positions[1], &mut self.values[1])?;
        }
        Ok(())
    }
    fn crossing_roots<F: FunctionWithDerivative>(
        &self,
        search: &mut Search<'_, F>,
    ) -> Result<(), Status> {
        for (left, right) in [(0, 1), (2, 3)] {
            if opposite(self.values[right], self.values[left]) {
                search.refined_bracket(
                    self.positions[left],
                    self.values[left],
                    self.positions[right],
                    self.values[right],
                )?;
            }
        }
        Ok(())
    }
    fn candidate(&self, tolerance: f64) -> Option<Candidate> {
        let index: usize = if (self.minimum && self.values[1] < self.values[2])
            || (!self.minimum && self.values[1] > self.values[2])
        {
            1
        } else {
            2
        };
        let residual: f64 = self.values[index].abs();
        if residual < tolerance {
            Some(Candidate {
                root: self.positions[index],
                residual,
                derivative: 0.0,
            })
        } else {
            None
        }
    }
}
fn choose<F: FunctionWithDerivative>(
    search: &mut Search<'_, F>,
    first: Option<Candidate>,
    second: Option<Candidate>,
) -> Result<Option<f64>, Status> {
    match (first, second) {
        (Some(first), Some(second)) => choose_pair(search, first, second).map(Some),
        (Some(first), None) => Ok(Some(first.root)),
        (None, Some(second)) => Ok(Some(second.root)),
        (None, None) => Ok(None),
    }
}
fn choose_pair<F: FunctionWithDerivative>(
    search: &mut Search<'_, F>,
    first: Candidate,
    mut second: Candidate,
) -> Result<f64, Status> {
    if second.residual - first.residual > search.config.f_tolerance {
        return Ok(first.root);
    }
    if first.residual - second.residual > search.config.f_tolerance {
        return Ok(second.root);
    }
    let mut value: f64 = 0.0;
    search.shifted_values_into(second.root, &mut value, &mut second.derivative)?;
    Ok(if first.derivative.abs() < second.derivative.abs() {
        first.root
    } else {
        second.root
    })
}
pub(super) fn search_extremum<F: FunctionWithDerivative>(
    search: &mut Search<'_, F>,
    i: usize,
    midpoint: f64,
) -> Result<(), Status> {
    let lower: f64 = search.lower.max(midpoint - search.dx);
    let upper: f64 = search.upper.min(midpoint + search.dx);
    let first: Option<Candidate> = derivative_candidate(search, lower, upper)?;
    let mut golden: Golden = Golden::new(search, i, midpoint, lower, upper)?;
    while golden.continuing(search.epsilon) {
        golden.advance(search)?;
        golden.crossing_roots(search)?;
    }
    let second: Option<Candidate> = golden.candidate(search.config.f_tolerance);
    if let Some(root) = choose(search, first, second)? {
        search.add_at(root)?;
    }
    Ok(())
}
