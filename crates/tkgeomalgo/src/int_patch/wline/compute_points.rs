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
// Rust translation of the point phases of IntPatch_WLine::ComputeVertexParameters and the
// static helpers RecadreMemePeriode and CompareVertexAndPoint (IntPatch_WLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The phases of `IntPatch_WLine::ComputeVertexParameters` that merge the vertices with
//! the walking points: each vertex replaces the point it matches, or is inserted.
use super::IntPatchWLType;
use super::compute::VertexPass;
use crate::int_surf::IntSurfPntOn2S;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// The file-static `RecadreMemePeriode(u1, v1, u2, v2, anu1, ..., V2Period)` on one
/// parameter: moved by whole periods to within `0.8 * period` of the reference (a null
/// period leaves it).
fn recadre(mut value: f64, reference: f64, period: f64) -> f64 {
    if period != 0.0 {
        while reference - value > 0.8 * period {
            value += period;
        }
        while value - reference > 0.8 * period {
            value -= period;
        }
    }
    value
}

/// The file-static `RecadreMemePeriode(POn2S, RefPOn2S, up1, vp1, up2, vp2)`, in the
/// native order u1, u2, v1, v2.
pub(super) fn recadre_meme_periode(
    pon2s: &mut IntSurfPntOn2S,
    reference: &IntSurfPntOn2S,
    periods: [f64; 4],
) {
    let (u1, v1, u2, v2) = pon2s.parameters();
    let (pu1, pv1, pu2, pv2) = reference.parameters();
    let [up1, vp1, up2, vp2] = periods;
    let u1: f64 = recadre(u1, pu1, up1);
    let u2: f64 = recadre(u2, pu2, up2);
    let v1: f64 = recadre(v1, pv1, vp1);
    let v2: f64 = recadre(v2, pv2, vp2);
    pon2s.set_parameters(u1, v1, u2, v2);
}

/// The file-static `CompareVertexAndPoint(V, P, Tol)`.
pub(super) fn compare_vertex_and_point(v: Point3, p: Point3, tol: f64) -> bool {
    let sq_dist: f64 = v.square_distance(&p);
    let sq_tol: f64 = tol * tol;
    sq_dist <= sq_tol
}

/// Removes the point `i` of the line and moves back the vertices after it.
fn remove_point_and_shift(pass: &mut VertexPass<'_>, i: i32) -> Result<(), Failure> {
    pass.curv.remove_point(i)?;
    pass.nbponline -= 1;
    for j in 1..=pass.nbvtx {
        let indicevertex: f64 = pass.param(j)?;
        if indicevertex >= f64::from(i) {
            pass.vtx_mut(j)?.set_parameter(indicevertex - 1.0);
        }
    }
    Ok(())
}

/// Removes the confused points of the line (closer than `Precision::Confusion()`),
/// while it keeps more than `min_points` points.
fn remove_confused(pass: &mut VertexPass<'_>, min_points: i32) -> Result<(), Failure> {
    let dmini: f64 = precision::square_confusion();
    let mut i: i32 = 2;
    while i <= pass.nbponline && pass.nbponline > min_points {
        let (p1, p2) = (pass.curv.value(i - 1)?, pass.curv.value(i)?);
        if p1.value().square_distance(&p2.value()) < dmini {
            remove_point_and_shift(pass, i)?;
            // modified by NIZNHY-PKV Mon Feb 11 09:28:02 2002
            i -= 1;
        }
        i += 1;
    }
    Ok(())
}

/// The confused points of the walking line are removed (more than 2 points kept).
pub(super) fn remove_confused_points(pass: &mut VertexPass<'_>) -> Result<(), Failure> {
    remove_confused(pass, 2)
}

/// The vertex `i` against the points of the line around its parameter: it replaces a
/// point it matches within its tolerance, or it is inserted as a new point.
pub(super) fn merge_vertices_with_points(pass: &mut VertexPass<'_>, i: i32) -> Result<(), Failure> {
    let indicevertex: f64 = pass.param(i)?;
    // `(int)indicevertex`: truncation toward zero (saturating in Rust where C is undefined).
    let indicevertexonline: i32 = indicevertex as i32;
    if indicevertexonline < 1 {
        return merge_before_line(pass, i);
    }
    let substitution: bool = substitute_near_points(pass, i, indicevertexonline)?;
    // Remove the duplicated points (the line keeps more than 1 point).
    if substitution {
        remove_confused(pass, 1)?;
        return Ok(());
    }
    insert_vertex_point(pass, i, indicevertexonline)
}

/// A vertex before the line: it replaces the first point it matches, or it is inserted
/// before it (quirk kept: the other parameters shift for `j != 1`, not `j != i`).
fn merge_before_line(pass: &mut VertexPass<'_>, i: i32) -> Result<(), Failure> {
    let (p, v_tol) = (pass.vtx(i)?.value(), pass.vtx(i)?.tolerance());
    let first: IntSurfPntOn2S = pass.curv.value(1)?;
    let mut pon2s: IntSurfPntOn2S = pass.vtx(i)?.pnt_on_2s();
    recadre_meme_periode(&mut pon2s, &first, pass.state.periods);
    if compare_vertex_and_point(p, first.value(), v_tol) {
        if pass.state.creation_way == IntPatchWLType::ImpImp {
            // The 3d point of the vertex, the 2d points of the point on the curve.
            pass.curv.set_point(1, pon2s.value())?;
            let (mu1, mv1, mu2, mv2) = pass.curv.value(1)?.parameters();
            let vtx = pass.vtx_mut(i)?;
            vtx.set_parameter(1.0);
            vtx.set_parameters(mu1, mv1, mu2, mv2);
        } else {
            pass.curv.set_value(1, &pon2s)?;
        }
        pass.vtx_mut(i)?.set_parameter(1.0);
        return Ok(());
    }
    pass.curv.insert_before(1, &pon2s)?;
    pass.vtx_mut(i)?.set_parameter(1.0);
    pass.nbponline += 1;
    for j in 2..=pass.nbvtx {
        let t: f64 = pass.param(j)?;
        if t > 1.0 {
            pass.vtx_mut(j)?.set_parameter(t + 1.0);
        }
    }
    Ok(())
}

/// The vertex against the points `indicevertexonline + 1` down to `- 1`: every point it
/// matches is replaced by it (the loop does not stop at the first one).
fn substitute_near_points(
    pass: &mut VertexPass<'_>,
    i: i32,
    indicevertexonline: i32,
) -> Result<bool, Failure> {
    let (p, v_tol) = (pass.vtx(i)?.value(), pass.vtx(i)?.tolerance());
    let mut mu: [f64; 4] = params4(&pass.curv.value(indicevertexonline)?);
    let mut substitution: bool = false;
    for k in (indicevertexonline - 1..=indicevertexonline + 1).rev() {
        if k <= 0
            || k > pass.nbponline
            || !compare_vertex_and_point(p, pass.curv.value(k)?.value(), v_tol)
        {
            continue;
        }
        let mut pon2s: IntSurfPntOn2S = pass.vtx(i)?.pnt_on_2s();
        recadre_meme_periode(&mut pon2s, &pass.curv.value(k)?, pass.state.periods);
        if pass.state.creation_way == IntPatchWLType::ImpImp {
            // The 3d point of the vertex, the 2d points of the point `indicevertexonline`.
            pass.curv.set_point(k, pon2s.value())?;
            pass.curv.set_uv(k, true, mu[0], mu[1])?;
            pass.curv.set_uv(k, false, mu[2], mu[3])?;
        } else {
            pass.curv.set_value(k, &pon2s)?;
            mu = params4(&pon2s);
        }
        let vtx = pass.vtx_mut(i)?;
        vtx.set_parameter(f64::from(k));
        vtx.set_parameters(mu[0], mu[1], mu[2], mu[3]);
        substitution = true;
    }
    Ok(substitution)
}

fn params4(p: &IntSurfPntOn2S) -> [f64; 4] {
    let (u1, v1, u2, v2) = p.parameters();
    [u1, v1, u2, v2]
}

/// The vertex matches no point: it is inserted after the point `indicevertexonline`
/// (appended beyond the last point), and the vertices after it move forward.
fn insert_vertex_point(
    pass: &mut VertexPass<'_>,
    i: i32,
    indicevertexonline: i32,
) -> Result<(), Failure> {
    let mut pon2s: IntSurfPntOn2S = pass.vtx(i)?.pnt_on_2s();
    if indicevertexonline >= pass.nbponline {
        recadre_meme_periode(
            &mut pon2s,
            &pass.curv.value(pass.nbponline)?,
            pass.state.periods,
        );
        pass.curv.add(&pon2s);
    } else {
        let next: IntSurfPntOn2S = pass.curv.value(indicevertexonline + 1)?;
        recadre_meme_periode(&mut pon2s, &next, pass.state.periods);
        pass.curv.insert_before(indicevertexonline + 1, &pon2s)?;
    }
    pass.vtx_mut(i)?
        .set_parameter(f64::from(indicevertexonline + 1));
    pass.nbponline += 1;
    for j in 1..=pass.nbvtx {
        let t: f64 = pass.param(j)?;
        if j != i && t > f64::from(indicevertexonline) {
            pass.vtx_mut(j)?.set_parameter(t + 1.0);
        }
    }
    Ok(())
}
