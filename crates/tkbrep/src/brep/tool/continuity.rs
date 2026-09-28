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

//! The regularity queries of `BRep_Tool`.
use super::BRepTool;
use super::curves::representations;
use crate::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;

/// Two surfaces of a regularity.
pub type SurfacePair<'a> = (&'a Handle<dyn GeomSurface>, &'a Handle<dyn GeomSurface>);
/// The locations of the two surfaces of a regularity.
pub type LocationPair<'a> = (&'a Location, &'a Location);

impl BRepTool {
    /// `HasContinuity(E, F1, F2)`: the edge has a regularity between the faces' located
    /// surfaces; `BRepTool::has_continuity_on_faces(&edge, &f1, &f2)?`.
    pub fn has_continuity_on_faces(
        edge: &Edge,
        face1: &Face,
        face2: &Face,
    ) -> Result<bool, Failure> {
        let (surface1, location1) = Self::surface_with_location(face1)?;
        let (surface2, location2) = Self::surface_with_location(face2)?;
        Self::has_continuity_on_surfaces(edge, (&surface1, &surface2), (&location1, &location2))
    }

    /// `Continuity(E, F1, F2)`: the continuity between the faces, C0 without a
    /// regularity; `BRepTool::continuity_on_faces(&edge, &f1, &f2)?`.
    pub fn continuity_on_faces(
        edge: &Edge,
        face1: &Face,
        face2: &Face,
    ) -> Result<Continuity, Failure> {
        let (surface1, location1) = Self::surface_with_location(face1)?;
        let (surface2, location2) = Self::surface_with_location(face2)?;
        Self::continuity_on_surfaces(edge, (&surface1, &surface2), (&location1, &location2))
    }

    /// `HasContinuity(E, S1, S2, L1, L2)`: the edge has a regularity between the located
    /// surfaces; `BRepTool::has_continuity_on_surfaces(&edge, (&s1, &s2), (&l1, &l2))?`.
    pub fn has_continuity_on_surfaces(
        edge: &Edge,
        surfaces: SurfacePair<'_>,
        locations: LocationPair<'_>,
    ) -> Result<bool, Failure> {
        let relative1 = locations.0.predivided(edge.location())?;
        let relative2 = locations.1.predivided(edge.location())?;
        let list = representations(edge, "BRep_Tool::HasContinuity")?;
        Ok(list
            .iter()
            .filter_map(|r| r.get())
            .any(|r| r.is_regularity_between(surfaces.0, surfaces.1, &relative1, &relative2)))
    }

    /// `Continuity(E, S1, S2, L1, L2)`: the continuity of the first regularity between
    /// the located surfaces, C0 without one;
    /// `BRepTool::continuity_on_surfaces(&edge, (&s1, &s2), (&l1, &l2))?`.
    pub fn continuity_on_surfaces(
        edge: &Edge,
        surfaces: SurfacePair<'_>,
        locations: LocationPair<'_>,
    ) -> Result<Continuity, Failure> {
        let relative1 = locations.0.predivided(edge.location())?;
        let relative2 = locations.1.predivided(edge.location())?;
        let list = representations(edge, "BRep_Tool::Continuity")?;
        let found = list
            .iter()
            .filter_map(|r| r.get())
            .find(|r| r.is_regularity_between(surfaces.0, surfaces.1, &relative1, &relative2));
        found.map_or(Ok(Continuity::C0), |r| r.continuity())
    }

    /// `HasContinuity(E)`: the edge has a regularity on some two surfaces;
    /// `BRepTool::has_continuity(&edge)?`.
    pub fn has_continuity(edge: &Edge) -> Result<bool, Failure> {
        let list = representations(edge, "BRep_Tool::HasContinuity")?;
        Ok(list
            .iter()
            .filter_map(|r| r.get())
            .any(|r| r.is_regularity()))
    }

    /// `MaxContinuity(E)`: the highest continuity of the regularities, C0 without one;
    /// `BRepTool::max_continuity(&edge)?`.
    pub fn max_continuity(edge: &Edge) -> Result<Continuity, Failure> {
        let mut maximum = Continuity::C0;
        for representation in representations(edge, "BRep_Tool::MaxContinuity")? {
            let Some(r) = representation.get().filter(|r| r.is_regularity()) else {
                continue;
            };
            let continuity = r.continuity()?;
            if continuity as i32 > maximum as i32 {
                maximum = continuity;
            }
        }
        Ok(maximum)
    }
}
