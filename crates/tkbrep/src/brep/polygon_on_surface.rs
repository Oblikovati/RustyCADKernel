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
// Rust translation of ModelingData/TKBRep/BRep/BRep_PolygonOnSurface.hxx/.cxx, BRep_PolygonOnClosedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_PolygonOnSurface` and `BRep_PolygonOnClosedSurface`: 2D polygons of an edge
//! in the parametric space of a surface.
use super::curve_representation::{
    BRepCurveRepresentation, CurveRepresentationBase, dump_curve_representation,
};
use super::dump::{dump_base, dump_class_name, dump_handle};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::Polygon2D;
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private fields of `BRep_PolygonOnSurface`.
#[derive(Clone)]
struct PolygonOnSurfaceFields {
    polygon: Handle<Polygon2D>,
    surface: Handle<dyn GeomSurface>,
}

/// `BRep_PolygonOnSurface`: a 2D polygon on a located surface.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepPolygonOnSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let polygon = BRepPolygonOnSurface::new(&Handle::default(), &Handle::default(), &Location::default());
/// assert!(polygon.is_polygon_on_surface_at(&Handle::default(), &Location::default()));
/// ```
pub struct BRepPolygonOnSurface {
    transient: StandardTransient,
    representation: CurveRepresentationBase,
    fields: SharedState<PolygonOnSurfaceFields>,
}
impl_transient!(BRepPolygonOnSurface, "BRep_PolygonOnSurface", dyn BRepCurveRepresentation,
    transient, interfaces: [dyn BRepCurveRepresentation]);

impl BRepPolygonOnSurface {
    /// `BRep_PolygonOnSurface(P, S, L)`; `BRepPolygonOnSurface::new(&polygon, &surface, &location)`.
    pub fn new(
        polygon: &Handle<Polygon2D>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            representation: CurveRepresentationBase::new(location),
            fields: SharedState::new(PolygonOnSurfaceFields {
                polygon: polygon.clone(),
                surface: surface.clone(),
            }),
        }
    }

    /// The body of `BRep_PolygonOnSurface::DumpJson`, for the derived closed-surface class.
    fn dump_body(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PolygonOnSurface")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_curve_representation(&self.representation, nested, nested_depth)
        })?;
        let fields = self.fields.read().clone();
        dump_handle(stream, depth, "Polygon2D", &fields.polygon, |p, s, _| {
            p.dump_json(s)
        })?;
        dump_handle(stream, depth, "Surface", &fields.surface, |c, s, d| {
            c.dump_json(s, d)
        })
    }
}

impl BRepCurveRepresentation for BRepPolygonOnSurface {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.representation
    }
    /// Returns true.
    fn is_polygon_on_surface(&self) -> bool {
        true
    }
    /// `(S == mySurface) && (L == myLocation)`.
    fn is_polygon_on_surface_at(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        *surface == self.fields.read().surface && *location == self.location()
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.fields.read().surface.clone())
    }
    fn polygon(&self) -> Result<Handle<Polygon2D>, Failure> {
        Ok(self.fields.read().polygon.clone())
    }
    fn set_polygon(&self, polygon: &Handle<Polygon2D>) -> Result<(), Failure> {
        self.fields.write().polygon = polygon.clone();
        Ok(())
    }
    /// A new representation on the same polygon, surface and location.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let fields = self.fields.read().clone();
        let copy = Self::new(&fields.polygon, &fields.surface, &self.location());
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.dump_body(stream, depth)
    }
}

/// `BRep_PolygonOnClosedSurface`: the two 2D polygons of a seam edge. It derives from
/// `BRep_PolygonOnSurface`, which it embeds.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepPolygonOnClosedSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let none = Handle::default();
/// let seam = BRepPolygonOnClosedSurface::new(&none, &none, &Handle::default(), &Location::default());
/// assert!(seam.is_polygon_on_closed_surface() && seam.polygon2().unwrap().is_null());
/// ```
pub struct BRepPolygonOnClosedSurface {
    parent: BRepPolygonOnSurface,
    polygon2: SharedState<Handle<Polygon2D>>,
}
impl_transient!(BRepPolygonOnClosedSurface, "BRep_PolygonOnClosedSurface", BRepPolygonOnSurface,
    parent, interfaces: [dyn BRepCurveRepresentation]);

impl BRepPolygonOnClosedSurface {
    /// `BRep_PolygonOnClosedSurface(P1, P2, S, L)`;
    /// `BRepPolygonOnClosedSurface::new(&p1, &p2, &surface, &location)`.
    pub fn new(
        polygon1: &Handle<Polygon2D>,
        polygon2: &Handle<Polygon2D>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Self {
        Self {
            parent: BRepPolygonOnSurface::new(polygon1, surface, location),
            polygon2: SharedState::new(polygon2.clone()),
        }
    }
}

impl BRepCurveRepresentation for BRepPolygonOnClosedSurface {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        &self.parent.representation
    }
    fn is_polygon_on_surface(&self) -> bool {
        self.parent.is_polygon_on_surface()
    }
    fn is_polygon_on_surface_at(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        self.parent.is_polygon_on_surface_at(surface, location)
    }
    /// Returns true.
    fn is_polygon_on_closed_surface(&self) -> bool {
        true
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.parent.surface()
    }
    fn polygon(&self) -> Result<Handle<Polygon2D>, Failure> {
        self.parent.polygon()
    }
    fn set_polygon(&self, polygon: &Handle<Polygon2D>) -> Result<(), Failure> {
        self.parent.set_polygon(polygon)
    }
    fn polygon2(&self) -> Result<Handle<Polygon2D>, Failure> {
        Ok(self.polygon2.read().clone())
    }
    fn set_polygon2(&self, polygon: &Handle<Polygon2D>) -> Result<(), Failure> {
        *self.polygon2.write() = polygon.clone();
        Ok(())
    }
    /// A new representation on the same polygons, surface and location.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let copy = Self::new(
            &self.parent.polygon()?,
            &self.polygon2.read(),
            &self.parent.surface()?,
            &self.location(),
        );
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PolygonOnClosedSurface")?;
        dump_base(stream, depth, |nested, nested_depth| {
            self.parent.dump_body(nested, nested_depth)
        })?;
        let polygon2 = self.polygon2.read().clone();
        dump_handle(stream, depth, "Polygon2", &polygon2, |p, s, _| {
            p.dump_json(s)
        })
    }
}
