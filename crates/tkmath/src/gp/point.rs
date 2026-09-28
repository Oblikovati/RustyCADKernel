// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/gp/gp_Pnt.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::dot;
use rustycad_tkernel::standard::{
    Failure,
    dump::{self, DumpOutput},
};
use std::io;
/// A Cartesian point, initially the origin; `Point3::new(1.0,2.0,3.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    pub(crate) coordinates: [f64; 3],
}
impl Point3 {
    /// Constructs unmodified coordinates; `Point3::new(1.0,2.0,3.0)`.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            coordinates: [x, y, z],
        }
    }
    /// Returns X; `point.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates[0]
    }
    /// Returns Y; `point.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates[1]
    }
    /// Returns Z; `point.z()`.
    pub const fn z(&self) -> f64 {
        self.coordinates[2]
    }
    /// Borrows the coordinate triple; `point.coordinates()`.
    pub const fn coordinates(&self) -> &[f64; 3] {
        &self.coordinates
    }
    /// Computes native squared Euclidean distance; `point.square_distance(&other)`.
    pub fn square_distance(&self, other: &Self) -> f64 {
        let diff = std::array::from_fn(|index| self.coordinates[index] - other.coordinates[index]);
        dot(diff, diff)
    }
    /// Applies native distance/tolerance comparison; `point.is_equal(&other,1e-7)`.
    pub fn is_equal(&self, other: &Self, tolerance: f64) -> bool {
        self.square_distance(other).sqrt() <= tolerance
    }
    /// Writes the native JSON fragment; `point.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "gp_Pnt", &self.coordinates)
    }
    /// Parses a native fragment with one-based cursor and partial coordinate writes; `point.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        dump::init_vector(text, "gp_Pnt", position, &mut self.coordinates)
    }
}

mod operations;
