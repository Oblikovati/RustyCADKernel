// Created on: 2013-12-20
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_Box.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH_Box::DumpJson` / `InitFromJson`: at most the first three components.
use super::BvhBox;
use crate::bvh::scalar::BvhScalar;
use crate::bvh::vector::BvhVector;
use rustycad_tkernel::standard::{
    Failure,
    dump::{self, DumpOutput},
};
use std::io;

/// Number of serialized components, `(N < 3) ? N : 3`.
fn dumped_count<V: BvhVector>() -> usize {
    V::DIMENSION.min(3)
}

fn dumped_values<V: BvhVector>(point: &V) -> Vec<f64> {
    (0..dumped_count::<V>())
        .map(|axis| point[axis].to_double())
        .collect()
}

impl<V: BvhVector> BvhBox<V> {
    /// Writes `IsValid`, `MinPoint` and `MaxPoint`; floats are promoted to double
    /// as the variadic dump promotes them. Example: `bounding.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_integer(output, "IsValid", i32::from(self.is_valid()))?;
        dump::dump_vector(output, "MinPoint", &dumped_values(&self.min_point))?;
        dump::dump_vector(output, "MaxPoint", &dumped_values(&self.max_point))
    }

    /// Reads a dump; `IsValid: 0` clears the box. Components past the third
    /// become the unbounded range `[lowest, max]`. A failed `MaxPoint` keeps the
    /// already assigned `MinPoint`, as upstream does.
    /// Example: `bounding.init_from_json(text, &mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        let mut is_valid = 0;
        if !dump::init_integer_field(text, "IsValid", &mut local, &mut is_valid)? {
            return Ok(false);
        }
        if is_valid == 0 {
            self.clear();
            *position = local;
            return Ok(true);
        }
        if !read_corner(text, "MinPoint", &mut local, &mut self.min_point)?
            || !read_corner(text, "MaxPoint", &mut local, &mut self.max_point)?
        {
            return Ok(false);
        }
        for axis in dumped_count::<V>()..V::DIMENSION {
            self.min_point[axis] = V::Scalar::LOWEST;
            self.max_point[axis] = V::Scalar::MAX;
        }
        *position = local;
        Ok(true)
    }
}

/// Parses doubles into locals and narrows them only after the vector succeeds.
fn read_corner<V: BvhVector>(
    text: &str,
    name: &str,
    position: &mut i32,
    corner: &mut V,
) -> Result<bool, Failure> {
    let mut values = [0.0; 3];
    let count = dumped_count::<V>();
    if !dump::init_vector(text, name, position, &mut values[..count])? {
        return Ok(false);
    }
    for (axis, value) in values.iter().take(count).enumerate() {
        corner[axis] = V::Scalar::from_double(*value);
    }
    Ok(true)
}
