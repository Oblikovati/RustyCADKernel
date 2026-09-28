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

/// Native constant function; `Constant::new(constant)`.
#[derive(Clone)]
pub struct Constant {
    constant: f64,
}
impl Constant {
    /// Stores native parameters without validation; `Constant::new(constant)`.
    pub fn new(constant: f64) -> Self {
        Self { constant }
    }
}
impl Function for Constant {
    fn value(&mut self, _x: f64, value: &mut f64) -> bool {
        *value = self.constant;
        true
    }
}
impl FunctionWithDerivative for Constant {
    fn derivative(&mut self, _x: f64, derivative: &mut f64) -> bool {
        *derivative = 0.0;
        true
    }
    fn values(&mut self, _x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = self.constant;
        *derivative = 0.0;
        true
    }
}

/// Native linear function; `Linear::new(slope,intercept)`.
#[derive(Clone)]
pub struct Linear {
    slope: f64,
    intercept: f64,
}
impl Linear {
    /// Stores native parameters without validation; `Linear::new(slope,intercept)`.
    pub fn new(slope: f64, intercept: f64) -> Self {
        Self { slope, intercept }
    }
}
impl Function for Linear {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = self.slope * x + self.intercept;
        true
    }
}
impl FunctionWithDerivative for Linear {
    fn derivative(&mut self, _x: f64, derivative: &mut f64) -> bool {
        *derivative = self.slope;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = self.slope * x + self.intercept;
        *derivative = self.slope;
        true
    }
}

/// Native sine function; `Sine::new(amplitude,frequency,phase,offset)`.
#[derive(Clone)]
pub struct Sine {
    amplitude: f64,
    frequency: f64,
    phase: f64,
    offset: f64,
}
impl Sine {
    /// Stores native parameters without validation; `Sine::new(amplitude,frequency,phase,offset)`.
    pub fn new(amplitude: f64, frequency: f64, phase: f64, offset: f64) -> Self {
        Self {
            amplitude,
            frequency,
            phase,
            offset,
        }
    }
}
impl Function for Sine {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = self.amplitude * (self.frequency * x + self.phase).sin() + self.offset;
        true
    }
}
impl FunctionWithDerivative for Sine {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = self.amplitude * self.frequency * (self.frequency * x + self.phase).cos();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = self.amplitude * (self.frequency * x + self.phase).sin() + self.offset;
        *derivative = self.amplitude * self.frequency * (self.frequency * x + self.phase).cos();
        true
    }
}
impl Default for Sine {
    fn default() -> Self {
        Self::new(1.0, 1.0, 0.0, 0.0)
    }
}

/// Native cosine function; `Cosine::new(amplitude,frequency,phase,offset)`.
#[derive(Clone)]
pub struct Cosine {
    amplitude: f64,
    frequency: f64,
    phase: f64,
    offset: f64,
}
impl Cosine {
    /// Stores native parameters without validation; `Cosine::new(amplitude,frequency,phase,offset)`.
    pub fn new(amplitude: f64, frequency: f64, phase: f64, offset: f64) -> Self {
        Self {
            amplitude,
            frequency,
            phase,
            offset,
        }
    }
}
impl Function for Cosine {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = self.amplitude * (self.frequency * x + self.phase).cos() + self.offset;
        true
    }
}
impl FunctionWithDerivative for Cosine {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = -self.amplitude * self.frequency * (self.frequency * x + self.phase).sin();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = self.amplitude * (self.frequency * x + self.phase).cos() + self.offset;
        *derivative = -self.amplitude * self.frequency * (self.frequency * x + self.phase).sin();
        true
    }
}
impl Default for Cosine {
    fn default() -> Self {
        Self::new(1.0, 1.0, 0.0, 0.0)
    }
}

/// Native exponential function; `Exponential::new(scale,rate,offset)`.
#[derive(Clone)]
pub struct Exponential {
    scale: f64,
    rate: f64,
    offset: f64,
}
impl Exponential {
    /// Stores native parameters without validation; `Exponential::new(scale,rate,offset)`.
    pub fn new(scale: f64, rate: f64, offset: f64) -> Self {
        Self {
            scale,
            rate,
            offset,
        }
    }
}
impl Function for Exponential {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = self.scale * (self.rate * x).exp() + self.offset;
        true
    }
}
impl FunctionWithDerivative for Exponential {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = self.scale * self.rate * (self.rate * x).exp();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = self.scale * (self.rate * x).exp() + self.offset;
        *derivative = self.scale * self.rate * (self.rate * x).exp();
        true
    }
}
impl Default for Exponential {
    fn default() -> Self {
        Self::new(1.0, 1.0, 0.0)
    }
}
