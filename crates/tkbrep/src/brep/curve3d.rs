// Created on: 1993-07-06
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Curve3D.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_Curve3D`: the 3D curve of an edge.
use super::curve_representation::{BRepCurveRepresentation, CurveRepresentationBase};
use super::dump::{dump_base, dump_class_name, dump_handle};
use super::errors::null_object;
use super::gcurve::{BRepGCurve, GCurveBase, dump_gcurve};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use std::io;

/// `BRep_Curve3D`: a 3D curve with its location and range. A null curve is allowed
/// (degenerated edges); its range is then `RealFirst()..RealLast()`.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurve3D, BRepGCurve};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let curve = BRepCurve3D::new(&Handle::default(), &Location::default());
/// assert_eq!(curve.first(), -f64::MAX);
/// ```
pub struct BRepCurve3D {
    transient: StandardTransient,
    gcurve: GCurveBase,
    curve: SharedState<Handle<dyn GeomCurve>>,
}
impl_transient!(BRepCurve3D, "BRep_Curve3D", dyn BRepGCurve, transient,
    interfaces: [dyn BRepCurveRepresentation, dyn BRepGCurve]);

impl BRepCurve3D {
    /// `BRep_Curve3D(C, L)`: the range is the curve range, or `RealFirst()..RealLast()`
    /// for a null curve; `BRepCurve3D::new(&curve, &location)`.
    pub fn new(curve: &Handle<dyn GeomCurve>, location: &Location) -> Self {
        let (first, last) = curve.get().map_or((real_first(), real_last()), |c| {
            (c.first_parameter(), c.last_parameter())
        });
        Self {
            transient: StandardTransient::default(),
            gcurve: GCurveBase::new(location, first, last),
            curve: SharedState::new(curve.clone()),
        }
    }
}

impl BRepCurveRepresentation for BRepCurve3D {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        self.gcurve.representation()
    }
    /// Returns true.
    fn is_curve3d(&self) -> bool {
        true
    }
    fn curve3d(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(self.curve.read().clone())
    }
    fn set_curve3d(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        *self.curve.write() = curve.clone();
        Ok(())
    }
    /// A new `BRep_Curve3D` on the same curve and location, then `SetRange`.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let copy = Self::new(&self.curve.read(), &self.location());
        let (first, last) = self.range();
        copy.set_range(first, last)?;
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_Curve3D")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_gcurve(&self.gcurve, nested, nested_depth)
        })?;
        dump_handle(
            stream,
            depth,
            "Curve",
            &self.curve.read(),
            |curve, nested, d| curve.dump_json(nested, d),
        )
    }
}

impl BRepGCurve for BRepCurve3D {
    fn gcurve_base(&self) -> &GCurveBase {
        &self.gcurve
    }
    /// `P = myCurve->Value(U)`; a null curve raises Standard_NullObject (a null
    /// dereference natively).
    fn d0(&self, u: f64) -> Result<Point3, Failure> {
        let curve = self.curve.read().clone();
        let curve = curve
            .get()
            .ok_or_else(|| null_object("BRep_Curve3D::D0: the 3D curve is null"))?;
        curve.value(u)
    }
}
