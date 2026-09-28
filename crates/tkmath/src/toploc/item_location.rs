// Created on: 1993-01-14
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/TopLoc/TopLoc_ItemLocation.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::Datum3;
use crate::gp::Transform3;
use rustycad_tkernel::standard::{
    Failure, Handle,
    dump::{self, DumpOutput, DumpStream},
};
use std::io;
/// `TopLoc_ItemLocation`: one datum raised to a power, with its cached chain product; `ItemLocation::new(datum, 2)?`.
#[derive(Clone)]
pub struct ItemLocation {
    pub(super) datum: Handle<Datum3>,
    pub(super) power: i32,
    pub(super) trsf: Transform3,
}
impl ItemLocation {
    /// Stores `datum^power`; a singular datum with a negative power fails like `gp_Trsf::Powered`.
    /// `ItemLocation::new(Handle::new(Datum3::default()), -1)?`.
    pub fn new(datum: Handle<Datum3>, power: i32) -> Result<Self, Failure> {
        let trsf = datum.transformation().powered(power)?;
        Ok(Self { datum, power, trsf })
    }
    /// Writes the cached transformation, the datum and the power, in native field order;
    /// `item.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "TopLoc_ItemLocation")?;
        if depth != 0 {
            let mut field = DumpStream::default();
            self.trsf.dump_json(&mut field)?;
            dump::dump_key_to_class(output, "Trsf", field.text())?;
            let mut datum = DumpStream::default();
            self.datum.dump_json(&mut datum, depth - 1)?;
            dump::dump_key_to_class(output, "Datum", datum.text())?;
        }
        dump::dump_integer(output, "Power", self.power)
    }
}
