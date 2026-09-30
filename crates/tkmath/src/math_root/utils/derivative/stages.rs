//! Sampling, sign-change, exact-zero and extremum-detection stages of the derivative search.
use super::super::have_opposite_signs;
use super::*;
use crate::bvh::{std_max, std_min};
impl<F: FunctionWithDerivative> Search<'_, F> {
    pub(super) fn sample(&mut self) -> Result<(), Status> {
        for i in 0..=self.count {
            let ratio: f64 = i as f64 / self.count as f64;
            let x: f64 = (1.0 - ratio) * self.lower + ratio * self.upper;
            let value: f64 = self.shifted(x)?;
            self.xs.push(x);
            self.ys.push(value);
        }
        let null: f64 = self.config.null_tolerance;
        self.result.is_all_null = self.ys.iter().all(|y| !(*y > null || *y < -null));
        Ok(())
    }
    pub(super) fn crossing_samples(&mut self) -> Result<(), Status> {
        for i in 0..self.count {
            if have_opposite_signs(self.ys[i], self.ys[i + 1]) {
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
        if have_opposite_signs(left_value, right_value) {
            self.refined_bracket(left, left_value, right, right_value)?;
        } else if left_value != 0.0 || right_value != 0.0 {
            self.add_at(zero)?;
        }
        Ok(())
    }
    pub(super) fn sampled_zeros(&mut self) -> Result<(), Status> {
        for i in 0..=self.count {
            if self.ys[i] == 0.0 {
                self.exact_zero_sample(i)?;
            }
        }
        let tolerance: f64 = self.config.f_tolerance;
        if self.ys[0] <= tolerance && self.ys[0] >= -tolerance {
            self.add_at(self.lower)?;
        }
        if self.ys[self.count] <= tolerance && self.ys[self.count] >= -tolerance {
            self.add_at(self.upper)?;
        }
        Ok(())
    }
    /// `std::max(aLower, aMidX - aDx)`, keeping the first operand on ties (signed zeros).
    pub(super) fn probe_lower(&self, midpoint: f64) -> f64 {
        std_max(self.lower, midpoint - self.dx)
    }
    /// `std::min(aUpper, aMidX + aDx)`, keeping the first operand on ties (signed zeros).
    pub(super) fn probe_upper(&self, midpoint: f64) -> f64 {
        std_min(self.upper, midpoint + self.dx)
    }
    fn sampled_extremum(&self, i: usize) -> bool {
        (self.ys[i] > 0.0 && self.ys[i - 1] > self.ys[i] && self.ys[i + 1] > self.ys[i])
            || (self.ys[i] < 0.0 && self.ys[i - 1] < self.ys[i] && self.ys[i + 1] < self.ys[i])
    }
    /// Probes left then right of the extremum; a short Newton step to zero means a near root.
    fn rediscretize(&mut self, i: usize, midpoint: f64) -> Result<bool, Status> {
        if !self.sampled_extremum(i) {
            return Ok(false);
        }
        let mut value: f64 = 0.0;
        let mut derivative: f64 = 0.0;
        let major: f64 = 5.0 * self.dx;
        for x in [self.probe_lower(midpoint), self.probe_upper(midpoint)] {
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
    pub(super) fn extrema_search(&mut self) -> Result<(), Status> {
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
}
