// Created on: 1995-03-15
// Created by: Laurent PAINNOT
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_PolygonOnTriangulation.hxx/.cxx, BRep_PolygonOnClosedTriangulation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_PolygonOnTriangulation` and `BRep_PolygonOnClosedTriangulation`: node index
//! arrays of an edge on a triangulation.
use super::curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
use super::dump::{dump_base, dump_class_name, dump_handle};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::poly::{PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private fields of `BRep_PolygonOnTriangulation`.
#[derive(Clone)]
struct OnTriangulationFields {
    polygon: Handle<PolygonOnTriangulation>,
    triangulation: Handle<Triangulation>,
}

/// `BRep_PolygonOnTriangulation`: an array of node indices on a located triangulation.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepPolygonOnTriangulation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let polygon = BRepPolygonOnTriangulation::new(&Handle::default(), &Handle::default(), &Location::default());
/// assert!(polygon.is_polygon_on_triangulation());
/// ```
pub struct BRepPolygonOnTriangulation {
    transient: StandardTransient,
    representation: CurveRepresentationBase,
    fields: SharedState<OnTriangulationFields>,
}
impl_transient!(BRepPolygonOnTriangulation, "BRep_PolygonOnTriangulation",
    dyn BRepCurveRepresentation, transient, interfaces: [dyn BRepCurveRepresentation]);

impl BRepPolygonOnTriangulation {
    /// `BRep_PolygonOnTriangulation(P, T, L)`;
    /// `BRepPolygonOnTriangulation::new(&polygon, &triangulation, &location)`.
    pub fn new(
        polygon: &Handle<PolygonOnTriangulation>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            representation: CurveRepresentationBase::new(location),
            fields: SharedState::new(OnTriangulationFields {
                polygon: polygon.clone(),
                triangulation: triangulation.clone(),
            }),
        }
    }

    /// The body of `BRep_PolygonOnTriangulation::DumpJson`, for the derived closed class.
    fn dump_body(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PolygonOnTriangulation")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_curve_representation(&self.representation, nested, nested_depth)
        })?;
        let fields = self.fields.read().clone();
        dump_handle(stream, depth, "Polygon", &fields.polygon, |p, s, _| {
            p.dump_json(s)
        })?;
        dump_handle(
            stream,
            depth,
            "Triangulation",
            &fields.triangulation,
            |t, s, _| t.dump_json(s),
        )
    }
}

impl BRepCurveRepresentation for BRepPolygonOnTriangulation {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.representation
    }
    /// Returns true.
    fn is_polygon_on_triangulation(&self) -> bool {
        true
    }
    /// `(T == myTriangulation) && (L == myLocation)`.
    fn is_polygon_on_triangulation_at(
        &self,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> bool {
        *triangulation == self.fields.read().triangulation && *location == self.location()
    }
    fn set_polygon_on_triangulation(
        &self,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        self.fields.write().polygon = polygon.clone();
        Ok(())
    }
    fn triangulation(&self) -> Result<Handle<Triangulation>, Failure> {
        Ok(self.fields.read().triangulation.clone())
    }
    fn polygon_on_triangulation(&self) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        Ok(self.fields.read().polygon.clone())
    }
    /// A new representation on the same polygon, triangulation and location.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let fields = self.fields.read().clone();
        let copy = Self::new(&fields.polygon, &fields.triangulation, &self.location());
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.dump_body(stream, depth)
    }
}

/// `BRep_PolygonOnClosedTriangulation`: the two node arrays of a seam edge on a
/// triangulation. It derives from `BRep_PolygonOnTriangulation`, which it embeds.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepPolygonOnClosedTriangulation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let none = Handle::default();
/// let seam = BRepPolygonOnClosedTriangulation::new(&none, &none, &Handle::default(), &Location::default());
/// assert!(seam.is_polygon_on_closed_triangulation());
/// ```
pub struct BRepPolygonOnClosedTriangulation {
    parent: BRepPolygonOnTriangulation,
    polygon2: SharedState<Handle<PolygonOnTriangulation>>,
}
impl_transient!(BRepPolygonOnClosedTriangulation, "BRep_PolygonOnClosedTriangulation",
    BRepPolygonOnTriangulation, parent, interfaces: [dyn BRepCurveRepresentation]);

impl BRepPolygonOnClosedTriangulation {
    /// `BRep_PolygonOnClosedTriangulation(P1, P2, T, L)`;
    /// `BRepPolygonOnClosedTriangulation::new(&p1, &p2, &triangulation, &location)`.
    pub fn new(
        polygon1: &Handle<PolygonOnTriangulation>,
        polygon2: &Handle<PolygonOnTriangulation>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Self {
        Self {
            parent: BRepPolygonOnTriangulation::new(polygon1, triangulation, location),
            polygon2: SharedState::new(polygon2.clone()),
        }
    }
}

impl BRepCurveRepresentation for BRepPolygonOnClosedTriangulation {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.parent.representation
    }
    fn is_polygon_on_triangulation(&self) -> bool {
        self.parent.is_polygon_on_triangulation()
    }
    fn is_polygon_on_triangulation_at(
        &self,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> bool {
        self.parent
            .is_polygon_on_triangulation_at(triangulation, location)
    }
    /// Returns true.
    fn is_polygon_on_closed_triangulation(&self) -> bool {
        true
    }
    fn set_polygon_on_triangulation(
        &self,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        self.parent.set_polygon_on_triangulation(polygon)
    }
    fn triangulation(&self) -> Result<Handle<Triangulation>, Failure> {
        self.parent.triangulation()
    }
    fn polygon_on_triangulation(&self) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        self.parent.polygon_on_triangulation()
    }
    fn polygon_on_triangulation2(&self) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        Ok(self.polygon2.read().clone())
    }
    fn set_polygon_on_triangulation2(
        &self,
        polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        *self.polygon2.write() = polygon.clone();
        Ok(())
    }
    /// A new representation on the same polygons, triangulation and location.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let copy = Self::new(
            &self.parent.polygon_on_triangulation()?,
            &self.polygon2.read(),
            &self.parent.triangulation()?,
            &self.location(),
        );
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PolygonOnClosedTriangulation")?;
        dump_base(stream, depth, |nested, nested_depth| {
            self.parent.dump_body(nested, nested_depth)
        })?;
        let polygon2 = self.polygon2.read().clone();
        dump_handle(stream, depth, "Polygon2", &polygon2, |p, s, _| {
            p.dump_json(s)
        })
    }
}
