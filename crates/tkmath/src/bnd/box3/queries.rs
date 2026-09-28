// Created on: 1991-03-08
// Created by: Christophe MARION
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::BoundingBox3;
use crate::gp::{Plane3, Point3, Transform3};
use rustycad_tkernel::standard::Failure;
impl BoundingBox3 {
    /// Tests a point with open flags and gap; `bounds.is_out_point(point)`.
    pub fn is_out_point(&self, point: Point3) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        (0..3).any(|axis| {
            (!self.side_open(axis, false)
                && point.coordinates()[axis] < self.minimum[axis] - self.gap)
                || (!self.side_open(axis, true)
                    && point.coordinates()[axis] > self.maximum[axis] + self.gap)
        })
    }
    /// Includes the boundary and tolerance gap; `bounds.contains(point)`.
    pub fn contains(&self, point: Point3) -> bool {
        !self.is_out_point(point)
    }
    /// Uses native corner subtraction before the sum of gaps; `bounds.is_out_box(&other)`.
    pub fn is_out_box(&self, other: &Self) -> bool {
        if self.is_void() || other.is_void() {
            return true;
        }
        if self.is_whole() || other.is_whole() {
            return false;
        }
        let delta = other.gap + self.gap;
        (0..3).any(|i| {
            (!self.side_open(i, false)
                && !other.side_open(i, true)
                && self.minimum[i] - other.maximum[i] > delta)
                || (!self.side_open(i, true)
                    && !other.side_open(i, false)
                    && other.minimum[i] - self.maximum[i] > delta)
        })
    }
    /// Includes touching boxes; `bounds.intersects(&other)`.
    pub fn intersects(&self, other: &Self) -> bool {
        !self.is_out_box(other)
    }
    /// Tests against the transformed other box; `bounds.is_out_transformed(&other,transform)`.
    pub fn is_out_transformed(&self, other: &Self, transform: Transform3) -> bool {
        self.is_out_box(&other.transformed(transform))
    }
    /// Transforms both bounding volumes before testing; `bounds.is_out_both(first,&other,second)`.
    pub fn is_out_both(&self, first: Transform3, other: &Self, second: Transform3) -> bool {
        self.transformed(first)
            .is_out_box(&other.transformed(second))
    }
    /// Reproduces strict plane-side tests at all corners, including native tangent asymmetry; `bounds.is_out_plane(plane)?`.
    pub fn is_out_plane(&self, plane: Plane3) -> Result<bool, Failure> {
        if self.is_whole() {
            return Ok(false);
        }
        if self.is_void() {
            return Ok(true);
        }
        let bounds = self.get()?;
        let equation = plane.coefficients()?;
        let first = corner_side(bounds, equation, 0);
        Ok((1..8).all(|index| corner_side(bounds, equation, index) == first))
    }
    /// Returns zero for void boxes and native interval distance otherwise; `bounds.distance(&other)`.
    pub fn distance(&self, other: &Self) -> f64 {
        if self.is_void() || other.is_void() {
            return 0.0;
        }
        let left = self.get().expect("nonvoid bounds");
        let right = other.get().expect("nonvoid bounds");
        let [x, y, z] = std::array::from_fn(|i| {
            interval_square_distance(left[i], left[i + 3], right[i], right[i + 3])
        });
        (x + y + z).sqrt()
    }
}
fn corner_side(bounds: [f64; 6], equation: [f64; 4], index: usize) -> bool {
    let [x, y, z] = std::array::from_fn(|axis| {
        bounds[axis + if index & (1 << (2 - axis)) == 0 { 0 } else { 3 }]
    });
    let [a, b, c, d] = equation;
    a * x + b * y + c * z + d > 0.0
}
fn interval_square_distance(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if (a <= c && c <= b) || (c <= a && a <= d) {
        return 0.0;
    }
    let first = (a - d) * (a - d);
    let second = (b - c) * (b - c);
    if second < first { second } else { first }
}
