// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge flag and range members of `BRep_Builder`.
use super::edge::locked_checked_edge;
use super::update_curves::{as_gcurve, update_curve3d};
use super::{BRepBuilder, relative_to};
use crate::brep::BRepTool;
use crate::brep::errors::domain_error;
use crate::topods::{Edge, Face, TShape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;

impl BRepBuilder {
    /// `SameParameter(E, S)`; `builder.same_parameter(&edge, false)?`.
    pub fn same_parameter(&self, edge: &Edge, same: bool) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::SameParameter")?;
        tedge.set_same_parameter(same);
        tedge.set_modified(true);
        Ok(())
    }

    /// `SameRange(E, S)`; `builder.same_range(&edge, false)?`.
    pub fn same_range(&self, edge: &Edge, same: bool) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::SameRange")?;
        tedge.set_same_range(same);
        tedge.set_modified(true);
        Ok(())
    }

    /// `Degenerated(E, D)`: sets the flag and, when set, a null 3D curve. The null curve
    /// is stored at the edge location itself (not relative to it), as natively;
    /// `builder.degenerated(&edge, true)?`.
    pub fn degenerated(&self, edge: &Edge, degenerated: bool) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::Degenerated")?;
        tedge.set_degenerated(degenerated);
        if degenerated {
            let location = edge.location().clone();
            tedge.change_curves(|curves| update_curve3d(curves, &Handle::default(), &location))?;
        }
        tedge.set_modified(true);
        Ok(())
    }

    /// `Range(E, First, Last, Only3d)`: sets the range of the 3D curve (`only3d`) or of
    /// every parametric representation; `builder.range(&edge, 0.0, 1.0, false)?`.
    pub fn range(&self, edge: &Edge, first: f64, last: f64, only3d: bool) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::Range")?;
        for representation in tedge.curves() {
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            if !only3d || gcurve.is_curve3d() {
                gcurve.set_range(first, last)?;
            }
        }
        tedge.set_modified(true);
        Ok(())
    }

    /// `Range(E, S, L, First, Last)`: sets the range of the p-curve on (`surface`,
    /// `location`); without one it raises Standard_DomainError ("no pcurve");
    /// `builder.range_on_surface(&edge, &surface, &location, 0.0, 1.0)?`.
    pub fn range_on_surface(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let tedge = locked_checked_edge(edge, "BRep_Builder::Range")?;
        let relative = relative_to(location, edge)?;
        let found = tedge
            .curves()
            .into_iter()
            .map(|r| as_gcurve(&r))
            .find(|gcurve| {
                gcurve
                    .get()
                    .is_some_and(|g| g.is_curve_on_surface_at(surface, &relative))
            });
        let Some(gcurve) = found.as_ref().and_then(|g| g.get()) else {
            return Err(domain_error(
                "BRep_Builder::Range, no pcurve",
                "no p-curve on the surface at the location",
            ));
        };
        gcurve.set_range(first, last)?;
        tedge.set_modified(true);
        Ok(())
    }

    /// `Range(E, F, First, Last)`: the range of the p-curve on the face's located surface;
    /// `builder.range_on_face(&edge, &face, 0.0, 1.0)?`.
    pub fn range_on_face(
        &self,
        edge: &Edge,
        face: &Face,
        first: f64,
        last: f64,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.range_on_surface(edge, &surface, &location, first, last)
    }
}
