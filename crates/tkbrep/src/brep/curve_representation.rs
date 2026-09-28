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
// Rust translation of ModelingData/TKBRep/BRep/BRep_CurveRepresentation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_CurveRepresentation`: the root of the edge representations (docs/topology-design.md section 9).
use super::dump::{dump_class_name, dump_location};
use super::errors::undefined_member;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The protected `myLocation` of `BRep_CurveRepresentation`, embedded by every
/// representation class.
///
/// ```
/// use rustycad_tkbrep::brep::CurveRepresentationBase;
/// use rustycad_tkmath::toploc::Location;
/// let base = CurveRepresentationBase::new(&Location::default());
/// assert!(base.location().is_identity());
/// ```
#[derive(Default)]
pub struct CurveRepresentationBase {
    location: SharedState<Location>,
}

impl CurveRepresentationBase {
    /// The protected `BRep_CurveRepresentation(L)`; `CurveRepresentationBase::new(&location)`.
    pub fn new(location: &Location) -> Self {
        Self {
            location: SharedState::new(location.clone()),
        }
    }

    /// `Location()`; `base.location()`.
    pub fn location(&self) -> Location {
        self.location.read().clone()
    }

    /// `Location(L)`; `base.set_location(&location)`.
    pub fn set_location(&self, location: &Location) {
        *self.location.write() = location.clone();
    }
}

/// The class name the base members name in their Standard_DomainError.
const CLASS: &str = "BRep_CurveRepresentation";

/// `BRep_CurveRepresentation`: a representation of the geometry of an edge. The
/// kind queries answer false and the accessors raise Standard_DomainError unless a
/// derived class overrides them; `Copy` is pure virtual.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurve3D, BRepCurveRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let representation = BRepCurve3D::new(&Handle::default(), &Location::default());
/// assert!(representation.is_curve3d() && !representation.is_polygon3d());
/// assert!(representation.surface().is_err());
/// ```
pub trait BRepCurveRepresentation: Transient {
    /// The embedded `BRep_CurveRepresentation` data.
    fn curve_representation_base(&self) -> &CurveRepresentationBase;

    /// `IsCurve3D()`: a 3D curve representation; `representation.is_curve3d()`.
    fn is_curve3d(&self) -> bool {
        false
    }
    /// `IsCurveOnSurface()`: a curve in the parametric space of a surface.
    fn is_curve_on_surface(&self) -> bool {
        false
    }
    /// `IsRegularity()`: a continuity between two surfaces.
    fn is_regularity(&self) -> bool {
        false
    }
    /// `IsCurveOnClosedSurface()`: two parametric curves on the same surface.
    fn is_curve_on_closed_surface(&self) -> bool {
        false
    }
    /// `IsCurveOnSurface(S, L)`: a curve in the parametric space of `surface` with
    /// `location`; `representation.is_curve_on_surface_at(&surface, &location)`.
    fn is_curve_on_surface_at(
        &self,
        _surface: &Handle<dyn GeomSurface>,
        _location: &Location,
    ) -> bool {
        false
    }
    /// `IsRegularity(S1, S2, L1, L2)`: a regularity between the two located surfaces.
    fn is_regularity_between(
        &self,
        _surface1: &Handle<dyn GeomSurface>,
        _surface2: &Handle<dyn GeomSurface>,
        _location1: &Location,
        _location2: &Location,
    ) -> bool {
        false
    }
    /// `IsPolygon3D()`: a 3D polygon representation.
    fn is_polygon3d(&self) -> bool {
        false
    }
    /// `IsPolygonOnTriangulation()`: an array of nodes on a triangulation.
    fn is_polygon_on_triangulation(&self) -> bool {
        false
    }
    /// `IsPolygonOnTriangulation(T, L)`: a polygon on `triangulation` with `location`.
    fn is_polygon_on_triangulation_at(
        &self,
        _triangulation: &Handle<Triangulation>,
        _location: &Location,
    ) -> bool {
        false
    }
    /// `IsPolygonOnClosedTriangulation()`: two arrays of nodes on a triangulation.
    fn is_polygon_on_closed_triangulation(&self) -> bool {
        false
    }
    /// `IsPolygonOnSurface()`: a polygon in the parametric space of a surface.
    fn is_polygon_on_surface(&self) -> bool {
        false
    }
    /// `IsPolygonOnSurface(S, L)`: a polygon in the parametric space of `surface` with `location`.
    fn is_polygon_on_surface_at(
        &self,
        _surface: &Handle<dyn GeomSurface>,
        _location: &Location,
    ) -> bool {
        false
    }
    /// `IsPolygonOnClosedSurface()`: two 2D polygons in the parametric space of a surface.
    fn is_polygon_on_closed_surface(&self) -> bool {
        false
    }

    /// `Location()` (non-virtual); `representation.location()`.
    fn location(&self) -> Location {
        self.curve_representation_base().location()
    }
    /// `Location(L)` (non-virtual); `representation.set_location(&location)`.
    fn set_location(&self, location: &Location) {
        self.curve_representation_base().set_location(location);
    }

    /// `Curve3D()`; `representation.curve3d()?`.
    fn curve3d(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Err(undefined_member(CLASS, "Curve3D"))
    }
    /// `Curve3D(C)`; `representation.set_curve3d(&curve)?`.
    fn set_curve3d(&self, _curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Curve3D(C)"))
    }
    /// `Surface()`; `representation.surface()?`.
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Err(undefined_member(CLASS, "Surface"))
    }
    /// `PCurve()`; `representation.pcurve()?`.
    fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Err(undefined_member(CLASS, "PCurve"))
    }
    /// `PCurve(C)`; `representation.set_pcurve(&pcurve)?`.
    fn set_pcurve(&self, _pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "PCurve(C)"))
    }
    /// `PCurve2()`; `representation.pcurve2()?`.
    fn pcurve2(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Err(undefined_member(CLASS, "PCurve2"))
    }
    /// `PCurve2(C)`; `representation.set_pcurve2(&pcurve)?`.
    fn set_pcurve2(&self, _pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "PCurve2(C)"))
    }
    /// `Polygon3D()`; `representation.polygon3d()?`.
    fn polygon3d(&self) -> Result<Handle<Polygon3D>, Failure> {
        Err(undefined_member(CLASS, "Polygon3D"))
    }
    /// `Polygon3D(P)`; `representation.set_polygon3d(&polygon)?`.
    fn set_polygon3d(&self, _polygon: &Handle<Polygon3D>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Polygon3D(P)"))
    }
    /// `Polygon()`; `representation.polygon()?`.
    fn polygon(&self) -> Result<Handle<Polygon2D>, Failure> {
        Err(undefined_member(CLASS, "Polygon"))
    }
    /// `Polygon(P)`; `representation.set_polygon(&polygon)?`.
    fn set_polygon(&self, _polygon: &Handle<Polygon2D>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Polygon(P)"))
    }
    /// `Polygon2()`; `representation.polygon2()?`.
    fn polygon2(&self) -> Result<Handle<Polygon2D>, Failure> {
        Err(undefined_member(CLASS, "Polygon2"))
    }
    /// `Polygon2(P)`; `representation.set_polygon2(&polygon)?`.
    fn set_polygon2(&self, _polygon: &Handle<Polygon2D>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Polygon2(P)"))
    }
    /// `Triangulation()`; `representation.triangulation()?`.
    fn triangulation(&self) -> Result<Handle<Triangulation>, Failure> {
        Err(undefined_member(CLASS, "Triangulation"))
    }
    /// `PolygonOnTriangulation()`; `representation.polygon_on_triangulation()?`.
    fn polygon_on_triangulation(&self) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        Err(undefined_member(CLASS, "PolygonOnTriangulation"))
    }
    /// `PolygonOnTriangulation(P)`; `representation.set_polygon_on_triangulation(&polygon)?`.
    fn set_polygon_on_triangulation(
        &self,
        _polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "PolygonOnTriangulation(P)"))
    }
    /// `PolygonOnTriangulation2()`; `representation.polygon_on_triangulation2()?`.
    fn polygon_on_triangulation2(&self) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        Err(undefined_member(CLASS, "PolygonOnTriangulation2"))
    }
    /// `PolygonOnTriangulation2(P2)`; `representation.set_polygon_on_triangulation2(&polygon)?`.
    fn set_polygon_on_triangulation2(
        &self,
        _polygon: &Handle<PolygonOnTriangulation>,
    ) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "PolygonOnTriangulation2(P2)"))
    }
    /// `Surface2()`; `representation.surface2()?`.
    fn surface2(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Err(undefined_member(CLASS, "Surface2"))
    }
    /// `Location2()`; `representation.location2()?`.
    fn location2(&self) -> Result<Location, Failure> {
        Err(undefined_member(CLASS, "Location2"))
    }
    /// `Continuity()`; `representation.continuity()?`.
    fn continuity(&self) -> Result<Continuity, Failure> {
        Err(undefined_member(CLASS, "Continuity"))
    }
    /// `Continuity(C)`; `representation.set_continuity(Continuity::C1)?`.
    fn set_continuity(&self, _continuity: Continuity) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Continuity(C)"))
    }

    /// `Copy()`: a new representation with the same data. It returns `Result` because
    /// copying a parametric representation sets its range, which evaluates its curves;
    /// `representation.copy()?`.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure>;

    /// `DumpJson(theOStream, theDepth)`; `representation.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_curve_representation(self.curve_representation_base(), stream, depth)
    }
}
impl_transient_interface!(
    dyn BRepCurveRepresentation,
    "BRep_CurveRepresentation",
    StandardTransient
);

/// The body of `BRep_CurveRepresentation::DumpJson`: the class name and the location;
/// `dump_curve_representation(base, &mut stream, -1)?`.
pub fn dump_curve_representation(
    base: &CurveRepresentationBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    dump_class_name(stream, CLASS)?;
    dump_location(stream, depth, "Location", &base.location())
}
