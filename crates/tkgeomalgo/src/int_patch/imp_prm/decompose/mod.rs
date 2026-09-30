// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of DecomposeResult and of the post processing of
// IntPatch_ImpPrmIntersection::Perform of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The lines on a cone, sphere, cylinder or torus split where they cross the seam, a
//! pole or apex, or a bound of the quadric (docs/geomalgo-design.md section 17).
mod emit;
mod scan;
mod seam;
mod smooth;
mod vertices;

use super::super::IntPatchIType;
use super::super::line::IntPatchLine;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::super::IntPatchTheSurfFunction;
use super::IntPatchImpPrmIntersection;
use super::cleanup::point_line;
use super::perform::PrmInput;
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfLineOn2S, IntSurfQuadric};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

/// The fixed inputs of `DecomposeResult`.
pub(super) struct DecomposeCtx<'a> {
    is_reversed: bool,
    quad: &'a IntSurfQuadric,
    pdomain: &'a (dyn TheTopolTool + 'static),
    q_surf: &'a Adaptor3dSurfaceHandle,
    p_surf: &'a Adaptor3dSurfaceHandle,
    arc_tol: f64,
    tol_tang: f64,
}

/// `aTOL3D` of `DecomposeResult`.
const TOL_3D: f64 = 1.0e-10;

impl IntPatchImpPrmIntersection {
    /// The lines on a cone, sphere, cylinder or torus are decomposed (`DecomposeResult`
    /// on each); when one is, the decomposed lines replace the result.
    pub(super) fn decompose(&mut self, input: &PrmInput<'_>, func: &mut IntPatchTheSurfFunction) -> Result<(), Failure> {
        let t: SurfaceType = input.quad.type_quadric();
        if !matches!(t, SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Cylinder | SurfaceType::Torus) {
            return Ok(());
        }
        let tol_3d: f64 = std_max(func.tolerance(), input.tol_tang);
        let ctx = DecomposeCtx {
            is_reversed: input.reversed,
            quad: &input.quad,
            pdomain: if input.reversed { input.d1 } else { input.d2 },
            q_surf: input.qsurf(),
            p_surf: input.psurf().0,
            arc_tol: input.tol_arc,
            tol_tang: tol_3d,
        };
        let mut dslin: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        let mut is_decompose: bool = false;
        for l in self.slin.clone() {
            if decompose_result(&point_line(&l)?, &ctx, &mut dslin)? {
                is_decompose = true;
            }
        }
        if is_decompose {
            self.slin = dslin;
        }
        Ok(())
    }
}

/// Whether the line is a restriction line whose arc is not a 2d line (not supported).
fn is_unsupported_restriction(line: &Handle<dyn IntPatchPointLine>) -> Result<bool, Failure> {
    if line.arc_type() != IntPatchIType::Restriction {
        return Ok(false);
    }
    let rl: Handle<IntPatchRLine> = line.cast();
    let Some(r) = rl.get() else {
        return Ok(false);
    };
    let arc = if r.is_arc_on_s1() { r.arc_on_s1() } else { r.arc_on_s2() };
    Ok(arc.get_type()? != CurveType::Line)
}

/// The point set of the line; a restriction line without one (dereferenced natively)
/// is a Standard_NullObject.
fn line_points(line: &Handle<dyn IntPatchPointLine>) -> Result<Handle<IntSurfLineOn2S>, Failure> {
    let c: Handle<IntSurfLineOn2S> = line.curve();
    if c.is_null() {
        return Err(super::super::failures::null_object("IntPatch_ImpPrmIntersection::Perform - DecomposeResult", "IntSurf_LineOn2S"));
    }
    Ok(c)
}

/// `DecomposeResult(theLine, IsReversed, theQuad, thePDomain, theQSurf, thePSurf,
/// theArcTol, theTolTang, theLines)`: the line split where it passes the seam, a pole
/// or apex, or a bound of the quadric; its parts go into `lines`. Its point set is
/// adjusted in place (`[0, 2 PI]` quadric U, seam points inserted into a walking line).
/// Returns whether the line was decomposed.
fn decompose_result(line: &Handle<dyn IntPatchPointLine>, ctx: &DecomposeCtx<'_>, lines: &mut Vec<Handle<dyn IntPatchLine>>) -> Result<bool, Failure> {
    if is_unsupported_restriction(line)? {
        // The restriction line must be an isoline: other cases are not supported.
        return Ok(false);
    }
    let tol_2d: f64 = precision::p_confusion();
    let ssline: Handle<IntSurfLineOn2S> = line_points(line)?;
    if ssline.nb_points() <= 2 {
        return Ok(false);
    }
    // The repeated vertices are deleted.
    let vline: Handle<IntSurfLineOn2S> = vertices::get_vertices(&**line, TOL_3D, tol_2d)?;
    seam::adjust_line(&ssline, ctx.is_reversed, ctx.q_surf, tol_2d)?;
    if line.arc_type() == IntPatchIType::Walking {
        loop {
            let ptypes: Vec<i32> = vertices::search_vertices(&ssline, &vline)?;
            if !seam::insert_seam_vertices(&ssline, ctx.is_reversed, &vline, &ptypes, tol_2d)? {
                break;
            }
        }
    }
    let mut state = scan::DecomposeState::new(ssline.nb_points());
    let parts = emit::PartSink { line, ctx, vline: &vline, lines };
    state.run(&ssline, ctx, parts)
}
