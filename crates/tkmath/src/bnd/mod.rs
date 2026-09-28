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

//! Native axis-aligned bounding volumes.
use crate::gp::Point3;
use rustycad_tkernel::standard::{
    Failure,
    dump::{self, DumpOutput},
};
use std::io;
/// Void/open flags, corners, and independent gap; `BoundingBox3::new(minimum,maximum)`.
#[derive(Clone, Debug)]
pub struct BoundingBox3 {
    minimum: [f64; 3],
    maximum: [f64; 3],
    gap: f64,
    flags: i32,
}
impl Default for BoundingBox3 {
    fn default() -> Self {
        Self {
            minimum: [f64::MAX; 3],
            maximum: [-f64::MAX; 3],
            gap: 0.0,
            flags: 1,
        }
    }
}
impl BoundingBox3 {
    /// Stores the supplied bounds with no gap; `BoundingBox3::new(min,max)`.
    pub const fn new(min: Point3, max: Point3) -> Self {
        Self {
            minimum: *min.coordinates(),
            maximum: *max.coordinates(),
            gap: 0.0,
            flags: 0,
        }
    }
    /// Uses the absolute tolerance as gap; `bounds.set_gap(1.5)`.
    pub fn set_gap(&mut self, gap: f64) {
        self.gap = gap.abs();
    }
    /// Returns the gap; `bounds.gap()`.
    pub const fn gap(&self) -> f64 {
        self.gap
    }
    /// Reports the native void flag; `bounds.is_void()`.
    pub const fn is_void(&self) -> bool {
        self.flags & 1 != 0
    }
    /// Returns xmin/ymin/zmin/xmax/ymax/zmax including gaps; `bounds.get()?`.
    pub fn get(&self) -> Result<[f64; 6], Failure> {
        if self.is_void() {
            return Err(Failure::with_type(
                "Standard_ConstructionError",
                "Bnd_Box is void",
                "",
            ));
        }
        Ok(std::array::from_fn(|index| {
            self.bound(index % 3, index >= 3)
        }))
    }
    /// Writes corners, gap and flags in native field order; `bounds.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "CornerMin", &self.minimum)?;
        dump::dump_vector(output, "CornerMax", &self.maximum)?;
        dump::dump_real(output, "Gap", self.gap)?;
        dump::dump_integer(output, "Flags", self.flags)
    }
    /// Reads fields with the original partial-write and cursor behavior; `bounds.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        if !dump::init_vector(text, "CornerMin", &mut local, &mut self.minimum)?
            || !dump::init_vector(text, "CornerMax", &mut local, &mut self.maximum)?
        {
            return Ok(false);
        }
        if !dump::init_real_field(text, "Gap", &mut local, &mut self.gap)?
            || !dump::init_integer_field(text, "Flags", &mut local, &mut self.flags)?
        {
            return Ok(false);
        }
        *position = local;
        Ok(true)
    }
}

mod range;
pub use range::{IntersectStatus, Range1};

mod box3;
pub use box3::BoxLimits3;

mod text_dump;

mod box2;
pub use box2::{BoundingBox2, BoxLimits2};

mod sphere;
pub use sphere::BoundingSphere;

mod compact;
pub use compact::{BoxReal, CompactBox2, CompactBox3};

mod obb;
pub use obb::{ObbHalfSizes, OrientedBoundingBox};

mod tools;
pub use tools::{box2_to_bvh, box3_to_bvh};
