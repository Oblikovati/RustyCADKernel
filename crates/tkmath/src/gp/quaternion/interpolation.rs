// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of gp_QuaternionNLerp.hxx and gp_QuaternionSLerp.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Linear and spherical quaternion interpolation along the shorter arc.
use super::Quaternion;
/// gp_QuaternionNLerp: `start + (end - start) * t`, not renormalized;
/// `QuaternionNLerp::new(a, b).interpolate(0.5)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuaternionNLerp {
    start: Quaternion,
    delta: Quaternion,
}
impl QuaternionNLerp {
    /// Normalizes both ends, as `gp_QuaternionNLerp(start, end)`/`Init`;
    /// `QuaternionNLerp::new(a, b)`.
    pub fn new(start: Quaternion, end: Quaternion) -> Self {
        let mut lerp = Self::default();
        lerp.init(start, end);
        lerp
    }
    /// Normalizes both ends, as `Init`; `lerp.init(a, b)`.
    pub fn init(&mut self, start: Quaternion, end: Quaternion) {
        self.init_from_unit(start.normalized(), end.normalized());
    }
    /// Takes unit ends as they are; flips `end` for a negative dot product so the path is the
    /// shorter arc, as `InitFromUnit`; `lerp.init_from_unit(a, b)`.
    pub fn init_from_unit(&mut self, start: Quaternion, end: Quaternion) {
        self.start = start;
        let mut end = end;
        if start.dot(end) < 0.0 {
            end = -end;
        }
        self.delta = end - start;
    }
    /// `start + delta * t`, as `Interpolate(t, result)`; `lerp.interpolate(0.25)`.
    pub fn interpolate(&self, parameter: f64) -> Quaternion {
        self.start + self.delta * parameter
    }
    /// The static `Interpolate(start, end, t)`; `QuaternionNLerp::interpolated(a, b, 0.5)`.
    pub fn interpolated(start: Quaternion, end: Quaternion, parameter: f64) -> Quaternion {
        Self::new(start, end).interpolate(parameter)
    }
}
/// gp_QuaternionSLerp: constant-speed interpolation with the angle clamped below
/// `acos(0.9999)`; `QuaternionSLerp::new(a, b).interpolate(0.5)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuaternionSLerp {
    start: Quaternion,
    end: Quaternion,
    omega: f64,
}
impl QuaternionSLerp {
    /// Normalizes both ends, as `gp_QuaternionSLerp(start, end)`/`Init`;
    /// `QuaternionSLerp::new(a, b)`.
    pub fn new(start: Quaternion, end: Quaternion) -> Self {
        let mut lerp = Self::default();
        lerp.init(start, end);
        lerp
    }
    /// Normalizes both ends, as `Init`; `lerp.init(a, b)`.
    pub fn init(&mut self, start: Quaternion, end: Quaternion) {
        self.init_from_unit(start.normalized(), end.normalized());
    }
    /// Takes unit ends; flips `end` onto the shorter arc, clamps the cosine to 0.9999 and
    /// pre-divides both ends by `sin(omega)`, as `InitFromUnit`; `lerp.init_from_unit(a, b)`.
    pub fn init_from_unit(&mut self, start: Quaternion, end: Quaternion) {
        self.start = start;
        self.end = end;
        let mut cosine = start.dot(end);
        if cosine < 0.0 {
            cosine = -cosine;
            self.end = -self.end;
        }
        if cosine > 0.9999 {
            cosine = 0.9999;
        }
        self.omega = cosine.acos();
        let inverse_sine = 1.0 / self.omega.sin();
        self.start.scale(inverse_sine);
        self.end.scale(inverse_sine);
    }
    /// `start*sin((1-t)*omega) + end*sin(t*omega)`, as `Interpolate(t, result)`;
    /// `lerp.interpolate(0.25)`.
    pub fn interpolate(&self, parameter: f64) -> Quaternion {
        self.start * ((1.0 - parameter) * self.omega).sin()
            + self.end * (parameter * self.omega).sin()
    }
    /// The static `Interpolate(start, end, t)`; `QuaternionSLerp::interpolated(a, b, 0.5)`.
    pub fn interpolated(start: Quaternion, end: Quaternion, parameter: f64) -> Quaternion {
        Self::new(start, end).interpolate(parameter)
    }
}
