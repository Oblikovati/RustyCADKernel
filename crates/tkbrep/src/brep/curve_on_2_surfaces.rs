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
// Rust translation of ModelingData/TKBRep/BRep/BRep_CurveOn2Surfaces.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_CurveOn2Surfaces`: the continuity of an edge between two surfaces.
use super::curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
use super::dump::{dump_base, dump_class_name, dump_handle, dump_location};
use super::errors::null_object;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private fields of `BRep_CurveOn2Surfaces`.
#[derive(Clone)]
struct TwoSurfacesFields {
    surface: Handle<dyn GeomSurface>,
    surface2: Handle<dyn GeomSurface>,
    location2: Location,
    continuity: Continuity,
}

/// `BRep_CurveOn2Surfaces`: the regularity of an edge between two located surfaces.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveOn2Surfaces, BRepCurveRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::toploc::Location;
/// let none = Handle::default();
/// let regularity = BRepCurveOn2Surfaces::new(&none, &none, &Location::default(), &Location::default(), Shape::G1);
/// assert_eq!(regularity.continuity().unwrap(), Shape::G1);
/// ```
pub struct BRepCurveOn2Surfaces {
    transient: StandardTransient,
    representation: CurveRepresentationBase,
    fields: SharedState<TwoSurfacesFields>,
}
impl_transient!(BRepCurveOn2Surfaces, "BRep_CurveOn2Surfaces", dyn BRepCurveRepresentation,
    transient, interfaces: [dyn BRepCurveRepresentation]);

impl BRepCurveOn2Surfaces {
    /// `BRep_CurveOn2Surfaces(S1, S2, L1, L2, C)`; `BRepCurveOn2Surfaces::new(&s1, &s2, &l1, &l2, Shape::C1)`.
    pub fn new(
        surface1: &Handle<dyn GeomSurface>,
        surface2: &Handle<dyn GeomSurface>,
        location1: &Location,
        location2: &Location,
        continuity: Continuity,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            representation: CurveRepresentationBase::new(location1),
            fields: SharedState::new(TwoSurfacesFields {
                surface: surface1.clone(),
                surface2: surface2.clone(),
                location2: location2.clone(),
                continuity,
            }),
        }
    }

    /// `D0(U, P)`: always raises Standard_NullObject ("BRep_CurveOn2Surfaces::D0"), as
    /// natively; `regularity.d0(0.0).is_err()`.
    pub fn d0(&self, _u: f64) -> Result<Point3, Failure> {
        Err(null_object("BRep_CurveOn2Surfaces::D0"))
    }
}

impl BRepCurveRepresentation for BRepCurveOn2Surfaces {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.representation
    }
    /// Returns true.
    fn is_regularity(&self) -> bool {
        true
    }
    /// The two located surfaces, in either order.
    fn is_regularity_between(
        &self,
        surface1: &Handle<dyn GeomSurface>,
        surface2: &Handle<dyn GeomSurface>,
        location1: &Location,
        location2: &Location,
    ) -> bool {
        let fields = self.fields.read().clone();
        let location = self.location();
        (fields.surface == *surface1
            && fields.surface2 == *surface2
            && location == *location1
            && fields.location2 == *location2)
            || (fields.surface == *surface2
                && fields.surface2 == *surface1
                && location == *location2
                && fields.location2 == *location1)
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.fields.read().surface.clone())
    }
    fn surface2(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.fields.read().surface2.clone())
    }
    fn location2(&self) -> Result<Location, Failure> {
        Ok(self.fields.read().location2.clone())
    }
    fn continuity(&self) -> Result<Continuity, Failure> {
        Ok(self.fields.read().continuity)
    }
    fn set_continuity(&self, continuity: Continuity) -> Result<(), Failure> {
        self.fields.write().continuity = continuity;
        Ok(())
    }
    /// A new representation on the same surfaces, locations and continuity.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let fields = self.fields.read().clone();
        let copy = Self::new(
            &fields.surface,
            &fields.surface2,
            &self.location(),
            &fields.location2,
            fields.continuity,
        );
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_CurveOn2Surfaces")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_curve_representation(&self.representation, nested, nested_depth)
        })?;
        let fields = self.fields.read().clone();
        dump_handle(stream, depth, "Surface", &fields.surface, |c, s, d| {
            c.dump_json(s, d)
        })?;
        dump_handle(stream, depth, "Surface2", &fields.surface2, |c, s, d| {
            c.dump_json(s, d)
        })?;
        dump_location(stream, depth, "Location2", &fields.location2)?;
        dump::dump_integer(stream, "Continuity", fields.continuity as i32)
    }
}
