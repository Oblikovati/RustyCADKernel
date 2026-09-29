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
//! The trimmed 3D curve is projected along the plane normal by
//! `GeomProjLib::ProjectOnPlane`, then onto the plane parameters by
//! `ProjLib_ProjectedCurve`. GeomProjLib is being ported beside TKBRep: every step but
//! that one call is ported, and the call raises Standard_NotImplemented naming the
//! missing package (docs/tkbrep-progress.md, "Dependency gap").
use super::BRepTool;
use crate::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::gp::Direction3;
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
    /// projecting the trimmed 3D curve on the plane (the basis of a trimmed result). A
    /// non-planar surface, an edge without a 3D curve and an invalid range return a null
    /// curve, with the range `0, 0` in the first two cases and the edge range in the last.
    ///
    /// The projection along the normal needs `GeomProjLib::ProjectOnPlane`, which is not
    /// ported yet: that step raises Standard_NotImplemented.
    /// `let (pcurve, first, last) = BRepTool::curve_on_plane(&edge, &surface, &location)?;`.
    pub fn curve_on_plane(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
        let plane = plane_of(surface);
        let Some(plane_ref) = plane.get() else {
            return Ok((Handle::default(), 0.0, 0.0));
        };
        let Some(framed) = curve_in_plane_frame(edge, location)? else {
            return Ok((Handle::default(), 0.0, 0.0));
        };
        let (curve, (first, last), reported) = (framed.curve, framed.range, framed.reported);
        if !is_valid_trim_range(&curve, first, last) {
            return Ok((Handle::default(), reported.0, reported.1));
        }
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, false)?).cast();
        let normal = plane_ref.position().direction();
        let projected = project_on_plane(&trimmed, &plane, normal, first, last)?;
        let pcurve = pcurve_on_plane(&plane, &projected)?;
        Ok((pcurve, reported.0, reported.1))
    }
}

/// `GeomProjLib::ProjectOnPlane(Curve, Plane, Dir, true)`: GeomProjLib is being ported
/// beside TKBRep (it needs ProjLib), so this one call raises Standard_NotImplemented.
fn project_on_plane(
    _curve: &Handle<dyn GeomCurve>,
    _plane: &Handle<GeomPlane>,
    _direction: Direction3,
    first: f64,
    last: f64,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    Err(Failure::with_type(
        "Standard_NotImplemented",
        &format!(
            "BRep_Tool::CurveOnPlane: projecting the trimmed 3D curve [{first}, {last}] along the \
             plane normal requires GeomProjLib::ProjectOnPlane (GeomProjLib is not ported); \
             expected a p-curve stored in the edge"
        ),
        "",
    ))
}

/// The p-curve of a curve lying in the plane: `ProjLib_ProjectedCurve` on the plane
/// adaptor made into a Geom2d curve (`Geom2dAdaptor::MakeCurve`), the basis of a trimmed
/// result.
fn pcurve_on_plane(
    plane: &Handle<GeomPlane>,
    projected: &Handle<dyn GeomCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let surface: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane.clone().cast())?).cast();
    let curve: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_curve(projected)?).cast();
    let projection = ProjectedCurve::with_curve(&surface, &curve)?;
    let pcurve = geom2d_adaptor::make_curve(&projection)?;
    if let Some(trimmed) = pcurve.cast::<Geom2dTrimmedCurve>().get() {
        return Ok(trimmed.basis_curve());
    }
    Ok(pcurve)
}
