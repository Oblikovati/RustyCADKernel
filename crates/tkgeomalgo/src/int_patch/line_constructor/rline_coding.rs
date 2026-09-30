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
// Rust translation of the static helper TestWLineToRLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_LineConstructor.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TestWLineToRLine`: a walking line whose end vertices are on a common arc of a
//! domain, and whose middle point is on that arc, is coded as a restriction line.
use super::wline_rline::{Domains, params_on, project_on_arc, test_if_wline_is_restriction};
use crate::int_patch::{
    IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchRLine, IntPatchWLine,
};
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfLineOn2S, IntSurfSituation, IntSurfTransition};
use crate::intf::native_int_cast;
use rustycad_tkernel::standard::{Failure, Handle};

/// The end vertices of the walking line: the parameters of its first and last vertices
/// on the line, and the vertices at each of them (`indicesV1`, `indicesV2`).
struct LineEnds {
    param_min: i32,
    param_max: i32,
    indices_v1: Vec<i32>,
    indices_v2: Vec<i32>,
}

impl LineEnds {
    /// The ends of the line; `None` for fewer than 2 vertices or an empty range.
    fn new(wline: &IntPatchWLine) -> Result<Option<Self>, Failure> {
        let nbvtx: i32 = wline.nb_vertex();
        if nbvtx < 2 {
            return Ok(None);
        }
        let param_min: i32 = native_int_cast(wline.vertex(1)?.parameter_on_line());
        let param_max: i32 = native_int_cast(wline.vertex(nbvtx)?.parameter_on_line());
        if param_min >= param_max {
            return Ok(None);
        }
        let (mut indices_v1, mut indices_v2) = (Vec::new(), Vec::new());
        for iv in 1..=nbvtx {
            let plin: i32 = native_int_cast(wline.vertex(iv)?.parameter_on_line());
            if plin == param_min {
                indices_v1.push(iv);
            } else if plin == param_max {
                indices_v2.push(iv);
            }
        }
        Ok(Some(Self {
            param_min,
            param_max,
            indices_v1,
            indices_v2,
        }))
    }
}

/// The arcs resolved for the vertices of one side that are not on its domain: the
/// nearest arc of the domain within the tolerance, and the parameter on it.
struct Resolved {
    arcs: Vec<Option<TheArc>>,
    params: Vec<f64>,
}

impl Resolved {
    /// Resolves the arcs of the vertices of `wline` for the side `on_first`.
    fn new(
        wline: &IntPatchWLine,
        domains: &Domains<'_>,
        on_first: bool,
        tol_arc: f64,
    ) -> Result<Self, Failure> {
        let nbvtx: usize = usize::try_from(wline.nb_vertex()).unwrap_or_default();
        let mut resolved = Self {
            arcs: vec![None; nbvtx + 1],
            params: vec![0.0; nbvtx + 1],
        };
        let (surf, domain) = domains.side(on_first);
        for iv in 1..=nbvtx {
            let vertex: IntPatchPoint = wline.vertex(iv as i32)?;
            if is_on_dom(&vertex, on_first) {
                continue;
            }
            let ip: i32 = native_int_cast(vertex.parameter_on_line());
            let uv: (f64, f64) = params_on(&wline.point(ip)?, on_first);
            let mut distmin: f64 = f64::MAX;
            domain.init();
            while domain.more() {
                let arc: TheArc = domain.value()?;
                let (ok, par, dist) = project_on_arc(uv, &arc, surf, tol_arc)?;
                if ok && dist < distmin {
                    (resolved.arcs[iv], resolved.params[iv], distmin) = (Some(arc), par, dist);
                }
                domain.next();
            }
        }
        Ok(resolved)
    }

    /// The arc of the vertex `iv`: its own when it is on the domain, else the resolved
    /// one (or none).
    fn arc_of(
        &self,
        wline: &IntPatchWLine,
        iv: i32,
        on_first: bool,
    ) -> Result<Option<TheArc>, Failure> {
        let vertex: IntPatchPoint = wline.vertex(iv)?;
        if is_on_dom(&vertex, on_first) {
            let arc: TheArc = if on_first {
                vertex.arc_on_s1()?
            } else {
                vertex.arc_on_s2()?
            };
            return Ok((!arc.is_null()).then_some(arc));
        }
        Ok(self.arcs[iv as usize].clone())
    }

    /// Whether an arc was resolved for the vertex `iv`.
    fn is_resolved(&self, iv: i32) -> bool {
        self.arcs[iv as usize].is_some()
    }
}

fn is_on_dom(vertex: &IntPatchPoint, on_first: bool) -> bool {
    if on_first {
        vertex.is_on_dom_s1()
    } else {
        vertex.is_on_dom_s2()
    }
}

/// The arcs common to both ends of the line, in the order of the second end
/// (`NCollection_IndexedMap` of the arcs of the first end, then of the common arcs).
fn common_arcs(
    wline: &IntPatchWLine,
    ends: &LineEnds,
    resolved: &Resolved,
    on_first: bool,
) -> Result<Vec<TheArc>, Failure> {
    let mut map_arcs_v1: Vec<TheArc> = Vec::new();
    for &iv in &ends.indices_v1 {
        if let Some(arc) = resolved.arc_of(wline, iv, on_first)?
            && !map_arcs_v1.contains(&arc)
        {
            map_arcs_v1.push(arc);
        }
    }
    let mut map_arcs: Vec<TheArc> = Vec::new();
    for &iv in &ends.indices_v2 {
        if let Some(arc) = resolved.arc_of(wline, iv, on_first)?
            && map_arcs_v1.contains(&arc)
            && !map_arcs.contains(&arc)
        {
            map_arcs.push(arc);
        }
    }
    Ok(map_arcs)
}

/// The first vertex of `indices` linked with `arc` (0 when none).
fn vertex_on_arc(
    wline: &IntPatchWLine,
    (resolved, on_first): (&Resolved, bool),
    indices: &[i32],
    arc: &TheArc,
) -> Result<i32, Failure> {
    for &iv in indices {
        if resolved.arc_of(wline, iv, on_first)?.as_ref() == Some(arc) {
            return Ok(iv);
        }
    }
    Ok(0)
}

/// The parameter on the arc of the vertex `iv`: its own, or the resolved one.
fn param_on_arc(
    wline: &IntPatchWLine,
    (resolved, on_first): (&Resolved, bool),
    iv: i32,
) -> Result<f64, Failure> {
    if !resolved.is_resolved(iv) {
        let vertex: IntPatchPoint = wline.vertex(iv)?;
        return if on_first {
            vertex.parameter_on_arc1()
        } else {
            vertex.parameter_on_arc2()
        };
    }
    Ok(resolved.params[iv as usize])
}

/// The restriction line built from the walking line on one arc.
struct RLineCoding<'a> {
    wline: &'a IntPatchWLine,
    resolved: &'a Resolved,
    on_first: bool,
    arc: TheArc,
    reverse: bool,
}

impl RLineCoding<'_> {
    /// The vertex `iv` at the parameter `par` on the arc (linked with the arc when it
    /// was resolved), with its transitions reversed for a reversed line.
    fn vertex(&self, iv: i32, par: f64) -> Result<IntPatchPoint, Failure> {
        let mut vtx: IntPatchPoint = self.wline.vertex(iv)?;
        vtx.set_parameter(par);
        if self.resolved.is_resolved(iv) {
            let undecided = IntSurfTransition::default();
            vtx.set_arc(self.on_first, &self.arc, par, &undecided, &undecided);
        }
        if self.reverse {
            // Inversion des transitions.
            vtx.reverse_transition();
        }
        Ok(vtx)
    }

    /// The vertices of `indices` other than `skip`, at `par`.
    fn add_others(
        &self,
        rlig: &IntPatchRLine,
        indices: &[i32],
        skip: i32,
        par: f64,
    ) -> Result<(), Failure> {
        for &iv in indices.iter().filter(|&&iv| iv != skip) {
            rlig.add_vertex(&self.vertex(iv, par)?, false);
        }
        Ok(())
    }
}

/// The points of the walking line between its end vertices, reversed when `reverse`.
fn line_points(
    wline: &IntPatchWLine,
    ends: &LineEnds,
    reverse: bool,
) -> Result<Handle<IntSurfLineOn2S>, Failure> {
    let points: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    let range: Vec<i32> = match reverse {
        false => (ends.param_min..=ends.param_max).collect(),
        true => (ends.param_min..=ends.param_max).rev().collect(),
    };
    for i in range {
        points.add(&wline.point(i)?);
    }
    Ok(points)
}

/// Codes the walking line as a restriction line on `arc` (from the vertex `iv1` of the
/// first end to `iv2` of the last end), when its middle point is on the arc.
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn code_on_arc(
    wline: &IntPatchWLine,
    (ends, resolved): (&LineEnds, &Resolved),
    (domains, tol_arc, on_first): (&Domains<'_>, f64, bool),
    (arc, iv1, iv2): (&TheArc, i32, i32),
) -> Result<Option<IntPatchRLine>, Failure> {
    let par1: f64 = param_on_arc(wline, (resolved, on_first), iv1)?;
    let par2: f64 = param_on_arc(wline, (resolved, on_first), iv2)?;
    // Check that the middle point is on the arc.
    let mid_ind: i32 = (ends.param_max + ends.param_min) / 2;
    let (mut utst, mut vtst) = params_on(&wline.point(mid_ind)?, on_first);
    if mid_ind == ends.param_min {
        let (utst1, vtst1) = params_on(&wline.point(mid_ind + 1)?, on_first);
        utst = (utst + utst1) * 0.5;
        vtst = (vtst + vtst1) * 0.5;
    }
    if !project_on_arc((utst, vtst), arc, domains.side(on_first).0, tol_arc)?.0 {
        return Ok(None);
    }
    let rlig = IntPatchRLine::new_touch(true, IntSurfSituation::Unknown, IntSurfSituation::Unknown);
    match on_first {
        true => rlig.set_arc_on_s1(arc),
        false => rlig.set_arc_on_s2(arc),
    }
    let reverse: bool = !(par1 < par2);
    rlig.add(&line_points(wline, ends, reverse)?);
    let (ivmin, ivmax, parmin, parmax) = if reverse {
        (iv2, iv1, par2, par1)
    } else {
        (iv1, iv2, par1, par2)
    };
    let (ind_vmin, ind_vmax) = if reverse {
        (&ends.indices_v2, &ends.indices_v1)
    } else {
        (&ends.indices_v1, &ends.indices_v2)
    };
    let coding = RLineCoding {
        wline,
        resolved,
        on_first,
        arc: arc.clone(),
        reverse,
    };
    rlig.add_vertex(&coding.vertex(ivmin, parmin)?, false);
    coding.add_others(&rlig, ind_vmin, ivmin, parmin)?;
    coding.add_others(&rlig, ind_vmax, ivmax, parmax)?;
    rlig.add_vertex(&coding.vertex(ivmax, parmax)?, false);
    rlig.set_first_point(1);
    rlig.set_last_point((ends.indices_v1.len() + ends.indices_v2.len()) as i32);
    Ok(Some(rlig))
}

/// The restriction lines of one side of the walking line: one for each arc common to
/// both ends whose middle point is on it.
fn code_side(
    wline: &IntPatchWLine,
    ends: &LineEnds,
    (domains, tol_arc, on_first): (&Domains<'_>, f64, bool),
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
) -> Result<bool, Failure> {
    // Resolve the arcs of the vertices that have no link to an arc.
    let resolved = Resolved::new(wline, domains, on_first, tol_arc)?;
    let mut is_rline = false;
    for arc in common_arcs(wline, ends, &resolved, on_first)? {
        // The end vertices of the line linked with this arc.
        let iv1: i32 = vertex_on_arc(wline, (&resolved, on_first), &ends.indices_v1, &arc)?;
        let iv2: i32 = vertex_on_arc(wline, (&resolved, on_first), &ends.indices_v2, &arc)?;
        if iv1 == 0 || iv2 == 0 {
            continue;
        }
        let side = (domains, tol_arc, on_first);
        if let Some(rlig) = code_on_arc(wline, (ends, &resolved), side, (&arc, iv1, iv2))? {
            slin.push(Handle::new(rlig).cast());
            is_rline = true;
        }
    }
    Ok(is_rline)
}

/// `TestWLineToRLine`: the last line of `slin` (a walking line) is coded as restriction
/// lines on the arcs it follows, or removed when it lies on the boundary of a domain
/// along a restriction line of `slinref`.
pub(super) fn test_wline_to_rline(
    slinref: &[Handle<dyn IntPatchLine>],
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    domains: &Domains<'_>,
    tol_arc: f64,
) -> Result<(), Failure> {
    let lastwline: usize = slin.len();
    let wline: Handle<IntPatchWLine> = slin[lastwline - 1].cast::<IntPatchWLine>();
    let Some(ends) = LineEnds::new(&wline)? else {
        return Ok(());
    };
    let mut is_rline = false;
    for on_first in [true, false] {
        let has_arc: bool = if on_first {
            wline.has_arc_on_s1()
        } else {
            wline.has_arc_on_s2()
        };
        if has_arc {
            is_rline |= code_side(&wline, &ends, (domains, tol_arc, on_first), slin)?;
        }
    }
    if is_rline || test_if_wline_is_restriction(slinref, &wline, domains, tol_arc)? {
        slin.remove(lastwline - 1);
    }
    Ok(())
}
