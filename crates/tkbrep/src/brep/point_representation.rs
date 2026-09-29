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
// Rust translation of ModelingData/TKBRep/BRep/BRep_PointRepresentation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_PointRepresentation`: the root of the vertex representations (docs/topology-design.md section 9).
use super::dump::{dump_class_name, dump_location};
use super::errors::undefined_member;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private `myLocation` and `myParameter` of `BRep_PointRepresentation`.
#[derive(Clone, Default)]
struct PointFields {
    location: Location,
    parameter: f64,
}

/// The data of `BRep_PointRepresentation`, embedded by every point representation.
///
/// ```
/// use rustycad_tkbrep::brep::PointRepresentationBase;
/// use rustycad_tkmath::toploc::Location;
/// let base = PointRepresentationBase::new(0.5, &Location::default());
/// assert_eq!(base.parameter(), 0.5);
/// ```
#[derive(Default)]
pub struct PointRepresentationBase {
    fields: SharedState<PointFields>,
}

impl PointRepresentationBase {
    /// The protected `BRep_PointRepresentation(P, L)`; `PointRepresentationBase::new(0.0, &location)`.
    pub fn new(parameter: f64, location: &Location) -> Self {
        Self {
            fields: SharedState::new(PointFields {
                location: location.clone(),
                parameter,
            }),
        }
    }
    /// `Location()`; `base.location()`.
    pub fn location(&self) -> Location {
        self.fields.read().location.clone()
    }
    /// `Location(L)`; `base.set_location(&location)`.
    pub fn set_location(&self, location: &Location) {
        self.fields.write().location = location.clone();
    }
    /// `Parameter()`; `base.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.fields.read().parameter
    }
    /// `Parameter(P)`; `base.set_parameter(0.5)`.
    pub fn set_parameter(&self, parameter: f64) {
        self.fields.write().parameter = parameter;
    }
}

/// The class name the base members name in their Standard_DomainError.
const CLASS: &str = "BRep_PointRepresentation";

/// `BRep_PointRepresentation`: a parameter of a vertex on a curve or a surface. The
/// kind queries answer false and the accessors raise Standard_DomainError unless a
/// derived class overrides them.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepPointOnCurve, BRepPointRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let point = BRepPointOnCurve::new(0.5, &Handle::default(), &Location::default());
/// assert!(point.is_point_on_curve() && point.parameter2().is_err());
/// ```
pub trait BRepPointRepresentation: Transient {
    /// The embedded `BRep_PointRepresentation` data.
    fn point_representation_base(&self) -> &PointRepresentationBase;

    /// `IsPointOnCurve()`; `point.is_point_on_curve()`.
    fn is_point_on_curve(&self) -> bool {
        false
    }
    /// `IsPointOnCurveOnSurface()`; `point.is_point_on_curve_on_surface()`.
    fn is_point_on_curve_on_surface(&self) -> bool {
        false
    }
    /// `IsPointOnSurface()`; `point.is_point_on_surface()`.
    fn is_point_on_surface(&self) -> bool {
        false
    }
    /// `IsPointOnCurve(C, L)`; `point.is_point_on_curve_at(&curve, &location)`.
    fn is_point_on_curve_at(&self, _curve: &Handle<dyn GeomCurve>, _location: &Location) -> bool {
        false
    }
    /// `IsPointOnCurveOnSurface(PC, S, L)`;
    /// `point.is_point_on_curve_on_surface_at(&pcurve, &surface, &location)`.
    fn is_point_on_curve_on_surface_at(
        &self,
        _pcurve: &Handle<dyn Geom2dCurve>,
        _surface: &Handle<dyn GeomSurface>,
        _location: &Location,
    ) -> bool {
        false
    }
    /// `IsPointOnSurface(S, L)`; `point.is_point_on_surface_at(&surface, &location)`.
    fn is_point_on_surface_at(
        &self,
        _surface: &Handle<dyn GeomSurface>,
        _location: &Location,
    ) -> bool {
        false
    }

    /// `Location()` (non-virtual); `point.location()`.
    fn location(&self) -> Location {
        self.point_representation_base().location()
    }
    /// `Location(L)` (non-virtual); `point.set_location(&location)`.
    fn set_location(&self, location: &Location) {
        self.point_representation_base().set_location(location);
    }
    /// `Parameter()` (non-virtual); `point.parameter()`.
    fn parameter(&self) -> f64 {
        self.point_representation_base().parameter()
    }
    /// `Parameter(P)` (non-virtual); `point.set_parameter(0.5)`.
    fn set_parameter(&self, parameter: f64) {
        self.point_representation_base().set_parameter(parameter);
    }
    /// `Parameter2()`; `point.parameter2()?`.
    fn parameter2(&self) -> Result<f64, Failure> {
        Err(undefined_member(CLASS, "Parameter2"))
    }
    /// `Parameter2(P)`; `point.set_parameter2(0.5)?`.
    fn set_parameter2(&self, _parameter: f64) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Parameter2(P)"))
    }
    /// `Curve()`; `point.curve()?`.
    fn curve(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        Err(undefined_member(CLASS, "Curve"))
    }
    /// `Curve(C)`; `point.set_curve(&curve)?`.
    fn set_curve(&self, _curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Curve(C)"))
    }
    /// `PCurve()`; `point.pcurve()?`.
    fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Err(undefined_member(CLASS, "PCurve"))
    }
    /// `PCurve(C)`; `point.set_pcurve(&pcurve)?`.
    fn set_pcurve(&self, _pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "PCurve(C)"))
    }
    /// `Surface()`; `point.surface()?`.
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Err(undefined_member(CLASS, "Surface"))
    }
    /// `Surface(S)`; `point.set_surface(&surface)?`.
    fn set_surface(&self, _surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        Err(undefined_member(CLASS, "Surface(S)"))
    }
    /// `DumpJson(theOStream, theDepth)`; `point.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_point_representation(self.point_representation_base(), stream, depth)
    }
}
impl_transient_interface!(
    dyn BRepPointRepresentation,
    "BRep_PointRepresentation",
    StandardTransient
);

/// The body of `BRep_PointRepresentation::DumpJson`: the class name, the location and
/// the parameter; `dump_point_representation(base, &mut stream, -1)?`.
pub fn dump_point_representation(
    base: &PointRepresentationBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    dump_class_name(stream, CLASS)?;
    let fields = base.fields.read().clone();
    dump_location(stream, depth, "Location", &fields.location)?;
    dump::dump_real(stream, "Parameter", fields.parameter)
}
