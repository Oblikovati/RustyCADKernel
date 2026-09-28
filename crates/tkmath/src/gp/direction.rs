// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/gp/gp_Dir.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{Failure, construction, cross, dot};
/// Normalized 3D direction; `Direction3::new(1.0,0.0,0.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Direction3 {
    pub(crate) coordinates: [f64; 3],
}
impl Default for Direction3 {
    fn default() -> Self {
        Self::X
    }
}
impl Direction3 {
    /// Positive X axis; `Direction3::X`.
    pub const X: Self = Self {
        coordinates: [1.0, 0.0, 0.0],
    };
    /// Positive Y axis; `Direction3::Y`.
    pub const Y: Self = Self {
        coordinates: [0.0, 1.0, 0.0],
    };
    /// Positive Z axis; `Direction3::Z`.
    pub const Z: Self = Self {
        coordinates: [0.0, 0.0, 1.0],
    };
    /// Normalizes with gp's squared-norm fast path; `Direction3::new(1.0,2.0,3.0)?`.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, Failure> {
        Self::normalize([x, y, z], "gp_Dir() - input vector has zero norm")
    }
    pub(crate) fn normalize(coordinates: [f64; 3], message: &str) -> Result<Self, Failure> {
        let squared = dot(coordinates, coordinates);
        if squared == 1.0 {
            return Ok(Self { coordinates });
        }
        if squared <= 0.0 {
            return Err(construction(message));
        }
        let magnitude = squared.sqrt();
        Ok(Self {
            coordinates: coordinates.map(|value| value / magnitude),
        })
    }
    /// Applies gp_Dir's normalization where the zero-norm raise is unreachable or compiled out
    /// (ElSLib builds with `No_Exception`); a zero input yields NaN components like the
    /// unchecked native division; `Direction3::normalize_unchecked([1.0,1.0,0.0])`.
    pub(crate) fn normalize_unchecked(coordinates: [f64; 3]) -> Self {
        let squared = dot(coordinates, coordinates);
        if squared == 1.0 {
            return Self { coordinates };
        }
        let magnitude = squared.sqrt();
        Self {
            coordinates: coordinates.map(|value| value / magnitude),
        }
    }
    /// Borrows normalized coordinates; `direction.coordinates()`.
    pub const fn coordinates(&self) -> &[f64; 3] {
        &self.coordinates
    }
    /// Computes gp_Dir's asin/acos branch around 45 degrees; `direction.angle(&other)`.
    // Preserve the decimal branch boundary in gp_Dir::Angle, not the rounded constant.
    #[allow(clippy::approx_constant)]
    pub fn angle(&self, other: &Self) -> f64 {
        let cosine = dot(self.coordinates, other.coordinates);
        if cosine > -0.70710678118655 && cosine < 0.70710678118655 {
            return cosine.acos();
        }
        let normal = cross(self.coordinates, other.coordinates);
        let sine = dot(normal, normal).sqrt();
        if cosine < 0.0 {
            std::f64::consts::PI - sine.asin()
        } else {
            sine.asin()
        }
    }
    /// Compares angular distance with a tolerance; `direction.is_equal(&other,1e-12)`.
    pub fn is_equal(&self, other: &Self, tolerance: f64) -> bool {
        self.angle(other) <= tolerance
    }
}

mod coordinates;
mod operations;
mod products;
mod standard;
pub(crate) use products::{CheckedProducts, DirectionProducts, UncheckedProducts};
pub use standard::StandardDirection3;
