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
use crate::gp::{Direction3, Point3, Transform3, TransformForm};
impl BoundingBox3 {
    /// Transforms finite corners and open directions independently; `bounds.transformed(transform)`.
    pub fn transformed(&self, transform: Transform3) -> Self {
        if self.is_void() {
            return Self::default();
        }
        if transform.form() == TransformForm::Identity {
            return self.clone();
        }
        if transform.form() == TransformForm::Translation {
            return self.translated_box(transform);
        }
        let mut result = Self::default();
        if self.has_finite_part() {
            self.transform_corners(transform, &mut result);
        }
        result.gap = self.gap;
        self.transform_open_sides(transform, &mut result);
        result
    }
    fn translated_box(&self, transform: Transform3) -> Self {
        let mut result = self.clone();
        if !self.has_finite_part() {
            return result;
        }
        for axis in 0..3 {
            result.minimum[axis] += transform.translation_part().coordinates()[axis];
            result.maximum[axis] += transform.translation_part().coordinates()[axis];
        }
        result
    }
    fn transform_corners(&self, transform: Transform3, result: &mut Self) {
        for index in 0..8 {
            let [x, y, z] = std::array::from_fn(|axis| {
                if index & (1 << axis) == 0 {
                    self.minimum[axis]
                } else {
                    self.maximum[axis]
                }
            });
            result.add_point(Point3::new(x, y, z).transformed(transform));
        }
    }
    fn transform_open_sides(&self, transform: Transform3, result: &mut Self) {
        for (axis, direction) in [Direction3::X, Direction3::Y, Direction3::Z]
            .into_iter()
            .enumerate()
        {
            if self.side_open(axis, false) {
                result.add_direction(direction.reversed().transformed(transform));
            }
            if self.side_open(axis, true) {
                result.add_direction(direction.transformed(transform));
            }
        }
    }
}
