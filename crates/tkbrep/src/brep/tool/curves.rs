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

//! The edge curve queries of `BRep_Tool`: the 3D curve, the 3D polygon and the p-curves.
use super::{BRepTool, EdgeCurve3D, EdgeCurveOnSurface, EdgePCurve};
use crate::brep::access::edge_tshape;
use crate::brep::builder::update_curves::as_gcurve;
use crate::brep::{BRepCurve3D, BRepCurveRepresentation, BRepGCurve};
use crate::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::poly::Polygon3D;
use rustycad_tkmath::toploc::Location;

/// The representations of an edge, in list order.
pub(super) fn representations(
    edge: &Edge,
    member: &str,
) -> Result<Vec<Handle<dyn BRepCurveRepresentation>>, Failure> {
    Ok(edge_tshape(edge, member)?.curves())
}

/// The first representation accepted by `matches`, as a parametric curve.
pub(super) fn first_gcurve(
    edge: &Edge,
    member: &str,
    matches: impl Fn(&dyn BRepCurveRepresentation) -> bool,
) -> Result<Option<Handle<dyn BRepGCurve>>, Failure> {
    let found = representations(edge, member)?
        .into_iter()
        .find(|representation| representation.get().is_some_and(&matches));
    Ok(found.map(|representation| as_gcurve(&representation)))
}

impl BRepTool {
    /// `Curve(E, L, First, Last)`: the first 3D curve representation, with the edge
    /// location composed with its location; `BRepTool::curve_with_location(&edge)?`.
    pub fn curve_with_location(edge: &Edge) -> Result<EdgeCurve3D, Failure> {
        let found = first_gcurve(edge, "BRep_Tool::Curve", |r| r.is_curve3d())?;
        let Some(gcurve) = found.as_ref().and_then(|g| g.get()) else {
            return Ok(EdgeCurve3D::default());
        };
        let (first, last) = gcurve.range();
        Ok(EdgeCurve3D {
            curve: gcurve.curve3d()?,
            location: (edge.location() * &gcurve.location())?,
            first,
            last,
        })
    }

    /// `Curve(E, First, Last)`: the 3D curve, transformed into a copy when its location
    /// is not the identity; `let (curve, first, last) = BRepTool::curve(&edge)?;`.
    pub fn curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
        let found = Self::curve_with_location(edge)?;
        let Some(curve) = found.curve.get() else {
            return Ok((found.curve.clone(), found.first, found.last));
        };
        if found.location.is_identity() {
            return Ok((found.curve.clone(), found.first, found.last));
        }
        let copy = curve.transformed(found.location.transformation())?;
        Ok((copy.cast::<dyn GeomCurve>(), found.first, found.last))
    }

    /// `IsGeometric(E)`: the edge has a non-null 3D curve or a curve on surface (the
    /// search stops at the first of the two kinds that answers yes);
    /// `BRepTool::is_geometric_edge(&edge)?`.
    pub fn is_geometric_edge(edge: &Edge) -> Result<bool, Failure> {
        for representation in representations(edge, "BRep_Tool::IsGeometric")? {
            let Some(r) = representation.get() else {
                continue;
            };
            if r.is_curve3d() {
                let curve3d = representation.cast::<BRepCurve3D>();
                if curve3d
                    .get()
                    .is_some_and(|c| c.curve3d().is_ok_and(|c| c.is_some()))
                {
                    return Ok(true);
                }
            } else if r.is_curve_on_surface() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `Polygon3D(E, L)`: the first 3D polygon and the composed location, or a null
    /// polygon at the identity; `BRepTool::polygon3d(&edge)?`.
    pub fn polygon3d(edge: &Edge) -> Result<(Handle<Polygon3D>, Location), Failure> {
        for representation in representations(edge, "BRep_Tool::Polygon3D")? {
            let Some(r) = representation.get().filter(|r| r.is_polygon3d()) else {
                continue;
            };
            return Ok((r.polygon3d()?, (edge.location() * &r.location())?));
        }
        Ok((Handle::default(), Location::default()))
    }

    /// `CurveOnSurface(E, F, First, Last, theIsStored)`: the p-curve on the face's
    /// surface, for the edge reversed when the face is REVERSED;
    /// `BRepTool::curve_on_face(&edge, &face)?`.
    pub fn curve_on_face(edge: &Edge, face: &Face) -> Result<EdgePCurve, Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        let mut local = edge.clone();
        if face.orientation() == Orientation::Reversed {
            local.reverse();
        }
        Self::curve_on_surface(&local, &surface, &location)
    }

    /// `CurveOnSurface(E, S, L, First, Last, theIsStored)`: the stored p-curve on
    /// (`surface`, `location`), the second one of a seam for a REVERSED edge; when none
    /// is stored, the planar projection of `CurveOnPlane` (`is_stored` false);
    /// `BRepTool::curve_on_surface(&edge, &surface, &location)?`.
    pub fn curve_on_surface(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<EdgePCurve, Failure> {
        let list = representations(edge, "BRep_Tool::CurveOnSurface")?;
        if !list.is_empty() {
            let relative = location.predivided(edge.location())?;
            let found = list.iter().find(|r| {
                r.get()
                    .is_some_and(|r| r.is_curve_on_surface_at(surface, &relative))
            });
            if let Some(representation) = found {
                return stored_pcurve(representation, edge.orientation() == Orientation::Reversed);
            }
        }
        let (pcurve, first, last) = Self::curve_on_plane(edge, surface, location)?;
        Ok(EdgePCurve {
            pcurve,
            first,
            last,
            is_stored: false,
        })
    }

    /// `CurveOnSurface(E, C, S, L, First, Last)`: the first curve on surface of the
    /// edge; `BRepTool::first_curve_on_surface(&edge)?`.
    pub fn first_curve_on_surface(edge: &Edge) -> Result<EdgeCurveOnSurface, Failure> {
        let found = first_gcurve(edge, "BRep_Tool::CurveOnSurface", |r| {
            r.is_curve_on_surface()
        })?;
        let Some(gcurve) = found.as_ref().and_then(|g| g.get()) else {
            return Ok(EdgeCurveOnSurface::default());
        };
        curve_on_surface_of(edge, gcurve, gcurve.pcurve()?)
    }

    /// `CurveOnSurface(E, C, S, L, First, Last, Index)`: the p-curve of rank `index`,
    /// counting both p-curves of a seam. An index below 1 returns the null result
    /// here; natively the outputs are left unchanged;
    /// `BRepTool::curve_on_surface_by_index(&edge, 1)?`.
    pub fn curve_on_surface_by_index(
        edge: &Edge,
        index: i32,
    ) -> Result<EdgeCurveOnSurface, Failure> {
        if index < 1 {
            return Ok(EdgeCurveOnSurface::default());
        }
        let mut rank = 0;
        for representation in representations(edge, "BRep_Tool::CurveOnSurface")? {
            let Some(r) = representation.get().filter(|r| r.is_curve_on_surface()) else {
                continue;
            };
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            rank += 1;
            if rank == index {
                return curve_on_surface_of(edge, gcurve, r.pcurve()?);
            }
            if r.is_curve_on_closed_surface() {
                rank += 1;
                if rank == index {
                    return curve_on_surface_of(edge, gcurve, r.pcurve2()?);
                }
            }
        }
        Ok(EdgeCurveOnSurface::default())
    }
}

/// The p-curve and range of a found curve on surface: the second p-curve of a seam
/// for a REVERSED edge.
fn stored_pcurve(
    representation: &Handle<dyn BRepCurveRepresentation>,
    reversed: bool,
) -> Result<EdgePCurve, Failure> {
    let gcurve = as_gcurve(representation);
    let Some(gcurve) = gcurve.get() else {
        return Ok(EdgePCurve::default());
    };
    let (first, last) = gcurve.range();
    let pcurve = if gcurve.is_curve_on_closed_surface() && reversed {
        gcurve.pcurve2()?
    } else {
        gcurve.pcurve()?
    };
    Ok(EdgePCurve {
        pcurve,
        first,
        last,
        is_stored: true,
    })
}

/// The outputs of the `CurveOnSurface(E, C, S, L, ...)` overloads for a found curve.
fn curve_on_surface_of(
    edge: &Edge,
    gcurve: &dyn BRepGCurve,
    pcurve: Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<EdgeCurveOnSurface, Failure> {
    let (first, last) = gcurve.range();
    Ok(EdgeCurveOnSurface {
        pcurve,
        surface: gcurve.surface()?,
        location: (edge.location() * &gcurve.location())?,
        first,
        last,
    })
}
