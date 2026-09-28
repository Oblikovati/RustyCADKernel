// Created on: 1993-07-07
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face queries of `BRep_Tool`.
use super::BRepTool;
use crate::brep::access::face_tshape;
use crate::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkmath::poly::mesh_purpose::MeshPurpose;
use rustycad_tkmath::toploc::Location;

/// `max(p, Precision::Confusion())`, written as the native `p > pMin ? p : pMin`.
pub(super) fn floored_tolerance(tolerance: f64) -> f64 {
    let minimum = precision::confusion();
    if tolerance > minimum {
        tolerance
    } else {
        minimum
    }
}

impl BRepTool {
    /// `Surface(F, L)`: the stored surface and the face location composed with the
    /// TFace location; `let (surface, location) = BRepTool::surface_with_location(&face)?;`.
    pub fn surface_with_location(
        face: &Face,
    ) -> Result<(Handle<dyn GeomSurface>, Location), Failure> {
        let tface = face_tshape(face, "BRep_Tool::Surface")?;
        let location = (face.location() * &tface.location())?;
        Ok((tface.surface(), location))
    }

    /// `Surface(F)`: the surface, transformed into a copy when the composed location
    /// is not the identity; `BRepTool::surface(&face)?`.
    pub fn surface(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
        let tface = face_tshape(face, "BRep_Tool::Surface")?;
        let surface = tface.surface();
        let Some(stored) = surface.get() else {
            return Ok(surface);
        };
        let location = (face.location() * &tface.location())?;
        if location.is_identity() {
            return Ok(surface);
        }
        Ok(stored
            .transformed(location.transformation())?
            .cast::<dyn GeomSurface>())
    }

    /// `Triangulation(F, L, thePurpose)`: the triangulation for the purpose and the
    /// face location (the TFace location is not composed);
    /// `BRepTool::triangulation(&face, mesh_purpose::NONE)?`.
    pub fn triangulation(
        face: &Face,
        purpose: MeshPurpose,
    ) -> Result<(Handle<Triangulation>, Location), Failure> {
        let tface = face_tshape(face, "BRep_Tool::Triangulation")?;
        Ok((tface.triangulation(purpose), face.location().clone()))
    }

    /// `Triangulations(F, L)`: every triangulation and the face location;
    /// `BRepTool::triangulations(&face)?`.
    pub fn triangulations(face: &Face) -> Result<(Vec<Handle<Triangulation>>, Location), Failure> {
        let tface = face_tshape(face, "BRep_Tool::Triangulations")?;
        Ok((tface.triangulations(), face.location().clone()))
    }

    /// `Tolerance(F)`: the stored tolerance, at least `Precision::Confusion()`;
    /// `BRepTool::face_tolerance(&face)?`.
    pub fn face_tolerance(face: &Face) -> Result<f64, Failure> {
        let tface = face_tshape(face, "BRep_Tool::Tolerance")?;
        Ok(floored_tolerance(tface.tolerance()))
    }

    /// `NaturalRestriction(F)`; `BRepTool::natural_restriction(&face)?`.
    pub fn natural_restriction(face: &Face) -> Result<bool, Failure> {
        Ok(face_tshape(face, "BRep_Tool::NaturalRestriction")?.natural_restriction())
    }

    /// `IsGeometric(F)`: the face has a surface; `BRepTool::is_geometric_face(&face)?`.
    pub fn is_geometric_face(face: &Face) -> Result<bool, Failure> {
        Ok(face_tshape(face, "BRep_Tool::IsGeometric")?
            .surface()
            .is_some())
    }
}
