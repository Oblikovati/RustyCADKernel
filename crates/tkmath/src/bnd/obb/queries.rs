// Created by: Eugeny MALTCHIKOV
// Copyright (c) 2017 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::OrientedBoundingBox;
use crate::gp::{Point3, Xyz};
use rustycad_tkernel::standard::real;

impl OrientedBoundingBox {
    /// Separating-axis test over the 15 candidate axes (6 box axes, 9 cross products);
    /// two axis-aligned boxes use the direct center test. Void boxes are always out.
    /// Example: `first.is_out_box(&second)`.
    pub fn is_out_box(&self, other: &Self) -> bool {
        if self.is_void() || other.is_void() {
            return true;
        }
        if self.is_aa_box && other.is_aa_box {
            return self.aligned_out(other);
        }
        let distance = other.center - self.center;
        self.own_axes_separate(other, distance)
            || other.own_axes_separate(self, distance)
            || self.cross_axes_separate(other, distance)
    }
    fn aligned_out(&self, other: &Self) -> bool {
        let (mine, theirs) = (self.center.coordinates(), other.center.coordinates());
        (0..3).any(|i| (theirs[i] - mine[i]).abs() > other.half_sizes[i] + self.half_sizes[i])
    }
    /// Tests the axes of `self`; the native loops differ only in operand order of the final
    /// sum (`h_i + L2` for this box, `L1 + h_i` for the other), which is commutative.
    fn own_axes_separate(&self, other: &Self, distance: Xyz) -> bool {
        (0..3).any(|i| {
            let reach = other.projected_length(self.axes[i]);
            distance.dot(self.axes[i]).abs() > self.half_sizes[i] + reach
        })
    }
    fn cross_axes_separate(&self, other: &Self, distance: Xyz) -> bool {
        // Near-parallel axis pairs give no separating direction.
        let null_norm = real::epsilon(1.0);
        (0..9).any(|pair| {
            let axis = self.axes[pair / 3].crossed(other.axes[pair % 3]);
            let norm = axis.modulus();
            if norm < null_norm {
                return false;
            }
            let axis = axis / norm;
            let reach = self.projected_length(axis) + other.projected_length(axis);
            distance.dot(axis).abs() > reach
        })
    }
    /// `Σ h_k·|a_k·L|`, accumulated from `0.` in native order.
    fn projected_length(&self, direction: Xyz) -> f64 {
        (0..3).fold(0.0, |sum, k| {
            sum + self.half_sizes[k] * self.axes[k].dot(direction).abs()
        })
    }
    /// True when some local coordinate of the point exceeds its half size; a void box is
    /// not special-cased. Example: `obb.is_out_point(Point3::new(10.0, 0.0, 0.0))`.
    pub fn is_out_point(&self, point: Point3) -> bool {
        let offset = point.xyz() - self.center;
        (0..3).any(|i| self.axes[i].dot(offset).abs() > self.half_sizes[i])
    }
    /// Inside or on the boundary; `obb.contains(point)`.
    pub fn contains(&self, point: Point3) -> bool {
        !self.is_out_point(point)
    }
    /// Intersecting or inside; `first.intersects(&second)`.
    pub fn intersects(&self, other: &Self) -> bool {
        !self.is_out_box(other)
    }
    /// True when all eight vertices of `other` are inside; false when either box is void.
    /// Example: `outer.is_completely_inside(&inner)`.
    pub fn is_completely_inside(&self, other: &Self) -> bool {
        if self.is_void() {
            return false;
        }
        let Some(vertices) = other.vertices() else {
            return false;
        };
        !vertices.iter().any(|vertex| self.is_out_point(*vertex))
    }
}
