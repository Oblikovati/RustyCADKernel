// Created on: 2005-09-08
// Created by: Alexander GRIGORIEV
// Copyright (c) 2005-2014 OPEN CASCADE SAS
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

use super::{BoxReal, storage::CompactBounds};
use crate::gp::{Point2, Xy};
/// Native Bnd_B2<double/float> storage; `CompactBox2::<f64>::new(center,half_size)`.
pub type CompactBox2<T = f64> = CompactBounds<T, 2>;
impl<T: BoxReal> CompactBounds<T, 2> {
    /// Converts double coordinates to the storage scalar; `CompactBox2::<f32>::new(center,half)`.
    pub fn new(center: Xy, half: Xy) -> Self {
        Self {
            center: convert(center),
            half: convert(half),
        }
    }
    /// Replaces the center through native scalar conversion; `bounds.set_center(center)`.
    pub fn set_center(&mut self, center: Xy) {
        self.center = convert(center);
    }
    /// Retains supplied half sizes without validation; `bounds.set_half_size(half)`.
    pub fn set_half_size(&mut self, half: Xy) {
        self.half = convert(half);
    }
    /// Subtracts before promotion, retaining float rounding; `bounds.corner_min()`.
    pub fn corner_min(&self) -> Xy {
        Xy::new(
            (self.center[0] - self.half[0]).to_double(),
            (self.center[1] - self.half[1]).to_double(),
        )
    }
    /// Adds before promotion, retaining float rounding; `bounds.corner_max()`.
    pub fn corner_max(&self) -> Xy {
        Xy::new(
            (self.center[0] + self.half[0]).to_double(),
            (self.center[1] + self.half[1]).to_double(),
        )
    }
    /// Computes the squared diagonal in the storage scalar before promotion; `bounds.square_extent()`.
    pub fn square_extent(&self) -> f64 {
        (T::from_double(4.0) * (self.half[0] * self.half[0] + self.half[1] * self.half[1]))
            .to_double()
    }
    /// Adds a geometric point via coordinate algebra; `bounds.add_point(point)`.
    pub fn add_point(&mut self, point: Point2) {
        self.add_xy(point.xy());
    }
    /// Expands by a point in native scalar arithmetic; `bounds.add_xy(point)`.
    pub fn add_xy(&mut self, point: Xy) {
        self.add_array(convert(point));
    }
    /// Checks a converted point against half extents; `bounds.is_out_point(point)`.
    pub fn is_out_point(&self, point: Xy) -> bool {
        self.is_out_array(convert(point))
    }
}
fn convert<T: BoxReal>(xy: Xy) -> [T; 2] {
    [T::from_double(xy.x()), T::from_double(xy.y())]
}
mod queries;
