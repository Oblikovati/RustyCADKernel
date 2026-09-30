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
// Rust translation of IntPatch_WLine::ComputeVertexParameters and its static helpers
// SameVtxRst and CompareVerticesOnSurf (IntPatch_WLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_WLine::ComputeVertexParameters`: the phases in the native order. The
//! locals the native body keeps across its phases (`Tol`, `nbvtx`, `nbponline`) live in
//! [`VertexPass`], so each phase reads what the native code reads, stale counts
//! included.
use super::WLineState;
use super::compute_ends::add_end_vertices;
use super::compute_filters::{
    filter_neighbours_on_restriction, filter_same_parameter, kill_duplicates, remove_free_twins,
};
use super::compute_points::{merge_vertices_with_points, remove_confused_points};
use crate::int_patch::point::IntPatchPoint;
use crate::int_patch::vertices::VertexScan;
use crate::int_surf::IntSurfLineOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// The member name the sequence reads of the body report.
pub(super) const MEMBER: &str = "IntPatch_WLine::ComputeVertexParameters";

/// The state of one `ComputeVertexParameters` run.
pub(super) struct VertexPass<'a> {
    pub(super) state: &'a mut WLineState,
    pub(super) curv: &'a IntSurfLineOn2S,
    /// `Tol`: the given tolerance, raised to the greatest vertex tolerance.
    pub(super) tol: f64,
    pub(super) nbvtx: i32,
    pub(super) nbponline: i32,
}

impl VertexScan for VertexPass<'_> {
    fn scan_len(&self) -> i32 {
        self.nbvtx
    }
}

impl VertexPass<'_> {
    /// `svtx.Value(Index)`.
    pub(super) fn vtx(&self, index: i32) -> Result<&IntPatchPoint, Failure> {
        self.state.vertices.value(index, MEMBER)
    }

    /// `svtx.ChangeValue(Index)`.
    pub(super) fn vtx_mut(&mut self, index: i32) -> Result<&mut IntPatchPoint, Failure> {
        self.state.vertices.value_mut(index, MEMBER)
    }

    /// `svtx.Value(Index).ParameterOnLine()`.
    pub(super) fn param(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.vtx(index)?.parameter_on_line())
    }
}

/// The body of `ComputeVertexParameters(RTol)` on the locked line state.
pub(super) fn compute_vertex_parameters(
    state: &mut WLineState,
    curv: &IntSurfLineOn2S,
    rtol: f64,
) -> Result<(), Failure> {
    let nbvtx: i32 = state.vertices.nb_vertex();
    let mut pass = VertexPass {
        state,
        curv,
        tol: rtol,
        nbvtx,
        nbponline: 0,
    };
    duplicate_seam_vertices(&mut pass)?;
    filter_same_parameter(&mut pass, true)?;
    filter_same_parameter(&mut pass, false)?;
    pass.nbvtx = pass.state.vertices.nb_vertex();
    pass.nbponline = curv.nb_points();
    pass.state.vertices.bubble_sort(pass.nbvtx, false, MEMBER)?;
    remove_confused_points(&mut pass)?;
    for i in 1..=pass.nbvtx {
        merge_vertices_with_points(&mut pass, i)?;
    }
    kill_duplicates(&mut pass)?;
    remove_free_twins(&mut pass)?;
    add_end_vertices(&mut pass)?;
    filter_neighbours_on_restriction(&mut pass, true)?;
    filter_neighbours_on_restriction(&mut pass, false)?;
    let nbvtx: i32 = pass.state.vertices.nb_vertex();
    pass.state.vertices.bubble_sort(nbvtx, false, MEMBER)
}

/// The seam phase: a vertex on a restriction of one surface is duplicated on the
/// restriction of a close vertex (in 3d within the greater tolerance, or with the same
/// parameters on a surface), until no vertex is created.
fn duplicate_seam_vertices(pass: &mut VertexPass<'_>) -> Result<(), Failure> {
    loop {
        pass.nbvtx = pass.state.vertices.nb_vertex();
        let mut encoreunefois: bool = false;
        let mut i: i32 = 1;
        while i <= pass.nbvtx && !encoreunefois {
            let mut j: i32 = 1;
            while j <= pass.nbvtx && !encoreunefois {
                if i != j {
                    encoreunefois = seam_pair(pass, i, j)?;
                }
                j += 1;
            }
            i += 1;
        }
        if !encoreunefois {
            return Ok(());
        }
    }
}

/// One `(i, j)` pair of the seam phase; returns the new `encoreunefois`.
fn seam_pair(pass: &mut VertexPass<'_>, i: i32, j: i32) -> Result<bool, Failure> {
    let (vi, vj) = (pass.vtx(i)?.clone(), pass.vtx(j)?.clone());
    if vi.parameter_on_line() == vj.parameter_on_line() {
        return Ok(false);
    }
    let d: f64 = vi.value().distance(&vj.value());
    let (toli, tolj) = (vi.tolerance(), vj.tolerance());
    let maxtol: f64 = toli.max(tolj);
    // MSV Oct 30, 2001: compare in 2D space also; increase tolerances.
    if !(d < maxtol
        || compare_vertices_on_surf(&vi, &vj, true)
        || compare_vertices_on_surf(&vi, &vj, false))
    {
        return Ok(false);
    }
    let newtols: [f64; 2] = [toli.max(tolj + d * 1.01), tolj.max(toli + d * 1.01)];
    let mut encoreunefois: bool = false;
    for on_first in [true, false] {
        encoreunefois = seam_creation(pass, (i, j), newtols, on_first, encoreunefois)?;
    }
    Ok(encoreunefois)
}

/// One creation of the seam phase (`REF:S1(i) S2(j)` when `on_first`, `REF:S2(i)
/// S1(j)` otherwise). Quirk kept: when the vertex already exists, `encoreunefois` is
/// reset to false, even after the other creation set it.
fn seam_creation(
    pass: &mut VertexPass<'_>,
    (i, j): (i32, i32),
    [newtoli, newtolj]: [f64; 2],
    on_first: bool,
    mut encoreunefois: bool,
) -> Result<bool, Failure> {
    let (vi, vj) = (pass.vtx(i)?.clone(), pass.vtx(j)?.clone());
    if !must_create(&vi, &vj, on_first)? {
        return Ok(encoreunefois);
    }
    let mut vtx: IntPatchPoint = vj;
    vtx.copy_arc_from(&vi, on_first)?;
    let mut k: i32 = 1;
    while !encoreunefois && k <= pass.nbvtx {
        if same_vtx_rst(pass.vtx(k)?, &vtx)? {
            encoreunefois = true;
        }
        k += 1;
    }
    if encoreunefois {
        return Ok(false);
    }
    pass.vtx_mut(i)?.set_tolerance(newtoli);
    pass.vtx_mut(j)?.set_tolerance(newtolj);
    vtx.set_tolerance(newtolj);
    pass.state.vertices.svtx.push(vtx);
    Ok(true)
}

/// `acreer`: `vi` is on the restriction and `vj` is not on it, or on another arc.
fn must_create(vi: &IntPatchPoint, vj: &IntPatchPoint, on_first: bool) -> Result<bool, Failure> {
    if on_first {
        if !vi.is_on_dom_s1() {
            return Ok(false);
        }
        return Ok(!vj.is_on_dom_s1() || vj.arc_on_s1()? != vi.arc_on_s1()?);
    }
    if !vi.is_on_dom_s2() {
        return Ok(false);
    }
    Ok(!vj.is_on_dom_s2() || vj.arc_on_s2()? != vi.arc_on_s2()?)
}

/// The file-static `SameVtxRst(vtx1, vtx2)`: the two vertices are on the same arcs of
/// both domains at the same parameters.
fn same_vtx_rst(vtx1: &IntPatchPoint, vtx2: &IntPatchPoint) -> Result<bool, Failure> {
    if vtx1.is_on_dom_s1() != vtx2.is_on_dom_s1() {
        return Ok(false);
    }
    if vtx1.is_on_dom_s1()
        && (vtx1.arc_on_s1()? != vtx2.arc_on_s1()?
            || vtx1.parameter_on_arc1()? != vtx2.parameter_on_arc1()?)
    {
        return Ok(false);
    }
    if vtx1.is_on_dom_s2() != vtx2.is_on_dom_s2() {
        return Ok(false);
    }
    if vtx1.is_on_dom_s2()
        && (vtx1.arc_on_s2()? != vtx2.arc_on_s2()?
            || vtx1.parameter_on_arc2()? != vtx2.parameter_on_arc2()?)
    {
        return Ok(false);
    }
    Ok(true)
}

/// The file-static `CompareVerticesOnSurf(vtx1, vtx2, onFirst)`: the parameters on a
/// surface are within `Precision::PConfusion()`.
fn compare_vertices_on_surf(vtx1: &IntPatchPoint, vtx2: &IntPatchPoint, on_first: bool) -> bool {
    let ((u1, v1), (u2, v2)) = if on_first {
        (vtx1.parameters_on_s1(), vtx2.parameters_on_s1())
    } else {
        (vtx1.parameters_on_s2(), vtx2.parameters_on_s2())
    };
    let (tol_u, tol_v) = (precision::p_confusion(), precision::p_confusion());
    (u1 - u2).abs() <= tol_u && (v1 - v2).abs() <= tol_v
}
