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
use crate::gp::{Direction2, Point2, Transform2, TransformForm};
impl BoundingBox2 {
    /// Retains native void state and omits corners incident on an open side; `bounds.transformed(transform)`.
    pub fn transformed(&self, transform: Transform2) -> Self {
        if self.is_void() || transform.form() == TransformForm::Identity {
            return self.clone();
        }
        if transform.form() == TransformForm::Translation {
            return self.translated_bounds(transform);
        }
        let mut result = Self::default();
        self.transform_open_directions(transform, &mut result);
        for index in 0..4 {
            self.transform_corner(index, transform, &mut result);
        }
        result.gap = self.gap;
        result
    }
    fn translated_bounds(&self, transform: Transform2) -> Self {
        let mut result = self.clone();
        let offset = transform.translation_part();
        for (i, shift) in [offset.x(), offset.y()].into_iter().enumerate() {
            if !self.side_open(i, false) {
                result.minimum[i] += shift;
            }
            if !self.side_open(i, true) {
                result.maximum[i] += shift;
            }
        }
        result
    }
    fn transform_corner(&self, index: usize, transform: Transform2, result: &mut Self) {
        if (0..2).any(|axis| self.side_open(axis, index & (1 << axis) != 0)) {
            return;
        }
        let [x, y] = std::array::from_fn(|axis| {
            if index & (1 << axis) == 0 {
                self.minimum[axis]
            } else {
                self.maximum[axis]
            }
        });
        result.add_point(Point2::new(x, y).transformed(transform));
    }
    fn transform_open_directions(&self, transform: Transform2, result: &mut Self) {
        for (axis, direction) in [Direction2::X, Direction2::Y].into_iter().enumerate() {
            if self.side_open(axis, false) {
                result.add_direction(direction.reversed().transformed(transform));
            }
            if self.side_open(axis, true) {
                result.add_direction(direction.transformed(transform));
            }
        }
    }
}
