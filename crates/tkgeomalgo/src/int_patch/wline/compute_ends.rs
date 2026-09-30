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
// Rust translation of the end-vertex phase of IntPatch_WLine::ComputeVertexParameters
// (IntPatch_WLine.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The phase of `IntPatch_WLine::ComputeVertexParameters` that adds the vertices of the
//! first and the last point of the line when they are missing.
use super::compute::VertexPass;
use super::compute_points::compare_vertex_and_point;
use crate::int_patch::point::IntPatchPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// Flags the vertices on the first and the last point of the line, then creates the
/// missing ones. Quirk kept: a first vertex prepended here does not shift `indl`.
pub(super) fn add_end_vertices(pass: &mut VertexPass<'_>) -> Result<(), Failure> {
    let (mut b_first, mut b_last) = (false, false);
    let nbponline: i32 = pass.curv.nb_points();
    for i in 1..=pass.nbvtx {
        let pol: f64 = pass.param(i)?;
        if pol == 1.0 {
            b_first = true;
            pass.state.vertices.set_first_point(i);
        }
        if pol == f64::from(nbponline) {
            b_last = true;
            pass.state.vertices.set_last_point(i);
        }
    }
    if !b_first {
        let vtx: IntPatchPoint = end_vertex(pass, 1)?;
        pass.state.vertices.svtx.insert(0, vtx);
        pass.nbvtx += 1;
        pass.state.vertices.set_first_point(1);
    }
    if !b_last {
        let vtx: IntPatchPoint = end_vertex(pass, nbponline)?;
        pass.state.vertices.svtx.push(vtx);
        pass.nbvtx += 1;
        let nbvtx: i32 = pass.nbvtx;
        pass.state.vertices.set_last_point(nbvtx);
    }
    Ok(())
}

/// The vertex of the point `index` of the line: a copy of the first vertex on the point
/// (within `Precision::Confusion()`, not the vertex tolerance, to avoid loops), or a new
/// point of tolerance `Tol`, with the parameters of the point.
fn end_vertex(pass: &VertexPass<'_>, index: i32) -> Result<IntPatchPoint, Failure> {
    let point = pass.curv.value(index)?;
    let (pu1, pv1, pu2, pv2) = point.parameters();
    let mut found: Option<IntPatchPoint> = None;
    let mut i: i32 = 1;
    while found.is_none() && i <= pass.nbvtx {
        let v: &IntPatchPoint = pass.vtx(i)?;
        if compare_vertex_and_point(v.value(), point.value(), precision::confusion()) {
            found = Some(v.clone());
        }
        i += 1;
    }
    let mut vtx: IntPatchPoint = found.unwrap_or_else(|| {
        let mut fresh = IntPatchPoint::new();
        fresh.set_value_tangent(point.value(), pass.tol, false);
        fresh
    });
    vtx.set_parameters(pu1, pv1, pu2, pv2);
    vtx.set_parameter(f64::from(index));
    Ok(vtx)
}
