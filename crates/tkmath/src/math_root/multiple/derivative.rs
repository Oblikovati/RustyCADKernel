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

//! Native bracket polishing and derivative-search staging.
use super::*;
impl<F: FunctionWithDerivative> Search<'_, F> {
    pub(super) fn shifted_values_into(
        &mut self,
        x: f64,
        value: &mut f64,
        derivative: &mut f64,
    ) -> Result<(), Status> {
        if !self.function.values(x, value, derivative) {
            return Err(Status::CallbackError);
        }
        finite(*value)?;
        finite(*derivative)?;
        *value -= self.config.offset;
        finite(*value)?;
        Ok(())
    }
    fn polished(&mut self, mut root: f64, lower: f64, upper: f64) -> Result<(f64, f64), Status> {
        let mut root_value: f64 = self.value(root)?;
        let mut shifted: f64 = root_value - self.config.offset;
        for _ in 0..5 {
            let Some((candidate, value)) = self.polish_candidate(root, lower, upper)? else {
                break;
            };
            let candidate_shifted: f64 = value - self.config.offset;
            if candidate_shifted.abs() >= shifted.abs() {
                break;
            }
            root = candidate;
            root_value = value;
            shifted = candidate_shifted;
        }
        Ok((root, root_value))
    }
    fn polish_candidate(
        &mut self,
        root: f64,
        lower: f64,
        upper: f64,
    ) -> Result<Option<(f64, f64)>, Status> {
        let mut value: f64 = 0.0;
        let mut derivative: f64 = 0.0;
        self.shifted_values_into(root, &mut value, &mut derivative)?;
        if derivative == 0.0 {
            return Ok(None);
        }
        let candidate: f64 = root - value / derivative;
        if !candidate.is_finite() || candidate < lower || candidate > upper {
            return Ok(None);
        }
        Ok(Some((candidate, self.value(candidate)?)))
    }
    pub(super) fn refined_bracket(
        &mut self,
        lower: f64,
        left: f64,
        upper: f64,
        right: f64,
    ) -> Result<(), Status> {
        finite(left)?;
        finite(right)?;
        let root: f64 = self.bracket(lower, upper)?;
        let (root, value) = self.polished(root, lower, upper)?;
        self.result.add(self.epsilon, root, value);
        Ok(())
    }
    fn crossing_samples(&mut self) -> Result<(), Status> {
        for i in 0..self.count {
            if opposite(self.ys[i], self.ys[i + 1]) {
                self.refined_bracket(self.xs[i], self.ys[i], self.xs[i + 1], self.ys[i + 1])?;
            }
        }
        Ok(())
    }
    fn clamp_sample(&self, mut x: f64) -> f64 {
        if x < self.lower {
            x = self.lower;
        }
        if x > self.upper {
            x = self.upper;
        }
        x
    }
    fn exact_zero_sample(&mut self, i: usize) -> Result<(), Status> {
        let zero: f64 = self.xs[i];
        let left: f64 = self.clamp_sample(zero - 0.5 * self.dx);
        let right: f64 = self.clamp_sample(zero + 0.5 * self.dx);
        let left_value: f64 = self.shifted(left)?;
        let right_value: f64 = self.shifted(right)?;
        if opposite(left_value, right_value) {
            self.refined_bracket(left, left_value, right, right_value)?;
        } else if left_value != 0.0 || right_value != 0.0 {
            self.add_at(zero)?;
        }
        Ok(())
    }
    fn sampled_zeros(&mut self) -> Result<(), Status> {
        for i in 0..=self.count {
            if self.ys[i] == 0.0 {
                self.exact_zero_sample(i)?;
            }
        }
        if self.ys[0] <= self.config.f_tolerance && self.ys[0] >= -self.config.f_tolerance {
            self.add_at(self.lower)?;
        }
        if self.ys[self.count] <= self.config.f_tolerance
            && self.ys[self.count] >= -self.config.f_tolerance
        {
            self.add_at(self.upper)?;
        }
        Ok(())
    }
    fn sampled_extremum(&self, i: usize) -> bool {
        (self.ys[i] > 0.0 && self.ys[i - 1] > self.ys[i] && self.ys[i + 1] > self.ys[i])
            || (self.ys[i] < 0.0 && self.ys[i - 1] < self.ys[i] && self.ys[i + 1] < self.ys[i])
    }
    fn rediscretize(&mut self, i: usize, midpoint: f64) -> Result<bool, Status> {
        if !self.sampled_extremum(i) {
            return Ok(false);
        }
        let mut value: f64 = 0.0;
        let mut derivative: f64 = 0.0;
        let major: f64 = 5.0 * self.dx;
        for x in [
            self.lower.max(midpoint - self.dx),
            self.upper.min(midpoint + self.dx),
        ] {
            self.shifted_values_into(x, &mut value, &mut derivative)?;
            if derivative.abs() > 1e-10 {
                let step: f64 = value / derivative;
                if step < major && step > -major {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    fn extrema_search(&mut self) -> Result<(), Status> {
        let mut midpoint: f64 = self.lower + self.dx;
        for i in 1..self.count {
            if midpoint > self.upper {
                midpoint = self.upper;
            }
            if self.rediscretize(i, midpoint)? {
                extrema::search_extremum(self, i, midpoint)?;
            }
            midpoint += self.dx;
        }
        Ok(())
    }
    pub(super) fn derivative_search(&mut self) -> Result<(), Status> {
        self.sample()?;
        if self.result.is_all_null {
            return Ok(());
        }
        self.crossing_samples()?;
        self.sampled_zeros()?;
        self.extrema_search()?;
        self.result.sort();
        Ok(())
    }
}
