// Created on: 1993-08-10
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_PointOnCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_PointOnCurve`: the parameter of a vertex on a 3D curve.
use super::dump::{dump_base, dump_class_name, dump_pointer};
use super::point_representation::{
    BRepPointRepresentation, PointRepresentationBase, dump_point_representation,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::toploc::Location;
use std::io;

/// `BRep_PointOnCurve`: a parameter on a located 3D curve.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepPointOnCurve, BRepPointRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let point = BRepPointOnCurve::new(2.0, &Handle::default(), &Location::default());
/// assert!(point.is_point_on_curve_at(&Handle::default(), &Location::default()));
/// ```
pub struct BRepPointOnCurve {
    transient: StandardTransient,
    point: PointRepresentationBase,
    curve: SharedState<Handle<dyn GeomCurve>>,
}
impl_transient!(BRepPointOnCurve, "BRep_PointOnCurve", dyn BRepPointRepresentation, transient,
    interfaces: [dyn BRepPointRepresentation]);

impl BRepPointOnCurve {
    /// `BRep_PointOnCurve(P, C, L)`; `BRepPointOnCurve::new(0.5, &curve, &location)`.
    pub fn new(parameter: f64, curve: &Handle<dyn GeomCurve>, location: &Location) -> Self {
        Self {
            transient: StandardTransient::default(),
            point: PointRepresentationBase::new(parameter, location),
            curve: SharedState::new(curve.clone()),
        }
    }
}

impl BRepPointRepresentation for BRepPointOnCurve {
    fn point_representation_base(&self) -> &PointRepresentationBase {
        &self.point
    }
    /// Returns true.
    fn is_point_on_curve(&self) -> bool {
        true
    }
    /// `(myCurve == C) && (Location() == L)`.
    fn is_point_on_curve_at(&self, curve: &Handle<dyn GeomCurve>, location: &Location) -> bool {
        *self.curve.read() == *curve && self.location() == *location
    }
    fn curve(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.curve.read().clone())
    }
    fn set_curve(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        *self.curve.write() = curve.clone();
        Ok(())
    }
    /// The base dump, then the curve as a pointer (`OCCT_DUMP_FIELD_VALUE_NUMERICAL`).
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PointOnCurve")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_point_representation(&self.point, nested, nested_depth)
        })?;
        dump_pointer(stream, "Curve", self.curve.read().object_address())
    }
}
