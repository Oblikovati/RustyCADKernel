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

use super::{BoxReal, CompactBox3, triple};
use crate::gp::{Axis1, Axis3, Xyz};
impl<T: BoxReal> CompactBox3<T> {
    /// Tests a solid or hollow sphere with native strict boundaries; `bounds.is_out_sphere(center,radius,false)`.
    pub fn is_out_sphere(&self, center: Xyz, radius: f64, hollow: bool) -> bool {
        self.ball_out(*center.coordinates(), radius, hollow)
    }
    /// Compares signed distances of opposite corners; `bounds.is_out_plane(frame)`.
    pub fn is_out_plane(&self, plane: Axis3) -> bool {
        if self.is_void() {
            return true;
        }
        let normal = Xyz::from(plane.direction());
        let distance =
            (triple(self.center.map(BoxReal::to_double)) - plane.location().xyz()).dot(normal);
        let half = self.half.map(BoxReal::to_double);
        let extent =
            half[0] * normal.x().abs() + half[1] * normal.y().abs() + half[2] * normal.z().abs();
        (distance + extent) * (distance - extent) > 0.0
    }
    /// Clips native parameter intervals with optional ray and overthickness checks; `bounds.is_out_line(axis,false,0.0)`.
    pub fn is_out_line(&self, axis: Axis1, ray: bool, overthickness: f64) -> bool {
        if self.is_void() {
            return true;
        }
        let difference = triple(self.center.map(BoxReal::to_double)) - axis.location().xyz();
        let half = self.half.map(|h| h.to_double() + overthickness);
        line_out(
            *difference.coordinates(),
            *axis.direction().coordinates(),
            half,
            ray,
        )
    }
}
const RESOLUTION: f64 = f64::MIN_POSITIVE * 100.0;
fn axis_interval(difference: f64, direction: f64, half: f64) -> Option<[f64; 2]> {
    if direction > RESOLUTION {
        return Some([
            (difference - half) / direction,
            (difference + half) / direction,
        ]);
    }
    if direction < -RESOLUTION {
        return Some([
            (difference + half) / direction,
            (difference - half) / direction,
        ]);
    }
    if difference.abs() > half {
        return None;
    }
    Some([-f64::MAX, f64::MAX])
}
fn line_out(difference: [f64; 3], direction: [f64; 3], half: [f64; 3], ray: bool) -> bool {
    let Some(mut first) = axis_interval(difference[0], direction[0], half[0]) else {
        return true;
    };
    let Some(second) = axis_interval(difference[1], direction[1], half[1]) else {
        return true;
    };
    if disjoint(first, second) {
        return true;
    }
    if second[0] > first[0] {
        first[0] = second[0];
    }
    if second[1] < first[1] {
        first[1] = second[1];
    }
    if ray && first[1] < -RESOLUTION {
        return true;
    }
    if !(direction[2] > RESOLUTION || direction[2] < -RESOLUTION) {
        return difference[2].abs() > half[2];
    }
    let Some(last) = axis_interval(difference[2], direction[2], half[2]) else {
        return true;
    };
    (ray && last[1] < -RESOLUTION) || disjoint(first, last)
}
fn disjoint(first: [f64; 2], second: [f64; 2]) -> bool {
    first[0] > second[1] + RESOLUTION || first[1] < second[0] - RESOLUTION
}
