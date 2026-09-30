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
// Rust translation of the vertex filters of IntPatch_WLine::ComputeVertexParameters
// (IntPatch_WLine.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The filters of `IntPatch_WLine::ComputeVertexParameters` that remove duplicated
//! vertices of the restrictions.
use super::compute::{MEMBER, VertexPass};
use crate::int_patch::point::IntPatchPoint;
use crate::int_patch::vertices::repeat_until_stable;
use rustycad_tkernel::standard::Failure;

/// Whether the vertex is on the domain of one surface only (`on_first`: S1).
fn only_on(v: &IntPatchPoint, on_first: bool) -> bool {
    if on_first {
        v.is_on_dom_s1() && !v.is_on_dom_s2()
    } else {
        v.is_on_dom_s2() && !v.is_on_dom_s1()
    }
}

/// Whether the two vertices are on the same arc of the domain.
fn same_arc(vi: &IntPatchPoint, vj: &IntPatchPoint, on_first: bool) -> Result<bool, Failure> {
    if on_first {
        return Ok(vi.arc_on_s1()? == vj.arc_on_s1()?);
    }
    Ok(vi.arc_on_s2()? == vj.arc_on_s2()?)
}

/// Removes vertex `j` and shifts the first and last indices (`indl >= j`).
fn remove_shifted(pass: &mut VertexPass<'_>, j: i32) -> Result<(), Failure> {
    pass.state.vertices.remove(j, MEMBER)?;
    pass.nbvtx -= 1;
    pass.state.vertices.shift_after_removal(j);
    Ok(())
}

/// Two vertices only on the same restriction must not have the same parameter on the
/// line: the second is removed. The pass on S1 also raises `Tol` to the greatest
/// vertex tolerance it meets.
pub(super) fn filter_same_parameter(
    pass: &mut VertexPass<'_>,
    on_first: bool,
) -> Result<(), Failure> {
    repeat_until_stable(
        pass,
        &mut |pass, i| {
            let vi: &IntPatchPoint = pass.vtx(i)?;
            let (tol, only) = (vi.tolerance(), only_on(vi, on_first));
            if on_first && tol > pass.tol {
                pass.tol = tol; //-- 9 oct 97
            }
            Ok(only)
        },
        &mut |pass, i, j| {
            let (vi, vj) = (pass.vtx(i)?, pass.vtx(j)?);
            let same: bool = only_on(vj, on_first)
                && vi.parameter_on_line() == vj.parameter_on_line()
                && same_arc(vi, vj, on_first)?;
            if same {
                remove_shifted(pass, j)?;
            }
            Ok(same)
        },
    )
}

/// The verdict on two vertices of the same parameter on the line: `(kill, killm1)`,
/// the first (`VTX`, index `i`) or the second (`VTXM1`, index `j`) is a duplicate.
fn kill_verdict(vtx: &IntPatchPoint, vtxm1: &IntPatchPoint) -> Result<(bool, bool), Failure> {
    let verdict: (bool, bool) = kill_on(vtx, vtxm1, true)?;
    if verdict.0 || verdict.1 {
        return Ok(verdict);
    }
    kill_on(vtx, vtxm1, false)
}

/// `OnS1 OnS1` (or `OnS2 OnS2`): both on the same arc of one domain; the one that is not
/// on the other domain (or on the same arc of it) is the duplicate.
fn kill_on(
    vtx: &IntPatchPoint,
    vtxm1: &IntPatchPoint,
    on_first: bool,
) -> Result<(bool, bool), Failure> {
    let on = |v: &IntPatchPoint| {
        if on_first {
            v.is_on_dom_s1()
        } else {
            v.is_on_dom_s2()
        }
    };
    let other_on = |v: &IntPatchPoint| {
        if on_first {
            v.is_on_dom_s2()
        } else {
            v.is_on_dom_s1()
        }
    };
    if !(on(vtxm1) && on(vtx)) || !same_arc(vtxm1, vtx, on_first)? {
        return Ok((false, false));
    }
    if other_on(vtxm1) {
        if !other_on(vtx) {
            return Ok((true, false));
        }
        return Ok((same_arc(vtxm1, vtx, !on_first)?, false));
    }
    Ok((false, other_on(vtx)))
}

/// The duplicates of the same parameter on the line: `kill` removes the vertex `i`,
/// `killm1` the vertex `j` (the first and last indices are not shifted, as natively).
pub(super) fn kill_duplicates(pass: &mut VertexPass<'_>) -> Result<(), Failure> {
    repeat_until_stable(pass, &mut |_, _| Ok(true), &mut |pass, i, j| {
        let (vtx, vtxm1) = (pass.vtx(i)?, pass.vtx(j)?);
        if vtxm1.parameter_on_line() != vtx.parameter_on_line() {
            return Ok(false);
        }
        let (kill, killm1) = kill_verdict(vtx, vtxm1)?;
        let removed: Option<i32> = if kill {
            Some(i)
        } else if killm1 {
            Some(j)
        } else {
            None
        };
        let Some(index) = removed else {
            return Ok(false);
        };
        pass.state.vertices.remove(index, MEMBER)?;
        pass.nbvtx -= 1;
        Ok(true)
    })
}

/// Of two neighbour vertices of the same parameter, the one on no domain is removed.
pub(super) fn remove_free_twins(pass: &mut VertexPass<'_>) -> Result<(), Failure> {
    pass.state
        .vertices
        .remove_free_twins(&mut pass.nbvtx, MEMBER)
}

/// Two vertices only on the same restriction, with neighbour parameters on the line and
/// the same transition type: the second is removed unless it is the first or the last
/// point of the line (this would make a gap).
pub(super) fn filter_neighbours_on_restriction(
    pass: &mut VertexPass<'_>,
    on_first: bool,
) -> Result<(), Failure> {
    pass.nbvtx = pass.state.vertices.nb_vertex();
    repeat_until_stable(
        pass,
        &mut |pass, i| Ok(only_on(pass.vtx(i)?, on_first)),
        &mut |pass, i, j| {
            let nb_pnts: i32 = pass.curv.nb_points();
            let (vi, vj) = (pass.vtx(i)?, pass.vtx(j)?);
            if !only_on(vj, on_first) {
                return Ok(false);
            }
            let (pi, pj) = (vi.parameter_on_line(), vj.parameter_on_line());
            if !(pi == pj + 1.0 || pi == pj - 1.0) || !same_arc(vi, vj, on_first)? {
                return Ok(false);
            }
            let (t1, t2) = if on_first {
                (vi.transition_line_arc1()?, vj.transition_line_arc1()?)
            } else {
                (vi.transition_line_arc2()?, vj.transition_line_arc2()?)
            };
            if t1.transition_type() != t2.transition_type() || pj == 1.0 || pj == f64::from(nb_pnts)
            {
                return Ok(false);
            }
            remove_shifted(pass, j)?;
            Ok(true)
        },
    )
}
