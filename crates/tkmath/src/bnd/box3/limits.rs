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
use super::super::text_dump::{self, DumpSide};
use std::io;

/// Gap-adjusted limits in the native `Bnd_Box::Limits` field order (Xmin, Xmax, Ymin, ...).
///
/// Open sides read ±1e100. Example: `let BoxLimits3 { x_min, x_max, .. } = bounds.limits();`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxLimits3 {
    /// Minimum X coordinate.
    pub x_min: f64,
    /// Maximum X coordinate.
    pub x_max: f64,
    /// Minimum Y coordinate.
    pub y_min: f64,
    /// Maximum Y coordinate.
    pub y_max: f64,
    /// Minimum Z coordinate.
    pub z_min: f64,
    /// Maximum Z coordinate.
    pub z_max: f64,
}

impl BoundingBox3 {
    /// `Update(X,Y,Z)`: takes the point on a void box, otherwise widens each side with
    /// `std::min`/`std::max` ordering; `bounds.update_point(1.0, 2.0, 3.0)`.
    pub fn update_point(&mut self, x: f64, y: f64, z: f64) {
        self.update_bounds([x, y, z], [x, y, z]);
    }
    /// Unchecked structured `Get()`: a void box yields its raw sentinels minus/plus the gap;
    /// `bounds.limits().x_min`.
    pub fn limits(&self) -> BoxLimits3 {
        BoxLimits3 {
            x_min: self.x_min(),
            x_max: self.x_max(),
            y_min: self.y_min(),
            y_max: self.y_max(),
            z_min: self.z_min(),
            z_max: self.z_max(),
        }
    }
    /// `GetXMin`: -1e100 when open, else `Xmin - Gap`; `bounds.x_min()`.
    pub fn x_min(&self) -> f64 {
        self.bound(0, false)
    }
    /// `GetXMax`: 1e100 when open, else `Xmax + Gap`; `bounds.x_max()`.
    pub fn x_max(&self) -> f64 {
        self.bound(0, true)
    }
    /// `GetYMin`: -1e100 when open, else `Ymin - Gap`; `bounds.y_min()`.
    pub fn y_min(&self) -> f64 {
        self.bound(1, false)
    }
    /// `GetYMax`: 1e100 when open, else `Ymax + Gap`; `bounds.y_max()`.
    pub fn y_max(&self) -> f64 {
        self.bound(1, true)
    }
    /// `GetZMin`: -1e100 when open, else `Zmin - Gap`; `bounds.z_min()`.
    pub fn z_min(&self) -> f64 {
        self.bound(2, false)
    }
    /// `GetZMax`: 1e100 when open, else `Zmax + Gap`; `bounds.z_max()`.
    pub fn z_max(&self) -> f64 {
        self.bound(2, true)
    }
    pub(in crate::bnd) fn bound(&self, axis: usize, upper: bool) -> f64 {
        if self.side_open(axis, upper) {
            return if upper { 1e100 } else { -1e100 };
        }
        if upper {
            self.maximum[axis] + self.gap
        } else {
            self.minimum[axis] - self.gap
        }
    }
    /// `Dump()`: writes the raw stored sides (without gap) and the gap as the native
    /// console text; `bounds.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, output: &mut impl io::Write) -> io::Result<()> {
        let labels = ["Xmin", "Xmax", "Ymin", "Ymax", "Zmin", "Zmax"];
        let sides: [DumpSide; 6] = std::array::from_fn(|index| {
            let (axis, upper) = (index / 2, index % 2 == 1);
            let stored = if upper {
                self.maximum[axis]
            } else {
                self.minimum[axis]
            };
            (
                labels[index],
                (!self.side_open(axis, upper)).then_some(stored),
            )
        });
        let state = (self.is_void(), self.is_whole());
        text_dump::write_box_text(output, "Box3D : ", state, &sides, self.gap)
    }
}
