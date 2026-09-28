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
// Rust translation of GeomBndLib_OptimizationHelpers.pxx (SurfCoordFunc) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SurfCoordFunc`: the two-variable coordinate function of the surface PSO + Powell
//! search, with a linear penalty outside the patch.
use super::optimization::pair_vector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::math::{MultipleVarFunction, Vector};

/// `sign * Coord(index)` of the surface point; outside the patch the point is clamped to
/// the patch border and `penalty * distance` is added per direction. The penalty is
/// `max(1, 10 * max(|dF/du|, |dF/dv|))` from the border midpoints.
pub struct SurfCoordFunc<'a> {
    surface: &'a dyn Adaptor3dSurface,
    patch: [f64; 4],
    index: i32,
    sign: f64,
    penalty: f64,
    failure: Option<Failure>,
}

impl<'a> SurfCoordFunc<'a> {
    /// The function over `patch = [u_min, u_max, v_min, v_max]`; the constructor samples
    /// four border midpoints for the penalty, as natively;
    /// `SurfCoordFunc::new(surface, [0.0, 1.0, 0.0, 1.0], 3, -1.0)?`.
    pub fn new(
        surface: &'a dyn Adaptor3dSurface,
        patch: [f64; 4],
        index: i32,
        sign: f64,
    ) -> Result<Self, Failure> {
        let mut func: Self = Self {
            surface,
            patch,
            index,
            sign,
            penalty: 0.0,
            failure: None,
        };
        func.penalty = func.border_penalty()?;
        Ok(func)
    }
    fn border_penalty(&mut self) -> Result<f64, Failure> {
        let [u_min, u_max, v_min, v_max] = self.patch;
        // The native evaluation order: U border (min, max), then V border (min, max).
        let v_mid: f64 = (v_min + v_max) / 2.0;
        let f1: f64 = self.sample(u_min, v_mid)?;
        let f2: f64 = self.sample(u_max, v_mid)?;
        let du: f64 = ((f2 - f1) / (u_max - u_min)).abs();
        let u_mid: f64 = (u_min + u_max) / 2.0;
        let f1: f64 = self.sample(u_mid, v_min)?;
        let f2: f64 = self.sample(u_mid, v_max)?;
        let dv: f64 = ((f2 - f1) / (v_max - v_min)).abs();
        Ok(std_max(10.0 * std_max(du, dv), 1.0))
    }
    // One `Value(aX, aF)` call of the constructor, through the same penalty rule.
    fn sample(&mut self, u: f64, v: f64) -> Result<f64, Failure> {
        let mut value: f64 = 0.0;
        self.evaluate(&pair_vector(u, v)?, &mut value);
        self.take_failure()?;
        Ok(value)
    }
    /// The first evaluation failure, which the native code raises; `func.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take().map_or(Ok(()), Err)
    }
    // The clamped parameter and the penalty of one direction.
    fn clamp(&self, x: f64, min: f64, max: f64) -> (f64, f64) {
        if x < min {
            return (min, self.penalty * (min - x));
        }
        if x > max {
            return (max, self.penalty * (x - max));
        }
        (x, 0.0)
    }
    fn evaluate(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let [u_min, u_max, v_min, v_max] = self.patch;
        let u: f64 = point.at(0).copied().unwrap_or(f64::NAN);
        let v: f64 = point.at(1).copied().unwrap_or(f64::NAN);
        let inside: bool = u >= u_min && u <= u_max && v >= v_min && v <= v_max;
        let ((u0, u_pen), (v0, v_pen)) = if inside {
            ((u, 0.0), (v, 0.0))
        } else {
            (self.clamp(u, u_min, u_max), self.clamp(v, v_min, v_max))
        };
        match self.surface.value(u0, v0).and_then(|p| p.coord(self.index)) {
            // Inside the patch the native expression has no penalty terms at all.
            Ok(coordinate) if inside => *value = self.sign * coordinate,
            Ok(coordinate) => *value = self.sign * coordinate + u_pen + v_pen,
            Err(failure) => {
                self.failure.get_or_insert(failure);
            }
        }
        true
    }
}

impl MultipleVarFunction for SurfCoordFunc<'_> {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        self.evaluate(point, value)
    }
}
