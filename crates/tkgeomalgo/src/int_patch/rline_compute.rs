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
// Rust translation of IntPatch_RLine::ComputeVertexParameters (IntPatch_RLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_RLine::ComputeVertexParameters`: the restriction filters, the sorts and the
//! first and last points.
use super::point::IntPatchPoint;
use super::vertices::{CountedVertices, VertexList, repeat_until_stable};
use rustycad_tkernel::standard::Failure;

const MEMBER: &str = "IntPatch_RLine::ComputeVertexParameters";

/// The domain data of a vertex on one surface: on the domain, the arc and the parameter
/// on it (the arc and parameter are read only when the vertex is on the domain).
fn on(v: &IntPatchPoint, on_first: bool) -> bool {
    if on_first {
        v.is_on_dom_s1()
    } else {
        v.is_on_dom_s2()
    }
}

/// Same arc of the domain (both vertices on it).
fn same_arc(a: &IntPatchPoint, b: &IntPatchPoint, on_first: bool) -> Result<bool, Failure> {
    if on_first {
        return Ok(a.arc_on_s1()? == b.arc_on_s1()?);
    }
    Ok(a.arc_on_s2()? == b.arc_on_s2()?)
}

/// Same parameter on the arc of the domain (both vertices on it).
fn same_prm(a: &IntPatchPoint, b: &IntPatchPoint, on_first: bool) -> Result<bool, Failure> {
    if on_first {
        return Ok(a.parameter_on_arc1()? == b.parameter_on_arc1()?);
    }
    Ok(a.parameter_on_arc2()? == b.parameter_on_arc2()?)
}

/// The body on the vertices of the line (the tolerance is not used natively).
pub(super) fn compute_vertex_parameters(vertices: &mut VertexList) -> Result<(), Failure> {
    let nbvtx: i32 = vertices.nb_vertex();
    let mut scan = CountedVertices { vertices, nbvtx };
    filter_same_point(&mut scan, true)?;
    filter_same_point(&mut scan, false)?;
    scan.nbvtx = scan.vertices.nb_vertex();
    scan.vertices.bubble_sort(scan.nbvtx, false, MEMBER)?;
    kill_duplicates(&mut scan)?;
    scan.vertices.remove_free_twins(&mut scan.nbvtx, MEMBER)?;
    // The sort again (a "strange case").
    let nbvtx: i32 = scan.vertices.nb_vertex();
    scan.vertices.bubble_sort(nbvtx, false, MEMBER)?;
    scan.vertices.set_first_point(1);
    scan.vertices.set_last_point(nbvtx);
    Ok(())
}

/// Two vertices only on the same restriction of one domain, at the same parameters on
/// the line and on the arc: the second is removed.
fn filter_same_point(scan: &mut CountedVertices<'_>, on_first: bool) -> Result<(), Failure> {
    let only = move |v: &IntPatchPoint| on(v, on_first) && !on(v, !on_first);
    repeat_until_stable(
        scan,
        &mut |scan, i| Ok(only(scan.vertices.value(i, MEMBER)?)),
        &mut |scan, i, j| {
            let (vi, vj) = (
                scan.vertices.value(i, MEMBER)?,
                scan.vertices.value(j, MEMBER)?,
            );
            let same: bool = only(vj)
                && vi.parameter_on_line() == vj.parameter_on_line()
                && same_arc(vi, vj, on_first)?
                && same_prm(vi, vj, on_first)?;
            if same {
                scan.vertices.remove(j, MEMBER)?;
                scan.nbvtx -= 1;
                scan.vertices.shift_after_removal(j);
            }
            Ok(same)
        },
    )
}

/// The verdict of one domain on two vertices of the same parameter on the line:
/// `Some((kill, killm1))`, or `None` when they are on different arcs of the domain
/// (`restrdiff`).
fn verdict_on(
    vtx: &IntPatchPoint,
    vtxm1: &IntPatchPoint,
    on_first: bool,
) -> Result<Option<(bool, bool)>, Failure> {
    if !(on(vtxm1, on_first) && on(vtx, on_first)) {
        return Ok(Some((false, false)));
    }
    if !same_arc(vtxm1, vtx, on_first)? {
        return Ok(None);
    }
    if !same_prm(vtx, vtxm1, on_first)? {
        return Ok(Some((false, false)));
    }
    let other: bool = !on_first;
    if !on(vtxm1, other) {
        return Ok(Some((false, on(vtx, other))));
    }
    if !on(vtx, other) {
        return Ok(Some((true, false)));
    }
    let kill: bool = same_arc(vtxm1, vtx, other)? && same_prm(vtx, vtxm1, other)?;
    Ok(Some((kill, false)))
}

/// The duplicates of the same parameter on the line on the same arcs: `kill` removes
/// the vertex `i`, `killm1` the vertex `j`, unless the vertices are on different arcs.
fn kill_duplicates(scan: &mut CountedVertices<'_>) -> Result<(), Failure> {
    repeat_until_stable(scan, &mut |_, _| Ok(true), &mut |scan, i, j| {
        let (vtx, vtxm1) = (
            scan.vertices.value(i, MEMBER)?,
            scan.vertices.value(j, MEMBER)?,
        );
        if vtxm1.parameter_on_line() != vtx.parameter_on_line() {
            return Ok(false);
        }
        let Some(mut verdict) = verdict_on(vtx, vtxm1, true)? else {
            return Ok(false);
        };
        if !(verdict.0 || verdict.1) {
            match verdict_on(vtx, vtxm1, false)? {
                None => return Ok(false),
                Some(second) => verdict = second,
            }
        }
        let index: i32 = match verdict {
            (true, _) => i,
            (false, true) => j,
            (false, false) => return Ok(false),
        };
        scan.vertices.remove(index, MEMBER)?;
        scan.nbvtx -= 1;
        Ok(true)
    })
}
