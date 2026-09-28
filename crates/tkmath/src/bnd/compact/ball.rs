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
impl<T: BoxReal, const N: usize> CompactBounds<T, N> {
    pub(super) fn ball_out(&self, center: [f64; N], radius: f64, hollow: bool) -> bool {
        let distance: [f64; N] =
            std::array::from_fn(|i| (center[i] - self.center[i].to_double()).abs());
        let half = self.half.map(BoxReal::to_double);
        let near: [f64; N] = std::array::from_fn(|i| distance[i] - half[i]);
        let squared = positive_squared(near);
        if !hollow {
            return squared > radius * radius;
        }
        if squared < radius * radius {
            return self.far_inside(distance, half, radius);
        }
        true
    }
    fn far_inside(&self, distance: [f64; N], half: [f64; N], radius: f64) -> bool {
        let far: [f64; N] = std::array::from_fn(|i| distance[i] + half[i]);
        let squared = far
            .map(|v| v * v)
            .into_iter()
            .reduce(|a, b| a + b)
            .expect("2D or 3D box");
        squared.partial_cmp(&(radius * radius)) != Some(std::cmp::Ordering::Greater)
    }
}
fn positive_squared<const N: usize>(values: [f64; N]) -> f64 {
    let mut squared = 0.0;
    for value in values {
        if value > 0.0 {
            squared += value * value;
        }
    }
    squared
}
