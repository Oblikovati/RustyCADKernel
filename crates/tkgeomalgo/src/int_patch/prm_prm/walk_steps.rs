// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the steps repeated by the Perform overloads of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps that every `Perform` overload repeats around a walking: the test of the
//! start point against the lines found, the transitions of a new line, the placing of
//! the vertices of the restrictions, the end vertices, and the rejection of a line that
//! duplicates a line found (docs/geomalgo-design.md section 18).
use super::IntPatchPrmPrmIntersection;
use super::helpers::{add_wline, dublicate_of_lines_processing, is_point_on_line};
use super::several_wlines::{SeveralWlinesInputs, several_wlines_processing};
use crate::int_patch::{
    IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchRstInt, IntPatchWLType, IntPatchWLine,
};
use crate::int_start::TheTopolTool;
use crate::int_surf::{IntSurfPntOn2S, IntSurfTypeTrans};
use crate::int_walk::IntWalkPWalking;
use crate::intf::std_max;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The surfaces and domains of a `Perform` (the same surface twice for the
/// self-intersection) and its tolerances.
pub(super) struct WalkInputs<'a> {
    pub(super) surf1: &'a Adaptor3dSurfaceHandle,
    pub(super) d1: &'a (dyn TheTopolTool + 'static),
    pub(super) surf2: &'a Adaptor3dSurfaceHandle,
    /// None for the self-intersection, which places the vertices on the first surface
    /// only.
    pub(super) d2: Option<&'a (dyn TheTopolTool + 'static)>,
    pub(super) tol_tangency: f64,
    pub(super) epsilon: f64,
    pub(super) deflection: f64,
    pub(super) increment: f64,
}

impl WalkInputs<'_> {
    /// `SeuildPointLigne`: `15 * Increment^2`.
    pub(super) fn seuil(&self) -> f64 {
        15.0 * self.increment * self.increment
    }
}

/// Whether the start point is far from every line found (`dminiPointLigne >
/// SeuildPointLigne` after the loop over the lines, with the native threshold test).
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
pub(super) fn is_new_start(
    inputs: &WalkInputs<'_>,
    start: &IntSurfPntOn2S,
    lines: &[Handle<dyn IntPatchLine>],
) -> Result<bool, Failure> {
    let seuil: f64 = inputs.seuil();
    let mut dmini: f64 = seuil + seuil;
    for line in lines {
        if !(dmini >= seuil) {
            break;
        }
        if is_point_on_line(start, &line.cast::<IntPatchWLine>(), inputs.deflection)? {
            dmini = 0.0;
        }
    }
    Ok(dmini > seuil)
}

/// The transitions of a new line from the tangent of the walking at its boundary
/// point: OUT on the first surface and IN on the second when the tangent turns from
/// the normal of the second to the one of the first (strictly, unless `loose`).
pub(super) fn transitions(
    inputs: &WalkInputs<'_>,
    pw: &IntWalkPWalking,
    loose: bool,
) -> Result<(IntSurfTypeTrans, IntSurfTypeTrans), Failure> {
    let (tangent, indextg) = pw.tangent_at_line()?;
    let tgline: Vector3 = Vector3::from(tangent);
    let point: IntSurfPntOn2S = pw.line()?.value(indextg)?;
    let (locu, locv) = point.parameters_on_s1();
    let d1 = inputs.surf1.d1(locu, locv)?;
    let norm1: Vector3 = d1.d1u.crossed(d1.d1v);
    let (locu, locv) = point.parameters_on_s2();
    let d2 = inputs.surf2.d1(locu, locv)?;
    let norm2: Vector3 = d2.d1u.crossed(d2.d1v);
    let dot_cross: f64 = tgline.dot_cross(norm2, norm1);
    let out_in: bool = if loose {
        dot_cross >= 0.0
    } else {
        dot_cross > 0.0
    };
    Ok(match out_in {
        true => (IntSurfTypeTrans::Out, IntSurfTypeTrans::In),
        false => (IntSurfTypeTrans::In, IntSurfTypeTrans::Out),
    })
}

/// The walking line on the points of the walking (shared with it, as natively).
pub(super) fn new_wline(
    pw: &IntWalkPWalking,
    (trans1, trans2): (IntSurfTypeTrans, IntSurfTypeTrans),
) -> Result<Handle<IntPatchWLine>, Failure> {
    let wline = IntPatchWLine::new(pw.line()?, false, trans1, trans2);
    wline.set_creating_way_info(IntPatchWLType::PrmPrm);
    Ok(Handle::new(wline))
}

/// `IntPatch_RstInt::PutVertexOnLine` on the restrictions of the first domain, then of
/// the second one (it can reduce the number of points of the line); false when fewer
/// than 2 points are left.
pub(super) fn put_vertices(
    inputs: &WalkInputs<'_>,
    wline: &Handle<IntPatchWLine>,
    tol_tang: f64,
) -> Result<bool, Failure> {
    let line: Handle<dyn IntPatchLine> = wline.cast();
    IntPatchRstInt::put_vertex_on_line(
        &line,
        inputs.surf1,
        inputs.d1,
        inputs.surf2,
        true,
        tol_tang,
    )?;
    if wline.nb_pnts()? < 2 {
        return Ok(false);
    }
    let Some(d2) = inputs.d2 else {
        return Ok(true);
    };
    IntPatchRstInt::put_vertex_on_line(&line, inputs.surf2, d2, inputs.surf1, false, tol_tang)?;
    Ok(wline.nb_pnts()? >= 2)
}

/// The vertex of the line at `para` on the point `rank` of the walking.
fn end_vertex(
    pw: &IntWalkPWalking,
    rank: i32,
    p: Point3,
    (tol_tang, para): (f64, f64),
) -> Result<IntPatchPoint, Failure> {
    let (pu1, pv1, pu2, pv2) = pw.line()?.value(rank)?.parameters();
    let mut vtx = IntPatchPoint::new();
    vtx.set_value_tangent(p, tol_tang, false);
    vtx.set_parameters(pu1, pv1, pu2, pv2);
    vtx.set_parameter(para);
    Ok(vtx)
}

/// A line without vertices gets its first and last points as vertices (the 3d points
/// are those of the walking before the vertices were placed).
pub(super) fn add_end_vertices(
    pw: &IntWalkPWalking,
    wline: &Handle<IntPatchWLine>,
    (debut, fin): (Point3, Point3),
    tol_tang: f64,
) -> Result<(), Failure> {
    if wline.nb_vertex() != 0 {
        return Ok(());
    }
    wline.add_vertex(&end_vertex(pw, 1, debut, (tol_tang, 1.0))?, false);
    let last: i32 = wline.nb_pnts()?;
    wline.add_vertex(
        &end_vertex(pw, last, fin, (tol_tang, f64::from(last)))?,
        false,
    );
    Ok(())
}

/// How the end points of a new line are compared with those of the lines found.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EndsCheck {
    /// Both ends within the tangency tolerance (`<=`, `&&`).
    Both,
    /// One end closer than the tolerance (`<`, `||`).
    Either,
}

/// The rank of the first line found that the new line duplicates: its last point is on
/// the line (when `check_on_line`), or its ends match the ends of the line.
pub(super) fn duplicated_line(
    inputs: &WalkInputs<'_>,
    pw: &IntWalkPWalking,
    lines: &[Handle<dyn IntPatchLine>],
    (nb_lig_calculee, check_on_line, ends): (i32, bool, EndsCheck),
) -> Result<Option<i32>, Failure> {
    let point_fin: IntSurfPntOn2S = pw.value(pw.nb_points()?)?;
    let (debut, fin) = (pw.value(1)?.value(), point_fin.value());
    let tol: f64 = inputs.tol_tangency;
    for ver in 1..=nb_lig_calculee {
        let verwline: Handle<IntPatchWLine> = lines[(ver - 1) as usize].cast::<IntPatchWLine>();
        // Check end point if it is on existing line. Start point is checked before.
        if check_on_line && is_point_on_line(&point_fin, &verwline, inputs.deflection)? {
            return Ok(Some(ver));
        }
        let ver_debut: Point3 = verwline.point(1)?.value();
        let ver_fin: Point3 = verwline.point(verwline.nb_pnts()?)?.value();
        let (d_debut, d_fin) = (debut.distance(&ver_debut), fin.distance(&ver_fin));
        let same: bool = match ends {
            EndsCheck::Both => d_debut <= tol && d_fin <= tol,
            EndsCheck::Either => d_debut < tol || d_fin < tol,
        };
        if same {
            return Ok(Some(ver));
        }
    }
    Ok(None)
}

/// A rejected line is kept after all when it is better than the line it duplicates
/// (which is then removed); returns whether it is still rejected.
pub(super) fn keep_rejected(
    pw: &IntWalkPWalking,
    duplicated: Option<i32>,
    lines: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<bool, Failure> {
    let Some(ver) = duplicated else {
        return Ok(false);
    };
    let mut reject = true;
    dublicate_of_lines_processing(pw, ver, lines, &mut reject)?;
    Ok(reject)
}

/// The variant of the steps that follow an accepted walking.
pub(super) struct AcceptOptions {
    /// The transitions test is `>=` (else `>`).
    pub(super) loose: bool,
    /// `EnablePurging(!hasBeenAdded)` when the points of the walking were densified.
    pub(super) purging: Option<bool>,
    /// The periods for `SeveralWlinesProcessing`, when it runs.
    pub(super) several: Option<[f64; 4]>,
}

impl IntPatchPrmPrmIntersection {
    /// The line of an accepted walking: its transitions, its vertices (false is
    /// returned when the line vanishes), its end vertices, the joining to the vertices
    /// of the lines found, and `AddWLine`.
    pub(super) fn accept_walking(
        &mut self,
        inputs: &WalkInputs<'_>,
        pw: &IntWalkPWalking,
        options: &AcceptOptions,
        ends: (Point3, Point3),
    ) -> Result<bool, Failure> {
        let trans = transitions(inputs, pw, options.loose)?;
        let tol_tang: f64 = inputs.tol_tangency;
        let mut wline: Handle<IntPatchWLine> = new_wline(pw, trans)?;
        if let Some(enabled) = options.purging {
            wline.enable_purging(enabled);
        }
        // The method PutVertexOnLine can reduce the number of points in <wline>.
        if !put_vertices(inputs, &wline, tol_tang)? {
            return Ok(false);
        }
        add_end_vertices(pw, &wline, ends, tol_tang)?;
        if let Some(periods) = options.several {
            let max_steps = (
                std_max(pw.max_step(0)?, pw.max_step(1)?),
                std_max(pw.max_step(2)?, pw.max_step(3)?),
            );
            let several = SeveralWlinesInputs {
                surf1: inputs.surf1,
                surf2: inputs.surf2,
                periods,
                trans,
                tol: tol_tang,
                max_steps,
            };
            several_wlines_processing(&several, &self.slin, &mut wline)?;
        }
        add_wline(&mut self.slin, &wline, inputs.deflection)?;
        self.empt = false;
        Ok(true)
    }
}
