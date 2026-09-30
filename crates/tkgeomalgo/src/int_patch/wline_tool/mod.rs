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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_WLineTool`: the static tools of the walking lines (docs/geomalgo-design.md
//! section 18): the purge of a line (equal points, points out of the domains, points in
//! the tube of their neighbours), the joining of lines that share an end point and the
//! extension of two lines to a common point. The file-static helpers of the native
//! source are private functions of the submodules, one per step.
mod join;
mod purge;
mod tube;

use super::{IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchWLine};
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use crate::intf::{native_int_cast, std_min};
use join::{JoinInputs, SurfaceBounds};
use purge::PurgeInputs;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp;

/// `IntPatch_WLineTool`: a set of static methods related to the walking lines.
///
/// ```ignore
/// let purged = IntPatchWLineTool::compute_purged_wline(&wline, &s1, &s2, &*dom1, &*dom2)?;
/// ```
pub struct IntPatchWLineTool;

impl IntPatchWLineTool {
    /// `myMaxConcatAngle`: the maximal angle between two walking lines that are
    /// concatenated, to avoid a result with a C0 continuity (a pure empirical value).
    pub const MY_MAX_CONCAT_ANGLE: f64 = std::f64::consts::PI / 6.0;

    /// `ComputePurgedWLine(theWLine, theS1, theS2, theDom1, theDom2)`: I. removes the
    /// equal points of the line (one of them is left) and recomputes the vertex
    /// parameters; II. removes the points out of the domains when no surface is periodic;
    /// III. removes the points that lie in the tubes of their neighbours in both
    /// parametric spaces and in 3d, each with its own tolerance. Returns the new line, or a
    /// null handle when fewer than 2 points are left.
    ///
    /// ```ignore
    /// let purged = IntPatchWLineTool::compute_purged_wline(&wline, &s1, &s2, &*dom1, &*dom2)?;
    /// ```
    pub fn compute_purged_wline(
        wline: &Handle<IntPatchWLine>,
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
        dom1: &dyn TheTopolTool,
        dom2: &dyn TheTopolTool,
    ) -> Result<Handle<IntPatchWLine>, Failure> {
        let nb: i32 = wline.nb_pnts()?;
        if nb == 2
            && wline
                .point(1)?
                .value()
                .is_equal(&wline.point(2)?.value(), gp::resolution())
        {
            return Ok(Handle::default());
        }
        let (local, nb) = delete_equal_points(wline)?;
        let nb_points: i32 = local.nb_pnts()?;
        if nb_points <= 2 {
            return Ok(if nb_points == 2 {
                local
            } else {
                Handle::default()
            });
        }
        // Avoid purge in case of C0 continuity: the intersection approximator may produce
        // an invalid curve after purge (bugs modalg_5 bug24731). Do not run purger when
        // the base number of points is too small.
        if is_c0(s1)? || is_c0(s2)? || nb < tube::NB_SINGLE_BEZIER {
            return Ok(local);
        }
        let inputs = PurgeInputs { s1, s2, dom1, dom2 };
        // II: Delete out of borders points.
        let local: Handle<IntPatchWLine> = purge::delete_outer_points(&local, &inputs)?;
        // III: Delete points by tube criteria.
        let tube: Handle<IntPatchWLine> = tube::delete_by_tube(&local, s1, s2)?;
        Ok(if tube.nb_pnts()? > 1 {
            tube
        } else {
            Handle::default()
        })
    }

    /// `JoinWLines(theSlin, theSPnt, theS1, theS2, theTol3D)`: joins the walking lines of
    /// `slin` that share an end point, when the joint crosses no seam and no bound of the
    /// surfaces and the joined line is smooth there; the points of `spnt` that match an
    /// end of a line are removed. The surfaces must be cylinders (their radii bound the
    /// curvature radius): another surface raises Standard_NoSuchObject, as natively.
    ///
    /// ```ignore
    /// IntPatchWLineTool::join_wlines(&mut lines, &mut points, &s1, &s2, 1e-7)?;
    /// ```
    pub fn join_wlines(
        slin: &mut Vec<Handle<dyn IntPatchLine>>,
        spnt: &mut Vec<IntPatchPoint>,
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
        tol3d: f64,
    ) -> Result<(), Failure> {
        if slin.is_empty() {
            return Ok(());
        }
        // For two cylindrical surfaces only.
        let min_rad: f64 = 1.0e-3 * std_min(s1.cylinder()?.radius(), s2.cylinder()?.radius());
        let inputs = JoinInputs {
            s1,
            s2,
            min_rad,
            bounds: SurfaceBounds::new(s1, s2)?,
        };
        join::join_wlines(slin, spnt, &inputs, tol3d)
    }
}

/// Whether the surface is C0 in one of its directions.
fn is_c0(s: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    Ok(s.u_continuity()? == Shape::C0 || s.v_continuity()? == Shape::C0)
}

/// Whether two points of the line are equal: the same 3d point, or the same parameters
/// on one surface up to `1e-16` times their largest magnitude.
fn are_equal_points(p1: &IntSurfPntOn2S, p2: &IntSurfPntOn2S) -> bool {
    let (a, b, c, d) = p1.parameters();
    let (e, f, g, h) = p2.parameters();
    let uv: [f64; 8] = [a, b, c, d, e, f, g, h];
    let mut max: f64 = uv[0].abs();
    for value in &uv[1..] {
        if max < value.abs() {
            max = value.abs();
        }
    }
    p1.value().is_equal(&p2.value(), gp::resolution())
        || (uv[0] - uv[4]).abs() + (uv[1] - uv[5]).abs() < 1.0e-16 * max
        || (uv[2] - uv[6]).abs() + (uv[3] - uv[7]).abs() < 1.0e-16 * max
}

/// The copy of the line with the vertex parameters shifted for the removed point `k`.
fn without_point(
    local: &IntPatchWLine,
    line: &Handle<IntSurfLineOn2S>,
    k: i32,
) -> Result<Handle<IntPatchWLine>, Failure> {
    let next: Handle<IntPatchWLine> = Handle::new(IntPatchWLine::new_undecided(line, false));
    next.set_creating_way_info(local.get_creating_way());
    for v in 1..=local.nb_vertex() {
        let mut vertex: IntPatchPoint = local.vertex(v)?;
        if native_int_cast(vertex.parameter_on_line()) >= k {
            vertex.set_parameter(vertex.parameter_on_line() - 1.0);
        }
        next.add_vertex(&vertex, false);
    }
    line.remove_point(k)?;
    Ok(next)
}

/// Step I of `ComputePurgedWLine`: a copy of the line without its equal points (each
/// point is compared with its next five points). Returns the line and the number of
/// points of the last pass (the native `nb`).
fn delete_equal_points(wline: &IntPatchWLine) -> Result<(Handle<IntPatchWLine>, i32), Failure> {
    let line: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    let mut local: Handle<IntPatchWLine> = Handle::new(IntPatchWLine::new_undecided(&line, false));
    local.set_creating_way_info(wline.get_creating_way());
    let mut nb: i32 = wline.nb_pnts()?;
    for i in 1..=nb {
        line.add(&wline.point(i)?);
    }
    for v in 1..=wline.nb_vertex() {
        local.add_vertex(&wline.vertex(v)?, false);
    }
    let mut i: i32 = 1;
    while i <= line.nb_points() {
        nb = line.nb_points();
        let start_index: i32 = i + 1;
        let mut end_index: i32 = (i + 5).min(nb);
        let mut k: i32 = start_index;
        while start_index <= nb && end_index > 1 && k <= end_index {
            if i != k && are_equal_points(&line.value(i)?, &line.value(k)?) {
                local = without_point(&local, &line, k)?;
                end_index -= 1;
                continue;
            }
            k += 1;
        }
        i += 1;
    }
    Ok((local, nb))
}
