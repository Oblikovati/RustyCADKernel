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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::BoundingBox2;
use crate::gp::{Direction2, Point2};
impl BoundingBox2 {
    /// Resets the gap and flags before setting a point; `bounds.set_point(point)`.
    pub fn set_point(&mut self, point: Point2) {
        self.flags = 1;
        self.gap = 0.0;
        self.add_point(point);
    }
    /// Adds coordinates without updating already-open sides; `bounds.add_point(point)`.
    pub fn add_point(&mut self, point: Point2) {
        self.update_bounds([point.x(), point.y()], [point.x(), point.y()]);
    }
    /// Replaces void bounds or expands only closed sides; `bounds.update_bounds([0.0;2],[1.0;2])`.
    pub fn update_bounds(&mut self, minimum: [f64; 2], maximum: [f64; 2]) {
        if self.is_void() {
            self.minimum = minimum;
            self.maximum = maximum;
            self.flags &= !1;
            return;
        }
        for i in 0..2 {
            if !self.side_open(i, false) && minimum[i] < self.minimum[i] {
                self.minimum[i] = minimum[i];
            }
            if !self.side_open(i, true) && self.maximum[i] < maximum[i] {
                self.maximum[i] = maximum[i];
            }
        }
    }
    /// Opens components greater than machine epsilon without clearing voidness; `bounds.add_direction(direction)`.
    pub fn add_direction(&mut self, direction: Direction2) {
        for (axis, value) in [direction.x(), direction.y()].into_iter().enumerate() {
            if value < -f64::EPSILON {
                self.open_side(axis, false);
            } else if value > f64::EPSILON {
                self.open_side(axis, true);
            }
        }
    }
    /// Preserves planar early-return and gap rules for whole boxes; `bounds.add_box(&other)`.
    pub fn add_box(&mut self, other: &Self) {
        if self.is_whole() || other.is_void() {
            return;
        }
        if other.is_whole() {
            self.set_whole();
            return;
        }
        if self.is_void() {
            *self = other.clone();
            return;
        }
        for axis in 0..2 {
            self.merge_axis(other, axis);
        }
        if self.gap < other.gap {
            self.gap = other.gap;
        }
    }
    fn merge_axis(&mut self, other: &Self, axis: usize) {
        for upper in [false, true] {
            if self.side_open(axis, upper) {
                continue;
            }
            if other.side_open(axis, upper) {
                self.open_side(axis, upper);
                continue;
            }
            self.merge_finite_side(other, axis, upper);
        }
    }
    fn merge_finite_side(&mut self, other: &Self, axis: usize, upper: bool) {
        if upper && self.maximum[axis] < other.maximum[axis] {
            self.maximum[axis] = other.maximum[axis];
        }
        if !upper && self.minimum[axis] > other.minimum[axis] {
            self.minimum[axis] = other.minimum[axis];
        }
    }
}
