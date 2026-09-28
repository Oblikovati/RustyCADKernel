// Created by: Olga Suryaninova
// Created on: 2019-11-25
// Copyright (c) 2019 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Ray.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Ray<T, N>`: a ray with its precomputed reciprocal direction.
use super::scalar::BvhScalar;
use super::vector::BvhVector;

/// Ray origin, direction and `1/direction` (infinity for zero components).
///
/// The default ray has zero origin, direction and reciprocal, as upstream.
/// Example: `BvhRay::new(Vec3::new(0.0,0.0,0.0), Vec3::new(1.0,0.0,0.0))`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BvhRay<V: BvhVector> {
    /// Ray origin point.
    pub origin: V,
    /// Ray direction vector.
    pub direct: V,
    /// Reciprocal of direction (`1/Direct`, `+inf` for a zero component).
    pub inv_direct: V,
}
impl<V: BvhVector> BvhRay<V> {
    /// Creates ray with given origin and direction; `BvhRay::new(origin, direction)`.
    pub fn new(origin: V, direct: V) -> Self {
        Self {
            origin,
            direct,
            inv_direct: inverse_direction(&direct),
        }
    }
}

/// `invComponent`: `1 / d`, or positive infinity when `d == 0` (either sign).
fn inverse_component<T: BvhScalar>(component: T) -> T {
    if component != T::ZERO {
        T::ONE / component
    } else {
        T::INFINITY
    }
}

fn inverse_direction<V: BvhVector>(direct: &V) -> V {
    let mut inverse = *direct;
    for axis in 0..V::DIMENSION {
        inverse[axis] = inverse_component(direct[axis]);
    }
    inverse
}
