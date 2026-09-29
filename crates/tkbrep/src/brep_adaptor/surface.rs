// Created on: 1993-02-22
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
// Rust translation of ModelingData/TKBRep/BRepAdaptor/BRepAdaptor_Surface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepAdaptor_Surface`: a face seen as a surface.
use super::forward::inherited_surface_members;
use crate::brep::BRepTool;
use crate::brep_tools;
use crate::topods::Face;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedSurface;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};

/// A face seen as a surface (`BRepAdaptor_Surface`): the surface of the face,
/// restricted to the UV bounds of the face or not, transformed by the face location.
///
/// The native class derives from `GeomAdaptor_TransformedSurface`; the Rust type embeds
/// it as its parent field and forwards every inherited member.
///
/// ```
/// use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
/// let adaptor = BRepAdaptorSurface::default();
/// assert!(adaptor.face().is_null());
/// ```
#[derive(Default)]
pub struct BRepAdaptorSurface {
    parent: GeomAdaptorTransformedSurface,
    face: SharedState<Face>,
}
impl_transient!(
    BRepAdaptorSurface,
    "BRepAdaptor_Surface",
    GeomAdaptorTransformedSurface,
    parent,
    interfaces: [dyn Adaptor3dSurface]
);

impl BRepAdaptorSurface {
    /// `BRepAdaptor_Surface(F, R)`: `restriction` limits the surface to the UV bounds of
    /// the face; `BRepAdaptorSurface::with_face(&face, true)?`.
    pub fn with_face(face: &Face, restriction: bool) -> Result<Self, Failure> {
        let adaptor = Self::default();
        adaptor.initialize(face, restriction)?;
        Ok(adaptor)
    }

    /// The embedded parent adaptor (the `GeomAdaptor_TransformedSurface` base).
    pub fn transformed_surface(&self) -> &GeomAdaptorTransformedSurface {
        &self.parent
    }

    /// `Initialize(F, Restriction)`: a null face is ignored; otherwise the face is
    /// stored and, when it has a surface, the surface is loaded (on the face's UV bounds
    /// with `restriction`) with the face location; `adaptor.initialize(&face, true)?`.
    pub fn initialize(&self, face: &Face, restriction: bool) -> Result<(), Failure> {
        if face.is_null() {
            return Ok(());
        }
        *self.face.write() = face.clone();
        let (surface, location) = BRepTool::surface_with_location(face)?;
        if surface.is_null() {
            return Ok(());
        }
        let trsf = location.transformation();
        if !restriction {
            return self.parent.load(&surface, trsf);
        }
        let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(face)?;
        self.parent
            .load_range(&surface, ((umin, umax, vmin, vmax), (0.0, 0.0)), trsf)
    }

    /// `Face()`; `adaptor.face()`.
    pub fn face(&self) -> Face {
        self.face.read().clone()
    }

    /// `Tolerance()`: the tolerance of the face; `adaptor.tolerance()?`.
    pub fn tolerance(&self) -> Result<f64, Failure> {
        BRepTool::face_tolerance(&self.face())
    }
}

impl Adaptor3dSurface for BRepAdaptorSurface {
    /// `ShallowCopy()`: a shallow copy of the original adaptor, the transformation, the
    /// shared transformed adaptor and the face.
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor3dSurface>, Failure> {
        let copy = Self {
            parent: self.parent.shallow_adaptor()?,
            face: SharedState::new(self.face()),
        };
        Ok(Handle::new(copy).cast())
    }

    inherited_surface_members!();
}
