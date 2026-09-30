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
// Rust translation of IntPatch_GLine::ComputeVertexParameters (IntPatch_GLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_GLine::ComputeVertexParameters`: the restriction filters, then the sort and
//! the removal of the duplicated vertices, with the seam treatment of the closed conics
//! (modified by eap, bug occ212, and by jgv for BUC61033).
use super::IntPatchIType;
use super::point::IntPatchPoint;
use super::vertices::{CountedVertices, VertexList, repeat_until_stable};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use std::f64::consts::PI;

const MEMBER: &str = "IntPatch_GLine::ComputeVertexParameters";

/// `Precision::PConfusion() * 1000.0` (1e-6).
fn pconfusion() -> f64 {
    precision::p_confusion() * 1000.0
}

fn on(v: &IntPatchPoint, on_first: bool) -> bool {
    if on_first {
        v.is_on_dom_s1()
    } else {
        v.is_on_dom_s2()
    }
}

fn same_arc(a: &IntPatchPoint, b: &IntPatchPoint, on_first: bool) -> Result<bool, Failure> {
    if on_first {
        return Ok(a.arc_on_s1()? == b.arc_on_s1()?);
    }
    Ok(a.arc_on_s2()? == b.arc_on_s2()?)
}

fn prm(v: &IntPatchPoint, on_first: bool) -> Result<f64, Failure> {
    if on_first {
        v.parameter_on_arc1()
    } else {
        v.parameter_on_arc2()
    }
}

/// The body on the vertices of a line of type `typ`.
pub(super) fn compute_vertex_parameters(
    vertices: &mut VertexList,
    typ: IntPatchIType,
) -> Result<(), Failure> {
    let bounds: [f64; 2] = [
        if vertices.fipt {
            vertices.param(vertices.indf, MEMBER)?
        } else {
            -100000.0
        },
        if vertices.lapt {
            vertices.param(vertices.indl, MEMBER)?
        } else {
            100000.0
        },
    ];
    let nbvtx: i32 = vertices.nb_vertex();
    let mut scan = CountedVertices { vertices, nbvtx };
    remove_free_at_domain_points(&mut scan)?;
    filter_same_arc_point(&mut scan, true)?;
    filter_same_arc_point(&mut scan, false)?;
    let mut sweep = SeamSweep {
        typ,
        bounds,
        u_ranges: [[real_last(), real_first()]; 2],
        to_break: false,
    };
    sweep.run(scan.vertices)?;
    let nbvtx: i32 = scan.vertices.nb_vertex();
    if nbvtx != 0 {
        scan.vertices.bubble_sort(nbvtx, false, MEMBER)?;
        // Quirk kept: the indices are reset, not the fipt/lapt flags.
        (scan.vertices.indl, scan.vertices.indf) = (nbvtx, 1);
    }
    Ok(())
}

/// A vertex on no domain at the parameter of a vertex on a domain is removed, while
/// more than two vertices are left.
fn remove_free_at_domain_points(scan: &mut CountedVertices<'_>) -> Result<(), Failure> {
    loop {
        let mut deleted: bool = false;
        repeat_once(scan, &mut deleted, &mut |scan, i, j| {
            let (vi, vj) = (
                scan.vertices.value(i, MEMBER)?,
                scan.vertices.value(j, MEMBER)?,
            );
            let dup: bool = (vi.is_on_dom_s1() || vi.is_on_dom_s2())
                && !vj.is_on_dom_s1()
                && !vj.is_on_dom_s2()
                && (vi.parameter_on_line() - vj.parameter_on_line()).abs() <= pconfusion();
            if dup {
                remove_strict(scan, j)?;
            }
            Ok(dup)
        })?;
        if !(deleted && scan.nbvtx > 2) {
            return Ok(());
        }
    }
}

/// One pass of the `for (i) for (j)` pair loops of the first filter (its `do while`
/// also tests the vertex count).
fn repeat_once(
    scan: &mut CountedVertices<'_>,
    deleted: &mut bool,
    pair: &mut dyn FnMut(&mut CountedVertices<'_>, i32, i32) -> Result<bool, Failure>,
) -> Result<(), Failure> {
    let mut i: i32 = 1;
    while i <= scan.nbvtx && !*deleted {
        let mut j: i32 = 1;
        while j <= scan.nbvtx && !*deleted {
            *deleted = i != j && pair(scan, i, j)?;
            j += 1;
        }
        i += 1;
    }
    Ok(())
}

/// Removes vertex `index`, shifting `indl`/`indf` when greater (the strict test of
/// IntPatch_GLine).
fn remove_strict(scan: &mut CountedVertices<'_>, index: i32) -> Result<(), Failure> {
    scan.vertices.remove(index, MEMBER)?;
    scan.nbvtx -= 1;
    scan.vertices.shift_after_removal_strict(index);
    Ok(())
}

/// Two vertices only on the same arc of one domain at the same parameter on it: the
/// vertex `j` is removed when `i` is a vertex of the domain of S1 (quirk kept: the S2
/// pass also tests `IsVertexOnS1`), else the vertex `i`.
fn filter_same_arc_point(scan: &mut CountedVertices<'_>, on_first: bool) -> Result<(), Failure> {
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
                && (prm(vi, on_first)? - prm(vj, on_first)?).abs() <= pconfusion()
                && same_arc(vi, vj, on_first)?;
            if same {
                let removed: i32 = if vi.is_vertex_on_s1() { j } else { i };
                remove_strict(scan, removed)?;
            }
            Ok(same)
        },
    )
}

/// The sort and the removal of the duplicates of the same parameter on the line, with
/// the locals the native loop keeps across its iterations.
struct SeamSweep {
    typ: IntPatchIType,
    /// `ParamMinOnLine`, `ParamMaxOnLine`.
    bounds: [f64; 2],
    /// `[u1min, u1max]` and `[u2min, u2max]`.
    u_ranges: [[f64; 2]; 2],
    /// `ToBreak`.
    to_break: bool,
}

/// The outcome of one pair of the sweep.
enum PairOutcome {
    /// Nothing changed.
    Kept,
    /// A vertex was removed or moved: the sweep restarts (`SortIsOK = false`), with a
    /// new sort when `sort_again`.
    Changed { sort_again: bool },
}

impl SeamSweep {
    fn run(&mut self, v: &mut VertexList) -> Result<(), Failure> {
        let mut sort_again: bool = true;
        loop {
            let mut nbvtx: i32 = v.nb_vertex();
            if sort_again {
                v.bubble_sort(nbvtx, true, MEMBER)?;
            }
            if self.to_break {
                return Ok(());
            }
            sort_again = false;
            let mut sort_is_ok: bool = true;
            let mut i: i32 = 2;
            while i <= nbvtx && sort_is_ok {
                let mut j: i32 = 1;
                while j <= nbvtx && sort_is_ok {
                    if i != j
                        && let PairOutcome::Changed { sort_again: again } =
                            self.pair(v, (i, j), &mut nbvtx)?
                    {
                        sort_is_ok = false;
                        sort_again |= again;
                    }
                    j += 1;
                }
                i += 1;
            }
            if sort_is_ok {
                return Ok(());
            }
        }
    }

    /// The pair `(i, j)` of the sweep (`VTX` is `i`, `VTXM1` is `j`).
    fn pair(
        &mut self,
        v: &mut VertexList,
        (i, j): (i32, i32),
        nbvtx: &mut i32,
    ) -> Result<PairOutcome, Failure> {
        let (vtx, vtxm1) = (v.value(i, MEMBER)?.clone(), v.value(j, MEMBER)?.clone());
        if (vtxm1.parameter_on_line() - vtx.parameter_on_line()).abs() >= pconfusion() {
            return Ok(PairOutcome::Kept);
        }
        let (kill, killm1) = kill_verdict(&vtx, &vtxm1)?;
        if kill {
            follow_removal(v, i, j);
            v.remove(i, MEMBER)?;
            *nbvtx -= 1;
            return Ok(PairOutcome::Changed { sort_again: false });
        }
        if killm1 {
            follow_removal(v, j, i - 1);
            v.remove(j, MEMBER)?;
            *nbvtx -= 1;
            return Ok(PairOutcome::Changed { sort_again: false });
        }
        if self.typ != IntPatchIType::Circle && self.typ != IntPatchIType::Ellipse {
            return Ok(PairOutcome::Kept);
        }
        self.seam_pair(v, (i, j), (&vtx, &vtxm1), *nbvtx)
    }

    /// Two points of the same parameter on a closed conic that cannot be merged: when
    /// their UV points differ by a period on one surface (a seam edge), one of them
    /// moves by 2 Pi (always from 2 Pi to 0).
    fn seam_pair(
        &mut self,
        v: &mut VertexList,
        (i, j): (i32, i32),
        (vtx, vtxm1): (&IntPatchPoint, &IntPatchPoint),
        nbvtx: i32,
    ) -> Result<PairOutcome, Failure> {
        let ponline: f64 = vtx.parameter_on_line();
        let pi_pi: f64 = PI + PI;
        let is2pi: bool = (ponline - pi_pi).abs() <= pconfusion();
        // The check is done only when the seam edge alone gives vertices.
        if nbvtx > 2 && !is2pi {
            return Ok(PairOutcome::Kept);
        }
        let new_param: f64 = if is2pi {
            0.0
        } else if ponline.abs() <= pconfusion() {
            pi_pi
        } else {
            ponline - pi_pi
        };
        let (a, b) = (params4(vtxm1), params4(vtx));
        let flag: i32 = (0..4)
            .filter(|&k| (a[k] - b[k]).abs() <= pconfusion())
            .fold(0, |f, k| f | (1 << k));
        let test_on: Option<usize> = match flag {
            14 => Some(0),
            11 => Some(1),
            _ => None,
        };
        let Some(surface) = test_on else {
            return Ok(PairOutcome::Kept);
        };
        let (ua, ub) = (a[2 * surface], b[2 * surface]);
        self.track_range(surface, ua, ub);
        let [min_on_line, max_on_line] = self.bounds;
        if !(new_param >= min_on_line && new_param <= max_on_line) {
            return Ok(PairOutcome::Kept);
        }
        let moves_vtx: bool = if new_param > ponline {
            ua < ub
        } else {
            ua > ub
        };
        let moved: i32 = if moves_vtx { i } else { j };
        v.value_mut(moved, MEMBER)?.set_parameter(new_param);
        Ok(PairOutcome::Changed { sort_again: true })
    }

    /// The U range of the seam pairs on one surface (BUC61033): the sweep stops after
    /// the next sort when a pair leaves it.
    fn track_range(&mut self, surface: usize, ua: f64, ub: f64) {
        let lower: f64 = if ua < ub { ua } else { ub };
        let upper: f64 = if ua < ub { ub } else { ua };
        let range: &mut [f64; 2] = &mut self.u_ranges[surface];
        if range[0] == real_last() {
            *range = [lower, upper];
            return;
        }
        if (lower - range[0]).abs() > pconfusion() {
            self.to_break = true;
        }
        if (upper - range[1]).abs() > pconfusion() {
            self.to_break = true;
        }
    }
}

/// `u1, v1, u2, v2`.
fn params4(p: &IntPatchPoint) -> [f64; 4] {
    let (u1, v1, u2, v2) = p.parameters();
    [u1, v1, u2, v2]
}

/// The first and last indices around the removal of `removed`: greater ones move back,
/// the removed one becomes `replacement`.
pub(super) fn follow_removal(v: &mut VertexList, removed: i32, replacement: i32) {
    let follow = |index: i32| -> i32 {
        if index > removed {
            index - 1
        } else if index == removed {
            replacement
        } else {
            index
        }
    };
    if v.lapt {
        v.indl = follow(v.indl);
    }
    if v.fipt {
        v.indf = follow(v.indf);
    }
}

/// The verdict on the two vertices of the same parameter: `(kill, killm1)`.
fn kill_verdict(vtx: &IntPatchPoint, vtxm1: &IntPatchPoint) -> Result<(bool, bool), Failure> {
    let verdict: (bool, bool) = verdict_on(vtx, vtxm1, true, true)?;
    if verdict.0 || verdict.1 {
        return Ok(verdict);
    }
    verdict_on(vtx, vtxm1, false, true)
}

/// The verdict of one domain: on the same arc, the one not on the other domain is the
/// duplicate (on the same arc of the other domain too: with `vertex_test`, IntPatch_GLine,
/// `VTX` unless `VTXM1` is a vertex of it; otherwise, IntPatch_ALine, `VTX`); when not
/// both on the domain, the one on no domain at all is.
pub(super) fn verdict_on(
    vtx: &IntPatchPoint,
    vtxm1: &IntPatchPoint,
    on_first: bool,
    vertex_test: bool,
) -> Result<(bool, bool), Failure> {
    let other: bool = !on_first;
    if !(on(vtxm1, on_first) && on(vtx, on_first)) {
        if on(vtxm1, other) || on(vtx, other) {
            return Ok((false, false));
        }
        return Ok((
            on(vtxm1, on_first) && !on(vtx, on_first),
            on(vtx, on_first) && !on(vtxm1, on_first),
        ));
    }
    if !same_arc(vtxm1, vtx, on_first)? {
        return Ok((false, false));
    }
    if !on(vtxm1, other) {
        return Ok((false, on(vtx, other)));
    }
    if !on(vtx, other) {
        return Ok((true, false));
    }
    if !same_arc(vtxm1, vtx, other)? {
        return Ok((false, false));
    }
    if !vertex_test {
        return Ok((true, false));
    }
    let vertex_on_other: bool = if other {
        vtxm1.is_vertex_on_s1()
    } else {
        vtxm1.is_vertex_on_s2()
    };
    Ok((vertex_on_other, !vertex_on_other))
}
