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
// Rust translation of IntPatch_ALine::ComputeVertexParameters (IntPatch_ALine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ALine::ComputeVertexParameters`: the phases in the native order.
use super::gline_compute::{follow_removal, verdict_on};
use super::point::IntPatchPoint;
use super::vertices::{CountedVertices, VertexList, repeat_until_stable};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::Curve as IntAnaCurve;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;

const MEMBER: &str = "IntPatch_ALine::ComputeVertexParameters";

/// `#define PCONFUSION 0.00001`.
const PCONFUSION: f64 = 0.00001; // tol:parametric

/// The body on the vertices of the line of curve `curv`.
pub(super) fn compute_vertex_parameters(
    vertices: &mut VertexList,
    curv: &IntAnaCurve,
    tol: f64,
) -> Result<(), Failure> {
    let (first, last) = curv.domain()?;
    // `OpenFirst`/`OpenLast` are the `IsIncluded` flags of First/LastParameter.
    let (open_first, open_last) = (!curv.is_first_open(), !curv.is_last_open());
    add_turn_representations(vertices, curv, (first, last), tol)?;
    let nbvtx: i32 = vertices.nb_vertex();
    if nbvtx <= 0 {
        return Ok(());
    }
    vertices.bubble_sort(nbvtx, true, MEMBER)?;
    let mut scan = CountedVertices { vertices, nbvtx };
    filter_same_arc_point(&mut scan, true)?;
    filter_same_arc_point(&mut scan, false)?;
    sweep_duplicates(scan.vertices)?;
    if !open_first && !open_last {
        close_periodic(vertices, curv, (first, last), tol)?;
    }
    remove_free_ends(vertices)?;
    merge_same_parameter(vertices)
}

/// A vertex also has its representations 2 Pi before and after it on the line when the
/// curve passes there within `tol` (a very rare case: a tangency point at the start of
/// a line parametrized over 2 * 2 Pi).
fn add_turn_representations(
    vertices: &mut VertexList,
    curv: &IntAnaCurve,
    (first, last): (f64, f64),
    tol: f64,
) -> Result<(), Failure> {
    let nbvtx: i32 = vertices.nb_vertex();
    for i in 1..=nbvtx {
        let vtx: IntPatchPoint = vertices.value(i, MEMBER)?.clone();
        let p: f64 = vtx.parameter_on_line();
        let before: f64 = p - PI - PI;
        if before >= first {
            add_if_close(vertices, curv, &vtx, before, tol)?;
        }
        let after: f64 = p + PI + PI;
        if after <= last {
            add_if_close(vertices, curv, &vtx, after, tol)?;
        }
    }
    Ok(())
}

fn add_if_close(
    vertices: &mut VertexList,
    curv: &IntAnaCurve,
    vtx: &IntPatchPoint,
    param: f64,
    tol: f64,
) -> Result<(), Failure> {
    let p1: Point3 = curv.value(param)?;
    if p1.distance(&vtx.value()) < tol {
        let mut ovtx: IntPatchPoint = vtx.clone();
        ovtx.set_parameter(param);
        vertices.svtx.push(ovtx);
    }
    Ok(())
}

fn on(v: &IntPatchPoint, on_first: bool) -> bool {
    if on_first {
        v.is_on_dom_s1()
    } else {
        v.is_on_dom_s2()
    }
}

/// Two vertices only on the same arc of one domain, at the same parameters on the arc
/// and on the line (within 1e-5): the second is removed.
fn filter_same_arc_point(scan: &mut CountedVertices<'_>, on_first: bool) -> Result<(), Failure> {
    let only = move |v: &IntPatchPoint| on(v, on_first) && !on(v, !on_first);
    let prm = move |v: &IntPatchPoint| {
        if on_first {
            v.parameter_on_arc1()
        } else {
            v.parameter_on_arc2()
        }
    };
    let arc = move |v: &IntPatchPoint| {
        if on_first {
            v.arc_on_s1()
        } else {
            v.arc_on_s2()
        }
    };
    repeat_until_stable(
        scan,
        &mut |scan, i| Ok(only(scan.vertices.value(i, MEMBER)?)),
        &mut |scan, i, j| {
            let (vi, vj) = (
                scan.vertices.value(i, MEMBER)?,
                scan.vertices.value(j, MEMBER)?,
            );
            let same: bool = only(vj)
                && (prm(vi)? - prm(vj)?).abs() <= PCONFUSION
                && (vi.parameter_on_line() - vj.parameter_on_line()).abs() <= PCONFUSION
                && arc(vi)? == arc(vj)?;
            if same {
                scan.vertices.remove(j, MEMBER)?;
                scan.nbvtx -= 1;
                scan.vertices.shift_after_removal(j);
            }
            Ok(same)
        },
    )
}

/// The sort and the removal of the duplicates of the same parameter on the line, over
/// the pairs `j < i`.
fn sweep_duplicates(v: &mut VertexList) -> Result<(), Failure> {
    let mut sort_again: bool = true;
    loop {
        let mut nbvtx: i32 = v.nb_vertex();
        if sort_again {
            v.bubble_sort(nbvtx, true, MEMBER)?;
        }
        sort_again = false;
        let mut sort_is_ok: bool = true;
        let mut i: i32 = 2;
        while i <= nbvtx && sort_is_ok {
            let mut j: i32 = 1;
            while j < i && sort_is_ok {
                sort_is_ok = !kill_pair(v, (i, j), &mut nbvtx)?;
                j += 1;
            }
            i += 1;
        }
        if sort_is_ok {
            return Ok(());
        }
    }
}

/// The pair `(i, j)` of the sweep (`VTX` is `i`, `VTXM1` is `j`); true when a vertex
/// was removed.
fn kill_pair(v: &mut VertexList, (i, j): (i32, i32), nbvtx: &mut i32) -> Result<bool, Failure> {
    let verdict: (bool, bool) = {
        let (vtx, vtxm1) = (v.value(i, MEMBER)?, v.value(j, MEMBER)?);
        if (vtxm1.parameter_on_line() - vtx.parameter_on_line()).abs() >= PCONFUSION {
            return Ok(false);
        }
        let first: (bool, bool) = verdict_on(vtx, vtxm1, true, false)?;
        if first.0 || first.1 {
            first
        } else {
            verdict_on(vtx, vtxm1, false, false)?
        }
    };
    let (removed, replacement) = match verdict {
        (true, _) => (i, j),
        (false, true) => (j, i - 1),
        (false, false) => return Ok(false),
    };
    follow_removal(v, removed, replacement);
    v.remove(removed, MEMBER)?;
    *nbvtx -= 1;
    Ok(true)
}

/// A periodic line (open at both ends) whose first vertex is at its first parameter
/// gets a copy of it at its last parameter when the curve closes there within `tol`.
fn close_periodic(
    v: &mut VertexList,
    curv: &IntAnaCurve,
    (first, last): (f64, f64),
    tol: f64,
) -> Result<(), Failure> {
    let nbvtx: i32 = v.nb_vertex();
    let (vtx0, vtxn) = (v.value(1, MEMBER)?.clone(), v.value(nbvtx, MEMBER)?.clone());
    if vtx0.parameter_on_line() != first {
        return Ok(());
    }
    if vtxn.parameter_on_line() != last {
        if curv.value(last)?.distance(&vtx0.value()) <= tol {
            let mut ovtx: IntPatchPoint = vtx0;
            ovtx.set_parameter(last);
            v.svtx.push(ovtx);
        }
        return Ok(());
    }
    // The native else branch tests `VTX0.ParameterOnLine() != ParamMinOnLine` inside
    // the branch where it is equal, so it never prepends; kept for the structure.
    if vtx0.parameter_on_line() != first && curv.value(first)?.distance(&vtx0.value()) <= tol {
        let mut ovtx: IntPatchPoint = vtxn;
        ovtx.set_parameter(first);
        v.svtx.insert(0, ovtx);
    }
    Ok(())
}

/// The first and the last vertices on no domain are removed (quirk kept: `indl`
/// decreases for each removal and `indf` never).
fn remove_free_ends(v: &mut VertexList) -> Result<(), Failure> {
    let free = |p: &IntPatchPoint| !p.is_on_dom_s1() && !p.is_on_dom_s2();
    let mut nbvtx: i32 = v.nb_vertex();
    if nbvtx > 1 && free(v.value(1, MEMBER)?) {
        v.remove(1, MEMBER)?;
        nbvtx -= 1;
        if v.lapt {
            v.indl -= 1;
        }
    }
    if nbvtx > 1 && free(v.value(nbvtx, MEMBER)?) {
        v.remove(nbvtx, MEMBER)?;
        if v.lapt {
            v.indl -= 1;
        }
    }
    Ok(())
}

/// Two neighbour vertices of the same parameter (within 1e-5) share their arcs and
/// vertices, and the previous one takes the point of the next one when they differ by
/// more than 1e-6, until nothing changes.
fn merge_same_parameter(v: &mut VertexList) -> Result<(), Failure> {
    let nbvtx: i32 = v.nb_vertex();
    loop {
        let mut sort_is_ok: bool = true;
        for i in 2..=nbvtx {
            let (mut vtx, mut vtxm1) =
                (v.value(i, MEMBER)?.clone(), v.value(i - 1, MEMBER)?.clone());
            if (vtx.parameter_on_line() - vtxm1.parameter_on_line()).abs() >= PCONFUSION {
                continue;
            }
            for on_first in [true, false] {
                share_domain(&mut vtx, &mut vtxm1, on_first)?;
            }
            if vtx.value().square_distance(&vtxm1.value()) > 1e-12 {
                take_point(&mut vtxm1, &vtx)?;
                sort_is_ok = false;
            }
            *v.value_mut(i, MEMBER)? = vtx;
            *v.value_mut(i - 1, MEMBER)? = vtxm1;
        }
        if sort_is_ok {
            return Ok(());
        }
    }
}

/// The arc and the vertex of one domain go to the neighbour that lacks them.
fn share_domain(
    vtx: &mut IntPatchPoint,
    vtxm1: &mut IntPatchPoint,
    on_first: bool,
) -> Result<(), Failure> {
    let is_vertex = |p: &IntPatchPoint| {
        if on_first {
            p.is_vertex_on_s1()
        } else {
            p.is_vertex_on_s2()
        }
    };
    let vertex = |p: &IntPatchPoint| {
        if on_first {
            p.vertex_on_s1()
        } else {
            p.vertex_on_s2()
        }
    };
    if on(vtx, on_first) && !on(vtxm1, on_first) {
        vtxm1.copy_arc_from(vtx, on_first)?;
    } else if on(vtxm1, on_first) && !on(vtx, on_first) {
        vtx.copy_arc_from(vtxm1, on_first)?;
    }
    if is_vertex(vtx) && !is_vertex(vtxm1) {
        vtxm1.set_vertex(on_first, &vertex(vtx)?);
        vtxm1.copy_arc_from(vtx, on_first)?;
    } else if is_vertex(vtxm1) && !is_vertex(vtx) {
        vtx.set_vertex(on_first, &vertex(vtxm1)?);
        vtx.copy_arc_from(vtxm1, on_first)?;
    }
    Ok(())
}

/// `VTXm1` takes the parameter, the point, the tolerance and the parameters of `VTX`
/// and gets its own arcs and vertices back (`SetValue` clears the domain flags).
fn take_point(vtxm1: &mut IntPatchPoint, vtx: &IntPatchPoint) -> Result<(), Failure> {
    let copy_vtx: IntPatchPoint = vtxm1.clone();
    vtxm1.set_parameter(vtx.parameter_on_line());
    vtxm1.set_value_tangent(vtx.value(), vtx.tolerance(), vtx.is_tangency_point());
    let (u1, v1, u2, v2) = vtx.parameters();
    vtxm1.set_parameters(u1, v1, u2, v2);
    for on_first in [true, false] {
        if on(&copy_vtx, on_first) {
            vtxm1.copy_arc_from(&copy_vtx, on_first)?;
        }
    }
    if copy_vtx.is_vertex_on_s1() {
        vtxm1.set_vertex(true, &copy_vtx.vertex_on_s1()?);
        vtxm1.copy_arc_from(&copy_vtx, true)?;
    }
    if copy_vtx.is_vertex_on_s2() {
        vtxm1.set_vertex(false, &copy_vtx.vertex_on_s2()?);
        vtxm1.copy_arc_from(&copy_vtx, false)?;
    }
    Ok(())
}
