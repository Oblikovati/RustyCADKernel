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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BoundedCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_BoundedCurve`: curves limited by two finite parameters.
use super::curve::{GeomCurve, dump_curve};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name};
use rustycad_tkmath::gp::Point3;
use std::io;

/// A curve with a start point and an end point; `curve.start_point()?`.
pub trait GeomBoundedCurve: GeomCurve {
    /// Point of the last parameter (`EndPoint`); `curve.end_point()?`.
    fn end_point(&self) -> Result<Point3, Failure>;
    /// Point of the first parameter (`StartPoint`); `curve.start_point()?`.
    fn start_point(&self) -> Result<Point3, Failure>;
}
impl_transient_interface!(dyn GeomBoundedCurve, "Geom_BoundedCurve", dyn GeomCurve);

/// `Geom_BoundedCurve::DumpJson`; `dump_bounded_curve(&mut stream, -1)?`.
pub fn dump_bounded_curve(stream: &mut DumpStream, depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom_BoundedCurve")?;
    dump_base(stream, depth, dump_curve)
}
