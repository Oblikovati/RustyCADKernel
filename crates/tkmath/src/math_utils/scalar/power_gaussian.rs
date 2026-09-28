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

use super::*;
/// Native power functor with finite-input validation; `Power::new(2.0,1.0,0.0)`.
#[derive(Clone)]
pub struct Power {
    exponent: f64,
    scale: f64,
    offset: f64,
}
impl Power {
    /// Retains the exponent, scale, and offset; `Power::new(0.5,1.0,0.0)`.
    pub fn new(exponent: f64, scale: f64, offset: f64) -> Self {
        Self {
            exponent,
            scale,
            offset,
        }
    }
    fn valid(&self, x: f64) -> bool {
        [x, self.exponent, self.scale, self.offset]
            .iter()
            .all(|v| v.is_finite())
            && !(x < 0.0 && self.exponent != self.exponent.floor())
    }
}
impl Function for Power {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        if !self.valid(x) {
            return false;
        }
        *value = self.scale * x.powf(self.exponent) + self.offset;
        value.is_finite()
    }
}
impl FunctionWithDerivative for Power {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        self.values(x, &mut 0.0, derivative)
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        if !self.valid(x) {
            return false;
        }
        if self.exponent == 0.0 {
            *value = self.scale + self.offset;
            *derivative = 0.0;
            return value.is_finite();
        }
        let power: f64 = x.powf(self.exponent);
        *value = self.scale * power + self.offset;
        if x == 0.0 {
            if self.exponent < 1.0 {
                return false;
            }
            *derivative = if self.exponent == 1.0 {
                self.scale
            } else {
                0.0
            };
        } else {
            *derivative = self.scale * self.exponent * power / x;
        }
        value.is_finite() && derivative.is_finite()
    }
}
/// Native Gaussian functor permits negative sigma; `Gaussian::default()`.
#[derive(Clone)]
pub struct Gaussian {
    amplitude: f64,
    mean: f64,
    sigma: f64,
}
impl Default for Gaussian {
    fn default() -> Self {
        Self::new(1.0, 0.0, 1.0)
    }
}
impl Gaussian {
    /// Retains parameters and validates them at evaluation; `Gaussian::new(1.0,0.0,2.0)`.
    pub fn new(amplitude: f64, mean: f64, sigma: f64) -> Self {
        Self {
            amplitude,
            mean,
            sigma,
        }
    }
    fn valid(&self, x: f64) -> bool {
        [x, self.amplitude, self.mean, self.sigma]
            .iter()
            .all(|v| v.is_finite())
            && self.sigma != 0.0
    }
}
impl Function for Gaussian {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        if !self.valid(x) {
            return false;
        }
        let z: f64 = (x - self.mean) / self.sigma;
        *value = self.amplitude * (-0.5 * z * z).exp();
        value.is_finite()
    }
}
impl FunctionWithDerivative for Gaussian {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        self.values(x, &mut 0.0, derivative)
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        if !self.valid(x) {
            return false;
        }
        let z: f64 = (x - self.mean) / self.sigma;
        let exponential: f64 = (-0.5 * z * z).exp();
        *value = self.amplitude * exponential;
        *derivative = -self.amplitude * z * exponential / self.sigma;
        value.is_finite() && derivative.is_finite()
    }
}
