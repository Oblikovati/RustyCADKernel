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
use super::{native_max, native_min};
use crate::gp::Line3;
impl BoundingBox3 {
    /// Retains OCCT's final coordinate rechecks after parameter clipping; `bounds.is_out_line(line)`.
    pub fn is_out_line(&self, line: Line3) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        let bounds = self.get().expect("nonvoid bounds");
        let origin = *line.location().coordinates();
        let direction = *line.direction().coordinates();
        let Some((minimum, maximum)) = clip_parameters(bounds, origin, direction) else {
            return true;
        };
        [2, 0, 1]
            .into_iter()
            .any(|axis| coordinate_out(bounds, origin, direction, axis, minimum, maximum))
    }
}

fn clip_parameters(bounds: [f64; 6], origin: [f64; 3], direction: [f64; 3]) -> Option<(f64, f64)> {
    let mut interval = (-1e100, 1e100);
    for axis in 0..3 {
        if direction[axis].abs() > 0.0 {
            if !clip_axis(bounds, origin, direction, axis, &mut interval) {
                return None;
            }
        } else if origin[axis] < bounds[axis] || bounds[axis + 3] < origin[axis] {
            return None;
        }
    }
    Some(interval)
}
fn clip_axis(
    bounds: [f64; 6],
    origin: [f64; 3],
    direction: [f64; 3],
    axis: usize,
    interval: &mut (f64, f64),
) -> bool {
    let first = (bounds[axis] - origin[axis]) / direction[axis];
    let second = (bounds[axis + 3] - origin[axis]) / direction[axis];
    let low = native_min(first, second);
    let high = native_max(first, second);
    if axis == 0 {
        *interval = (low, high);
        return true;
    }
    if interval.1 < low || interval.0 > high {
        return false;
    }
    *interval = (native_max(interval.0, low), native_min(interval.1, high));
    true
}
fn coordinate_out(
    bounds: [f64; 6],
    origin: [f64; 3],
    direction: [f64; 3],
    axis: usize,
    low: f64,
    high: f64,
) -> bool {
    let (minimum, maximum) = if direction[axis].abs() > 0.0 {
        let first = origin[axis] + low * direction[axis];
        let second = origin[axis] + high * direction[axis];
        (native_min(first, second), native_max(first, second))
    } else {
        (origin[axis], origin[axis])
    };
    maximum < bounds[axis] || bounds[axis + 3] < minimum
}
