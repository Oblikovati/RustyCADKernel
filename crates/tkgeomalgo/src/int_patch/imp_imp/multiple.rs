// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of MultiplePoint and PointOnSecondDom of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of a restriction that are multiple points of the lines, or points of the
//! domain of the other surface.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::line::IntPatchLine;
use super::super::path_point::IntPatchThePathPointOfTheSOnBounds;
use super::super::point::IntPatchPoint;
use super::find_line::single_line;
use super::lines::VertexLine;
use super::transitions::line_arc_transitions;
use crate::int_start::{TheArc, TheTopolTool, TheVertex};
use crate::int_surf::{IntSurfQuadric, IntSurfTransition};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The `Done` flags of the points and the `UsedLine` flags of the lines.
pub(super) struct Marks {
    pub(super) done: Vec<i32>,
    pub(super) used_line: Vec<i32>,
}

/// The inputs shared by the two searches: the points, the domain, the quadric and its
/// normal at the current point.
pub(super) type PointsOnDomain<'a> = (
    &'a [IntPatchThePathPointOfTheSOnBounds],
    &'a (dyn TheTopolTool + 'static),
    &'a IntSurfQuadric,
    Vector3,
);

/// The locals that the loops of `MultiplePoint` and `PointOnSecondDom` keep from one
/// vertex to the next.
#[derive(Default)]
struct VertexEdit {
    intpt: IntPatchPoint,
    transline: IntSurfTransition,
    transarc: IntSurfTransition,
    vtx: TheVertex,
    d1u: Vector3,
    d1v: Vector3,
    vtgrst: Vector3,
}

impl VertexEdit {
    /// The tangent of the arc at `prm` through the quadric derivatives, recomputed.
    fn tangent_on_quadric(
        &mut self,
        quad: &IntSurfQuadric,
        arc: &TheArc,
        prm: f64,
    ) -> Result<(), Failure> {
        let (p2d, d2d): (Point2, Vector2) = arc.d1(prm)?;
        if let Some((_, d1u, d1v)) = quad.d1(p2d.x(), p2d.y()) {
            (self.d1u, self.d1v) = (d1u, d1v);
        }
        self.vtgrst
            .set_linear_form(d2d.x(), self.d1u, d2d.y(), self.d1v);
        Ok(())
    }

    /// The tangent of the arc at `prm` with the quadric derivatives kept from before.
    fn tangent_kept(&mut self, arc: &TheArc, prm: f64) -> Result<(), Failure> {
        let (_, d2d): (Point2, Vector2) = arc.d1(prm)?;
        self.vtgrst
            .set_linear_form(d2d.x(), self.d1u, d2d.y(), self.d1v);
        Ok(())
    }

    /// The transitions at the point, then the arc of the vertex.
    fn set_arc(
        &mut self,
        on_first: bool,
        (arc, prm): (&TheArc, f64),
        vtgint: Vector3,
        normale: Vector3,
    ) -> Result<(), Failure> {
        line_arc_transitions(
            vtgint,
            self.vtgrst,
            normale,
            &mut self.transline,
            &mut self.transarc,
        )?;
        self.intpt
            .set_arc(on_first, arc, prm, &self.transline, &self.transarc);
        Ok(())
    }
}

/// `MultiplePoint(listpnt, Domain, QuadSurf, Normale, slin, Done, UsedLine, Index,
/// OnFirst, theToler)`: the point `index` replaces the multiple vertices (not on this
/// domain) that it is near, on every line; the later points of the same underlying
/// vertex are added to such a line, which becomes used. Returns false when the point
/// matched a multiple vertex without a later point of its vertex (or when a line
/// cannot be projected on, which "must not happen").
pub(super) fn multiple_point(
    (listpnt, domain, quad, normale): PointsOnDomain<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    marks: &mut Marks,
    index: usize,
    on_first: bool,
    toler: f64,
) -> Result<bool, Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &listpnt[index];
    let point: Point3 = current.value();
    let mut localdone: Vec<i32> = marks.done.clone();
    let mut edit = VertexEdit::default();
    let mut retvalue: bool = true;
    for (ii, line) in slin.iter().enumerate() {
        let vline = VertexLine::analytic_or_conic(line)?;
        let nbvtx: i32 = vline.nb_vertex();
        for jj in 1..=nbvtx {
            edit.intpt = vline.vertex(jj)?;
            let off_domain = if on_first {
                !edit.intpt.is_on_dom_s1()
            } else {
                !edit.intpt.is_on_dom_s2()
            };
            if !(edit.intpt.is_multiple() && off_domain) {
                continue;
            }
            if !(point.distance(&edit.intpt.value()) <= edit.intpt.tolerance()) {
                continue;
            }
            retvalue = false;
            let Some((_, vtgint)) = single_line(point, line, edit.intpt.tolerance())? else {
                return Ok(false); // must not happen
            };
            let found = MultipleFound {
                vline: &vline,
                jj,
                ii,
                vtgint,
            };
            let args = (listpnt, domain, quad, normale);
            if replace_multiple(
                args,
                &mut edit,
                &found,
                (index, on_first, toler),
                marks,
                &mut localdone,
            )? {
                retvalue = true;
            }
        }
    }
    marks.done.copy_from_slice(&localdone);
    Ok(retvalue)
}

/// The multiple vertex `jj` of the line `ii` that the point matched.
struct MultipleFound<'a> {
    vline: &'a VertexLine,
    jj: i32,
    ii: usize,
    vtgint: Vector3,
}

/// Replaces the matched multiple vertex by the point on its arc, then adds the later
/// points of the same underlying vertex; true when one was added.
fn replace_multiple(
    (listpnt, domain, quad, normale): PointsOnDomain<'_>,
    edit: &mut VertexEdit,
    found: &MultipleFound<'_>,
    (index, on_first, toler): (usize, bool, f64),
    marks: &mut Marks,
    localdone: &mut [i32],
) -> Result<bool, Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &listpnt[index];
    let goon: bool = !current.is_new();
    if goon {
        edit.vtx = current.vertex()?;
        edit.intpt.set_vertex(on_first, &edit.vtx);
    }
    let (arc, prm) = (current.arc(), current.parameter());
    edit.tangent_on_quadric(quad, &arc, prm)?;
    // A null normal (the apex of a cone) simulates an unknown transition.
    edit.intpt.set_value(current.value());
    edit.set_arc(on_first, (&arc, prm), found.vtgint, normale)?;
    edit.intpt.set_tolerance(toler);
    found.vline.replace(found.jj, &edit.intpt)?;
    localdone[index] = 1;
    if !goon {
        return Ok(false);
    }
    let mut added: bool = false;
    for k in index + 1..listpnt.len() {
        let other: &IntPatchThePathPointOfTheSOnBounds = &listpnt[k];
        if marks.done[k] == 1 || other.is_new() {
            continue;
        }
        let vtxbis: TheVertex = other.vertex()?;
        if !domain.identical(&edit.vtx, &vtxbis)? {
            continue;
        }
        edit.intpt.set_vertex(on_first, &vtxbis);
        let (arc, prm) = (other.arc(), other.parameter());
        edit.tangent_kept(&arc, prm)?;
        edit.set_arc(on_first, (&arc, prm), found.vtgint, normale)?;
        edit.intpt.set_tolerance(toler);
        found.vline.add_vertex(&edit.intpt)?;
        marks.used_line[found.ii] = 1;
        added = true;
        localdone[k] = 1;
    }
    Ok(added)
}

/// `PointOnSecondDom(listpnt, Domain, QuadSurf, Normale, Vtgint, lin, Done, Index,
/// theToler)`: the point `index` (on the domain of the second surface) is put on the
/// vertices of the line that are not on that domain and that it is near; the later
/// points of its underlying vertex are added. The vertex count is read again after
/// each vertex, so the added vertices are visited too, and the current point becomes
/// the last point not done that the search of the underlying vertex read, as natively
/// (the same variable). Returns false when a vertex matched.
pub(super) fn point_on_second_dom(
    (listpnt, domain, quad, normale, vtgint): (
        &[IntPatchThePathPointOfTheSOnBounds],
        &(dyn TheTopolTool + 'static),
        &IntSurfQuadric,
        Vector3,
        Vector3,
    ),
    vline: &VertexLine,
    done: &mut [i32],
    index: usize,
    toler: f64,
) -> Result<bool, Failure> {
    let mut edit = VertexEdit::default();
    let mut retvalue: bool = true;
    let mut jj: i32 = 1;
    let mut nbvtx: i32 = vline.nb_vertex();
    let mut cur: usize = index;
    while jj <= nbvtx {
        edit.intpt = vline.vertex(jj)?;
        let current: &IntPatchThePathPointOfTheSOnBounds = &listpnt[cur];
        if !edit.intpt.is_on_dom_s2()
            && current.value().distance(&edit.intpt.value()) <= edit.intpt.tolerance()
        {
            retvalue = false;
            let args = (listpnt, domain, quad, normale, vtgint);
            cur = second_dom_vertex(args, &mut edit, vline, (jj, index, cur, toler), done)?;
        }
        jj += 1;
        nbvtx = vline.nb_vertex();
    }
    Ok(retvalue)
}

/// The body of `PointOnSecondDom` for the matched vertex `jj`.
fn second_dom_vertex(
    (listpnt, domain, quad, normale, vtgint): (
        &[IntPatchThePathPointOfTheSOnBounds],
        &(dyn TheTopolTool + 'static),
        &IntSurfQuadric,
        Vector3,
        Vector3,
    ),
    edit: &mut VertexEdit,
    vline: &VertexLine,
    (jj, index, mut cur, toler): (i32, usize, usize, f64),
    done: &mut [i32],
) -> Result<usize, Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &listpnt[cur];
    let goon: bool = !current.is_new();
    if goon {
        edit.vtx = current.vertex()?;
        edit.intpt.set_vertex(false, &edit.vtx);
    }
    let (arc, prm) = (current.arc(), current.parameter());
    edit.tangent_on_quadric(quad, &arc, prm)?;
    edit.set_arc(false, (&arc, prm), vtgint, normale)?;
    edit.intpt.set_tolerance(toler);
    vline.replace(jj, &edit.intpt)?;
    done[index] = 1;
    if !goon {
        return Ok(cur);
    }
    for k in index + 1..listpnt.len() {
        let other: &IntPatchThePathPointOfTheSOnBounds = &listpnt[k];
        if done[k] == 1 {
            continue;
        }
        cur = k;
        if other.is_new() {
            continue;
        }
        let vtxbis: TheVertex = other.vertex()?;
        if !domain.identical(&edit.vtx, &vtxbis)? {
            continue;
        }
        edit.intpt.set_vertex(false, &vtxbis);
        let (arc, prm) = (other.arc(), other.parameter());
        edit.tangent_kept(&arc, prm)?;
        edit.set_arc(false, (&arc, prm), vtgint, normale)?;
        edit.intpt.set_tolerance(toler);
        vline.add_vertex(&edit.intpt)?;
        done[k] = 1;
    }
    Ok(cur)
}
