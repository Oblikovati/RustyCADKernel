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
// Rust translation of ModelingData/TKBRep/BRep/BRep_PointsOnSurface.hxx/.cxx, BRep_PointOnSurface.hxx/.cxx, BRep_PointOnCurveOnSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_PointsOnSurface` and its derived classes `BRep_PointOnSurface` and
//! `BRep_PointOnCurveOnSurface`: parameters of a vertex on a surface.
use super::dump::{dump_base, dump_class_name, dump_handle, dump_pointer};
use super::point_representation::{
    BRepPointRepresentation, PointRepresentationBase, dump_point_representation,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;
use std::io;

/// The data of `BRep_PointsOnSurface`: the point data and the private `mySurface`.
///
/// ```
/// use rustycad_tkbrep::brep::PointsOnSurfaceBase;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let base = PointsOnSurfaceBase::new(1.0, &Handle::default(), &Location::default());
/// assert!(base.surface().is_null());
/// ```
#[derive(Default)]
pub struct PointsOnSurfaceBase {
    point: PointRepresentationBase,
    surface: SharedState<Handle<dyn GeomSurface>>,
}

impl PointsOnSurfaceBase {
    /// The protected `BRep_PointsOnSurface(P, S, L)`;
    /// `PointsOnSurfaceBase::new(0.0, &surface, &location)`.
    pub fn new(parameter: f64, surface: &Handle<dyn GeomSurface>, location: &Location) -> Self {
        Self {
            point: PointRepresentationBase::new(parameter, location),
            surface: SharedState::new(surface.clone()),
        }
    }
    /// The embedded `BRep_PointRepresentation` data; `base.point()`.
    pub fn point(&self) -> &PointRepresentationBase {
        &self.point
    }
    /// `Surface()`; `base.surface()`.
    pub fn surface(&self) -> Handle<dyn GeomSurface> {
        self.surface.read().clone()
    }
    /// `Surface(S)`; `base.set_surface(&surface)`.
    pub fn set_surface(&self, surface: &Handle<dyn GeomSurface>) {
        *self.surface.write() = surface.clone();
    }
}

/// `BRep_PointsOnSurface`: a point representation on a surface. Its constructor is
/// protected: only `BRep_PointOnSurface` and `BRep_PointOnCurveOnSurface` exist.
pub trait BRepPointsOnSurface: BRepPointRepresentation {
    /// The embedded `BRep_PointsOnSurface` data.
    fn points_on_surface_base(&self) -> &PointsOnSurfaceBase;
}
impl_transient_interface!(
    dyn BRepPointsOnSurface,
    "BRep_PointsOnSurface",
    dyn BRepPointRepresentation
);

/// The body of `BRep_PointsOnSurface::DumpJson`: the class name, the base class and
/// the surface; `dump_points_on_surface(base, &mut stream, -1)?`.
pub fn dump_points_on_surface(
    base: &PointsOnSurfaceBase,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    dump_class_name(stream, "BRep_PointsOnSurface")?;
    dump_base(stream, depth, |nested, nested_depth| {
        dump_point_representation(&base.point, nested, nested_depth)
    })?;
    dump_handle(stream, depth, "Surface", &base.surface(), |s, nested, d| {
        s.dump_json(nested, d)
    })
}

/// `BRep_PointOnSurface`: the (U, V) parameters of a vertex on a located surface.
/// It does not override `DumpJson`, so it dumps as `BRep_PointsOnSurface`.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepPointOnSurface, BRepPointRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let point = BRepPointOnSurface::new(1.0, 2.0, &Handle::default(), &Location::default());
/// assert_eq!((point.parameter(), point.parameter2().unwrap()), (1.0, 2.0));
/// ```
pub struct BRepPointOnSurface {
    transient: StandardTransient,
    base: PointsOnSurfaceBase,
    parameter2: SharedState<f64>,
}
impl_transient!(BRepPointOnSurface, "BRep_PointOnSurface", dyn BRepPointsOnSurface, transient,
    interfaces: [dyn BRepPointRepresentation, dyn BRepPointsOnSurface]);

impl BRepPointOnSurface {
    /// `BRep_PointOnSurface(P1, P2, S, L)`; `BRepPointOnSurface::new(u, v, &surface, &location)`.
    pub fn new(
        parameter1: f64,
        parameter2: f64,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: PointsOnSurfaceBase::new(parameter1, surface, location),
            parameter2: SharedState::new(parameter2),
        }
    }
}

impl BRepPointRepresentation for BRepPointOnSurface {
    fn point_representation_base(&self) -> &PointRepresentationBase {
        &self.base.point
    }
    /// Returns true.
    fn is_point_on_surface(&self) -> bool {
        true
    }
    /// `(Surface() == S) && (Location() == L)`.
    fn is_point_on_surface_at(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        self.base.surface() == *surface && self.location() == *location
    }
    fn parameter2(&self) -> Result<f64, Failure> {
        Ok(*self.parameter2.read())
    }
    fn set_parameter2(&self, parameter: f64) -> Result<(), Failure> {
        *self.parameter2.write() = parameter;
        Ok(())
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.base.surface())
    }
    fn set_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.base.set_surface(surface);
        Ok(())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_points_on_surface(&self.base, stream, depth)
    }
}

impl BRepPointsOnSurface for BRepPointOnSurface {
    fn points_on_surface_base(&self) -> &PointsOnSurfaceBase {
        &self.base
    }
}

/// `BRep_PointOnCurveOnSurface`: a parameter on a p-curve of a located surface.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepPointOnCurveOnSurface, BRepPointRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::toploc::Location;
/// let point = BRepPointOnCurveOnSurface::new(1.0, &Handle::default(), &Handle::default(), &Location::default());
/// assert!(point.is_point_on_curve_on_surface());
/// ```
pub struct BRepPointOnCurveOnSurface {
    transient: StandardTransient,
    base: PointsOnSurfaceBase,
    pcurve: SharedState<Handle<dyn Geom2dCurve>>,
}
impl_transient!(BRepPointOnCurveOnSurface, "BRep_PointOnCurveOnSurface", dyn BRepPointsOnSurface,
    transient, interfaces: [dyn BRepPointRepresentation, dyn BRepPointsOnSurface]);

impl BRepPointOnCurveOnSurface {
    /// `BRep_PointOnCurveOnSurface(P, C, S, L)`;
    /// `BRepPointOnCurveOnSurface::new(0.5, &pcurve, &surface, &location)`.
    pub fn new(
        parameter: f64,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: PointsOnSurfaceBase::new(parameter, surface, location),
            pcurve: SharedState::new(pcurve.clone()),
        }
    }
}

impl BRepPointRepresentation for BRepPointOnCurveOnSurface {
    fn point_representation_base(&self) -> &PointRepresentationBase {
        &self.base.point
    }
    /// Returns true.
    fn is_point_on_curve_on_surface(&self) -> bool {
        true
    }
    /// `(myPCurve == PC) && (Surface() == S) && (Location() == L)`.
    fn is_point_on_curve_on_surface_at(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        *self.pcurve.read() == *pcurve
            && self.base.surface() == *surface
            && self.location() == *location
    }
    fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(self.pcurve.read().clone())
    }
    fn set_pcurve(&self, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        *self.pcurve.write() = pcurve.clone();
        Ok(())
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.base.surface())
    }
    fn set_surface(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.base.set_surface(surface);
        Ok(())
    }
    /// The native dump skips `BRep_PointsOnSurface`: the class name, the
    /// `BRep_PointRepresentation` base, then the p-curve as a pointer.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_PointOnCurveOnSurface")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_point_representation(&self.base.point, nested, nested_depth)
        })?;
        dump_pointer(stream, "PCurve", self.pcurve.read().object_address())
    }
}

impl BRepPointsOnSurface for BRepPointOnCurveOnSurface {
    fn points_on_surface_base(&self) -> &PointsOnSurfaceBase {
        &self.base
    }
}
