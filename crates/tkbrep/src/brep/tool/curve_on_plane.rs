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

//! `BRep_Tool::CurveOnPlane`: the p-curve of an edge on a plane, built by projection.
//!
//! The projection needs GeomProjLib and ProjLib, which are not ported: every check
//! that runs before it is ported and returns the native results, and the projection
//! step itself raises Standard_NotImplemented naming the missing packages
//! (docs/tkbrep-progress.md, "Dependency gap").
use super::BRepTool;
use crate::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::toploc::Location;

/// The plane of `surface`: the surface itself or the basis of a rectangular trimmed
/// surface; null otherwise (the planar test of `CurveOnPlane` and `UVPoints`).
pub(super) fn plane_of(surface: &Handle<dyn GeomSurface>) -> Handle<GeomPlane> {
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    match trimmed.get() {
        Some(trimmed) => trimmed.basis_surface().cast::<GeomPlane>(),
        None => surface.cast::<GeomPlane>(),
    }
}

/// The anonymous-namespace `isValidTrimRange`: the ordered range is not empty and,
/// after unwrapping nested trimmed curves, the basis is periodic or holds the range
/// within `Precision::PConfusion()`.
fn is_valid_trim_range(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> bool {
    let trim_first = first.min(last);
    let trim_last = first.max(last);
    // `!(first < last)` also refuses NaN bounds.
    if trim_first.partial_cmp(&trim_last) != Some(std::cmp::Ordering::Less) {
        return false;
    }
    let mut basis = curve.clone();
    while let Some(trimmed) = basis.cast::<GeomTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    let Some(basis) = basis.get() else {
        return false;
    };
    basis.is_periodic()
        || (basis.first_parameter() - trim_first <= precision::p_confusion()
            && trim_last - basis.last_parameter() <= precision::p_confusion())
}

/// The 3D curve of an edge expressed in the frame of the plane.
struct CurveInPlaneFrame {
    /// The curve, transformed when the relative location is not the identity.
    curve: Handle<dyn GeomCurve>,
    /// Its range, mapped by the transformation.
    range: (f64, f64),
    /// The untransformed range, which `CurveOnPlane` reports.
    reported: (f64, f64),
}

/// The 3D curve of the edge in the plane's frame and its range, transformed (with the
/// parameters mapped) when the relative location is not the identity.
fn curve_in_plane_frame(
    edge: &Edge,
    location: &Location,
) -> Result<Option<CurveInPlaneFrame>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let Some(curve) = found.curve.get() else {
        return Ok(None);
    };
    let relative = found.location.predivided(location)?;
    let reported = (found.first, found.last);
    if relative.is_identity() {
        return Ok(Some(CurveInPlaneFrame {
            curve: found.curve.clone(),
            range: reported,
            reported,
        }));
    }
    let transform = relative.transformation();
    let moved = curve.transformed(transform)?.cast::<dyn GeomCurve>();
    let Some(moved_curve) = moved.get() else {
        return Ok(None);
    };
    let first = moved_curve.transformed_parameter(found.first, transform);
    let last = moved_curve.transformed_parameter(found.last, transform);
    Ok(Some(CurveInPlaneFrame {
        curve: moved.clone(),
        range: (first, last),
        reported,
    }))
}

impl BRepTool {
    /// `CurveOnPlane(E, S, L, First, Last)`: for a plane (or a rectangular trimmed
    /// plane) and an edge with a 3D curve whose range is valid, the p-curve built by
    /// projecting the trimmed 3D curve on the plane. A non-planar surface, an edge
    /// without a 3D curve and an invalid range return a null curve, with the range
    /// `0, 0` in the first two cases and the edge range in the last.
    ///
    /// The projection itself needs GeomProjLib::ProjectOnPlane and
    /// ProjLib_ProjectedCurve, which are not ported: that step raises
    /// Standard_NotImplemented. `let (pcurve, first, last) = BRepTool::curve_on_plane(&edge, &surface, &location)?;`.
    pub fn curve_on_plane(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
        let plane = plane_of(surface);
        if plane.is_null() {
            return Ok((Handle::default(), 0.0, 0.0));
        }
        let Some(framed) = curve_in_plane_frame(edge, location)? else {
            return Ok((Handle::default(), 0.0, 0.0));
        };
        let (curve, (first, last), reported) = (framed.curve, framed.range, framed.reported);
        if !is_valid_trim_range(&curve, first, last) {
            return Ok((Handle::default(), reported.0, reported.1));
        }
        // The native code trims the curve before it projects it; the trim can raise.
        GeomTrimmedCurve::new(&curve, first, last, true, false)?;
        Err(projection_not_ported(first, last))
    }
}

/// Standard_NotImplemented for the projection step of `CurveOnPlane`.
fn projection_not_ported(first: f64, last: f64) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        &format!(
            "BRep_Tool::CurveOnPlane: projecting the trimmed 3D curve [{first}, {last}] on the plane \
             requires GeomProjLib::ProjectOnPlane and ProjLib_ProjectedCurve (ProjLib and GeomProjLib \
             are not ported); expected a p-curve stored in the edge"
        ),
        "",
    )
}
