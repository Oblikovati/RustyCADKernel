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
// Rust translation of ModelingData/TKBRep/BRep/BRep_CurveOnSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_CurveOnSurface`: a curve in the parametric space of a surface.
use super::curve_representation::{BRepCurveRepresentation, CurveRepresentationBase};
use super::dump::{dump_base, dump_class_name, dump_field, dump_handle};
use super::errors::null_object;
use super::gcurve::{BRepGCurve, GCurveBase, dump_gcurve};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The locked fields of `BRep_CurveOnSurface`: the protected `myUV1`, `myUV2` and the
/// private curve and surface.
#[derive(Clone)]
struct CurveOnSurfaceFields {
    uv1: Point2,
    uv2: Point2,
    pcurve: Handle<dyn Geom2dCurve>,
    surface: Handle<dyn GeomSurface>,
}

/// `BRep_CurveOnSurface`: a 2D curve on a located surface, with the UV points of its
/// range ends (recomputed by `Update`).
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveOnSurface, BRepCurveRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkmath::gp::{Axis2d, Axis3};
/// use rustycad_tkmath::toploc::Location;
/// let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
/// let pcurve = BRepCurveOnSurface::new(&line, &plane, &Location::default()).unwrap();
/// assert!(pcurve.is_curve_on_surface_at(&plane, &Location::default()));
/// ```
pub struct BRepCurveOnSurface {
    transient: StandardTransient,
    gcurve: GCurveBase,
    fields: SharedState<CurveOnSurfaceFields>,
}
impl_transient!(BRepCurveOnSurface, "BRep_CurveOnSurface", dyn BRepGCurve, transient,
    interfaces: [dyn BRepCurveRepresentation, dyn BRepGCurve]);

impl BRepCurveOnSurface {
    /// `BRep_CurveOnSurface(PC, S, L)`: the range is the p-curve range. A null p-curve
    /// raises Standard_NullObject (a null dereference natively);
    /// `BRepCurveOnSurface::new(&pcurve, &surface, &location)?`.
    pub fn new(
        pcurve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<Self, Failure> {
        let curve = pcurve
            .get()
            .ok_or_else(|| null_object("BRep_CurveOnSurface: the p-curve is null"))?;
        Ok(Self {
            transient: StandardTransient::default(),
            gcurve: GCurveBase::new(location, curve.first_parameter(), curve.last_parameter()),
            fields: SharedState::new(CurveOnSurfaceFields {
                uv1: Point2::default(),
                uv2: Point2::default(),
                pcurve: pcurve.clone(),
                surface: surface.clone(),
            }),
        })
    }

    /// `SetUVPoints(P1, P2)`; `pcurve.set_uv_points(first, last)`.
    pub fn set_uv_points(&self, first: Point2, last: Point2) {
        let mut fields = self.fields.write();
        fields.uv1 = first;
        fields.uv2 = last;
    }

    /// `UVPoints(P1, P2)`; `let (first, last) = pcurve.uv_points();`.
    pub fn uv_points(&self) -> (Point2, Point2) {
        let fields = self.fields.read();
        (fields.uv1, fields.uv2)
    }

    /// The body of `BRep_CurveOnSurface::Update`: the UV points at the finite range ends.
    pub(crate) fn update_uv_points(&self) -> Result<(), Failure> {
        let (first, last) = self.gcurve.range();
        let pcurve = self.fields.read().pcurve.clone();
        let curve = pcurve
            .get()
            .ok_or_else(|| null_object("BRep_CurveOnSurface::Update: the p-curve is null"))?;
        if !precision::is_negative_infinite(first) {
            self.fields.write().uv1 = curve.d0(first)?;
        }
        if !precision::is_positive_infinite(last) {
            self.fields.write().uv2 = curve.d0(last)?;
        }
        Ok(())
    }

    /// The body of `BRep_CurveOnSurface::DumpJson`, for the derived closed-surface class.
    pub(crate) fn dump_body(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_CurveOnSurface")?;
        dump_base(stream, depth, |nested, nested_depth| {
            dump_gcurve(&self.gcurve, nested, nested_depth)
        })?;
        let fields = self.fields.read().clone();
        dump_field(stream, depth, "UV1", |nested, _| {
            fields.uv1.dump_json(nested)
        })?;
        dump_field(stream, depth, "UV2", |nested, _| {
            fields.uv2.dump_json(nested)
        })?;
        dump_handle(stream, depth, "PCurve", &fields.pcurve, |c, s, d| {
            c.dump_json(s, d)
        })?;
        dump_handle(stream, depth, "Surface", &fields.surface, |c, s, d| {
            c.dump_json(s, d)
        })
    }
}

impl BRepCurveRepresentation for BRepCurveOnSurface {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        self.gcurve.representation()
    }
    /// Returns true.
    fn is_curve_on_surface(&self) -> bool {
        true
    }
    /// `(S == mySurface) && (L == myLocation)`.
    fn is_curve_on_surface_at(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        *surface == self.fields.read().surface && *location == self.location()
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(self.fields.read().surface.clone())
    }
    fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(self.fields.read().pcurve.clone())
    }
    fn set_pcurve(&self, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.fields.write().pcurve = pcurve.clone();
        Ok(())
    }
    /// A new representation on the same data, then `SetRange` and `SetUVPoints`.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let fields = self.fields.read().clone();
        let copy = Self::new(&fields.pcurve, &fields.surface, &self.location())?;
        let (first, last) = self.range();
        copy.set_range(first, last)?;
        copy.set_uv_points(fields.uv1, fields.uv2);
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        self.dump_body(stream, depth)
    }
}

impl BRepGCurve for BRepCurveOnSurface {
    fn gcurve_base(&self) -> &GCurveBase {
        &self.gcurve
    }
    /// The p-curve point, evaluated on the surface and moved by the location.
    fn d0(&self, u: f64) -> Result<Point3, Failure> {
        let fields = self.fields.read().clone();
        let pcurve = fields
            .pcurve
            .get()
            .ok_or_else(|| null_object("BRep_CurveOnSurface::D0: the p-curve is null"))?;
        let surface = fields
            .surface
            .get()
            .ok_or_else(|| null_object("BRep_CurveOnSurface::D0: the surface is null"))?;
        let uv = pcurve.value(u)?;
        let point = surface.value(uv.x(), uv.y())?;
        Ok(point.transformed(self.location().transformation()))
    }
    /// `BRep_CurveOnSurface::Update`: the UV points at the finite range ends.
    fn update(&self) -> Result<(), Failure> {
        self.update_uv_points()
    }
}
