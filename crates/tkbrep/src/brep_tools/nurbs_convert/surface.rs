// Created on: 1996-07-12
// Created by: Stagiaire Mary FABIEN
// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of BRepTools_NurbsConvertModification::NewSurface
// (ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_NurbsConvertModification::NewSurface`: the B-spline surface of a face.
use super::NurbsConvertModification;
use super::helpers::change_bounds;
use crate::brep::BRepTool;
use crate::brep_tools::modification::NewSurface;
use crate::brep_tools::uv_bounds;
use crate::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkgeombase::geom_convert::surface_to_bspline_surface;

/// The face bounds adjusted to the surface: snapped to the surface bounds within
/// `0.1 * tolerance`, clamped to them in a non-periodic direction, and at most one
/// period wide in a periodic one. `(U1, U2, V1, V2)`.
fn adjusted_bounds(
    surface: &dyn GeomSurface,
    face_bounds: (f64, f64, f64, f64),
    tolerance: f64,
) -> Result<(f64, f64, f64, f64), Failure> {
    let (cu1, cu2, cv1, cv2) = face_bounds;
    let (su1, su2, sv1, sv2) = surface.bounds();
    let tol_par = 0.1 * tolerance;
    let snap = |value: f64, bound: f64| {
        if (value - bound).abs() <= tol_par {
            bound
        } else {
            value
        }
    };
    let (mut u1, mut u2, mut v1, mut v2) = (
        snap(cu1, su1),
        snap(cu2, su2),
        snap(cv1, sv1),
        snap(cv2, sv2),
    );
    if !surface.is_u_periodic() {
        (u1, u2) = (su1.max(cu1), su2.min(cu2));
    }
    if !surface.is_v_periodic() {
        (v1, v2) = (sv1.max(cv1), sv2.min(cv2));
    }
    if surface.is_u_periodic() && u2 - u1 > surface.u_period()? {
        u2 = u1 + surface.u_period()?;
    }
    if surface.is_v_periodic() && v2 - v1 > surface.v_period()? {
        v2 = v1 + surface.v_period()?;
    }
    Ok((u1, u2, v1, v2))
}

impl NurbsConvertModification {
    /// `NewSurface(F, S, L, Tol, RevWires, RevFace)`: for a face whose surface is not a
    /// B-spline or Bezier surface, the surface trimmed to the adjusted face bounds (when
    /// they differ from its own by more than the tolerance) converted into a B-spline
    /// surface, whose non-periodic knots are brought back onto the bounds so that the old
    /// p-curves stay valid; the face location and tolerance are kept. False for a face
    /// without surface and for a B-spline or Bezier surface.
    pub(super) fn converted_surface(
        &self,
        face: &Face,
        result: &mut NewSurface,
    ) -> Result<bool, Failure> {
        result.rev_wires = false;
        result.rev_face = false;
        let (surface, location) = BRepTool::surface_with_location(face)?;
        result.location = location;
        let Some(source) = surface.get() else {
            return Ok(false);
        };
        let exact = source.dynamic_type().name();
        if exact == "Geom_BSplineSurface" || exact == "Geom_BezierSurface" {
            return Ok(false);
        }
        result.surface = surface.clone();
        let face_bounds = uv_bounds(face)?;
        result.tolerance = BRepTool::face_tolerance(face)?;
        let (u1, u2, v1, v2) = adjusted_bounds(source, face_bounds, result.tolerance)?;
        let (su1, su2, sv1, sv2) = source.bounds();
        let tolerance = result.tolerance;
        if (su1 - u1).abs() > tolerance
            || (su2 - u2).abs() > tolerance
            || (sv1 - v1).abs() > tolerance
            || (sv2 - v2).abs() > tolerance
        {
            let trimmed = GeomRectangularTrimmedSurface::new(
                &result.surface,
                (u1, u2),
                (v1, v2),
                (true, true),
            )?;
            result.surface = Handle::new(trimmed).cast();
        }
        let Some(to_convert) = result.surface.get() else {
            return Ok(false);
        };
        let (su1, _su2, sv1, _sv2) = to_convert.bounds();
        let bspline = surface_to_bspline_surface(&result.surface)?;
        result.surface = bspline.clone().cast();
        self.fit_knots_to_face(
            &bspline,
            face_bounds,
            (su1, sv1),
            ((u1, u2), (v1, v2)),
            tolerance,
        )?;
        if !self.is_recorded(&surface.cast()) {
            self.record(surface.cast(), result.surface.cast());
        }
        Ok(true)
    }

    /// The knots of each non-periodic direction reparametrised onto the adjusted bounds
    /// when the face starts farther than the B-spline resolution from the surface start.
    fn fit_knots_to_face(
        &self,
        bspline: &Handle<rustycad_tkg3d::geom::GeomBSplineSurface>,
        (cu1, _cu2, cv1, _cv2): (f64, f64, f64, f64),
        (su1, sv1): (f64, f64),
        (u_range, v_range): ((f64, f64), (f64, f64)),
        tolerance: f64,
    ) -> Result<(), Failure> {
        let Some(spline) = bspline.get() else {
            return Ok(());
        };
        let (u_tol, v_tol) = spline.resolution(tolerance)?;
        if (cu1 - su1).abs() > u_tol && !spline.is_u_periodic() {
            change_bounds(spline, true, u_range)?;
        }
        if (cv1 - sv1).abs() > v_tol && !spline.is_v_periodic() {
            change_bounds(spline, false, v_range)?;
        }
        Ok(())
    }
}
