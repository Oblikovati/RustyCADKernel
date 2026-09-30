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
// Rust translation of IntPatch_WLineTool::JoinWLines and its static helpers IsSeamOrBound
// and CheckArgumentsToJoin (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The joining of the walking lines that share an end point (`JoinWLines`).
use super::IntPatchWLineTool;
use crate::int_patch::{
    IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchWLine, curvature_radius_of_inters_line,
};
use crate::int_start::failures::sequence_value;
use crate::int_surf::IntSurfPntOn2S;
use crate::intf::std_min;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::bnd::{IntersectStatus, Range1};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The periods and the first and last bounds of the parameters of both surfaces, in the
/// order `U1, V1, U2, V2`.
pub(super) struct SurfaceBounds {
    pub(super) periods: [f64; 4],
    first: [f64; 4],
    last: [f64; 4],
}

impl SurfaceBounds {
    /// The periods (0 on a non periodic direction) and the bounds of the surfaces.
    pub(super) fn new(
        s1: &Adaptor3dSurfaceHandle,
        s2: &Adaptor3dSurfaceHandle,
    ) -> Result<Self, Failure> {
        let period = |s: &Adaptor3dSurfaceHandle, u: bool| -> Result<f64, Failure> {
            match u {
                true if s.is_u_periodic()? => s.u_period(),
                false if s.is_v_periodic()? => s.v_period(),
                _ => Ok(0.0),
            }
        };
        Ok(Self {
            periods: [
                period(s1, true)?,
                period(s1, false)?,
                period(s2, true)?,
                period(s2, false)?,
            ],
            first: [
                s1.first_u_parameter()?,
                s1.first_v_parameter()?,
                s2.first_u_parameter()?,
                s2.first_v_parameter()?,
            ],
            last: [
                s1.last_u_parameter()?,
                s1.last_v_parameter()?,
                s2.last_u_parameter()?,
                s2.last_v_parameter()?,
            ],
        })
    }
}

fn params_of(p: &IntSurfPntOn2S) -> [f64; 4] {
    let (a, b, c, d) = p.parameters();
    [a, b, c, d]
}

/// Whether the segment of `ptf` and `ptl` crosses a bound or a seam in one parameter.
fn segment_crosses(ptf: &IntSurfPntOn2S, ptl: &IntSurfPntOn2S, bounds: &SurfaceBounds) -> bool {
    let (par_f, par_l) = (params_of(ptf), params_of(ptl));
    let mut ranges = [Range1::default(); 4];
    for i in 0..4 {
        ranges[i].add(par_f[i]);
        ranges[i].add(par_l[i]);
        let period: f64 = bounds.periods[i];
        if ranges[i].is_intersected(bounds.first[i], period) == IntersectStatus::In
            || ranges[i].is_intersected(bounds.last[i], period) == IntersectStatus::In
        {
            return true;
        }
    }
    for i in 0..4 {
        if bounds.periods[i] == 0.0 {
            // Strictly equal.
            continue;
        }
        let delta: f64 = (par_l[i] - par_f[i]).abs();
        if 2.0 * delta > bounds.periods[i] {
            // Most likely, seam is intersected.
            return true;
        }
        if ranges[i].is_intersected(0.0, bounds.periods[i]) == IntersectStatus::In {
            return true;
        }
    }
    false
}

/// `IsSeamOrBound`: whether the segment `[ptf, ptl]` crosses the seam or a bound of the
/// surfaces, or its middle point `ptmid` lies strictly on one of them.
fn is_seam_or_bound(
    ptf: &IntSurfPntOn2S,
    ptl: &IntSurfPntOn2S,
    ptmid: &IntSurfPntOn2S,
    bounds: &SurfaceBounds,
) -> Result<bool, Failure> {
    if segment_crosses(ptf, ptl, bounds) {
        return Ok(true);
    }
    // The segment [thePtf, thePtl] does not intersect the boundaries and the seam-edge of
    // the surfaces. Nevertheless, thePtmid may lie on a seam or a bound between them:
    // this case must be processed, too.
    let mid: [f64; 4] = params_of(ptmid);
    for (i, &m) in mid.iter().enumerate() {
        let range: Range1 = Range1::new(m, m)?;
        let period: f64 = bounds.periods[i];
        if range.is_intersected(bounds.first[i], period) != IntersectStatus::Out
            || range.is_intersected(bounds.last[i], period) != IntersectStatus::Out
            || range.is_intersected(0.0, period) != IntersectStatus::Out
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `CheckArgumentsToJoin`: joining is enabled when the curvature radius of the
/// intersection line at `pnt` is greater than `min_rad`, or, when it cannot be
/// computed, when the polygon `p1, p2, p3` is smooth.
fn check_arguments_to_join(
    (s1, s2): (&Adaptor3dSurfaceHandle, &Adaptor3dSurfaceHandle),
    pnt: &IntSurfPntOn2S,
    [p1, p2, p3]: [Point3; 3],
    min_rad: f64,
) -> Result<bool, Failure> {
    let rad: f64 = curvature_radius_of_inters_line(s1, s2, pnt)?;
    if rad > min_rad {
        return Ok(true);
    } else if rad > 0.0 {
        return Ok(false);
    }
    // Curvature radius cannot be computed: joining is enabled if the angle (p1, p2, p3) is
    // big enough and the distance of p2 to the segment (p1, p3) is less than 0.01 of the
    // length of this segment.
    let (v12f, v12l) = (Vector3::between(p1, p2), Vector3::between(p2, p3));
    if v12f.angle(v12l)? > IntPatchWLineTool::MY_MAX_CONCAT_ANGLE {
        return Ok(false);
    }
    let v13: Vector3 = Vector3::between(p1, p3);
    let sq13: f64 = v13.square_magnitude();
    Ok(v12f.cross_square_magnitude(v13) < 1.0e-4 * sq13 * sq13)
}

/// The inputs of `JoinWLines`.
pub(super) struct JoinInputs<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) min_rad: f64,
    pub(super) bounds: SurfaceBounds,
}

/// The line of rank `index` as a walking line (null for another kind).
pub(super) fn wline_at(
    slin: &[Handle<dyn IntPatchLine>],
    index: i32,
) -> Result<Handle<IntPatchWLine>, Failure> {
    Ok(sequence_value(slin, index, "NCollection_Sequence::Value")?.cast::<IntPatchWLine>())
}

/// Removes the points of `spnt` that match an end of `wline1`.
fn remove_end_points(
    spnt: &mut Vec<IntPatchPoint>,
    (first, last): (&IntSurfPntOn2S, &IntSurfPntOn2S),
) {
    spnt.retain(|p| {
        let cur: IntSurfPntOn2S = p.pnt_on_2s();
        !(cur.is_same(first, precision::confusion(), -1.0)
            || cur.is_same(last, precision::confusion(), -1.0))
    });
}

/// One end of the line `wline1`: its end point, and the rank of the next point inside
/// the line (read only when needed, as natively).
struct LineEnd<'a> {
    wline: &'a IntPatchWLine,
    end: IntSurfPntOn2S,
    next_index: i32,
}

impl LineEnd<'_> {
    /// The neighbour of the end point.
    fn next(&self) -> Result<IntSurfPntOn2S, Failure> {
        self.wline.point(self.next_index)
    }

    /// The square distances of the end point to the first and last points of `wline2`.
    fn distances(&self, wline2: &IntPatchWLine) -> Result<(f64, f64), Failure> {
        let nb2: i32 = wline2.nb_pnts()?;
        let sq_dist_f: f64 = self.end.value().square_distance(&wline2.point(1)?.value());
        let sq_dist_l: f64 = self
            .end
            .value()
            .square_distance(&wline2.point(nb2)?.value());
        Ok((sq_dist_f, sq_dist_l))
    }
}

/// The neighbour in `wline2` of its end nearest to the end point: its second point when
/// its first point is nearer, else its last but one point.
fn near_neighbour(wline2: &IntPatchWLine, is_fm: bool) -> Result<IntSurfPntOn2S, Failure> {
    wline2.point(if is_fm { 2 } else { wline2.nb_pnts()? - 1 })
}

/// Whether the end `end` of `wline1` meets an end of `wline2` without crossing a seam or
/// a bound; `(connected, square distance)`.
fn end_connection(
    inputs: &JoinInputs<'_>,
    end: &LineEnd<'_>,
    wline2: &IntPatchWLine,
) -> Result<(bool, f64), Failure> {
    let (sq_dist_f, sq_dist_l) = end.distances(wline2)?;
    let sq_min: f64 = std_min(sq_dist_f, sq_dist_l);
    if sq_min >= precision::square_confusion() {
        return Ok((false, sq_min));
    }
    let pt1: IntSurfPntOn2S = end.next()?;
    let pt2: IntSurfPntOn2S = near_neighbour(wline2, sq_dist_f < sq_dist_l)?;
    Ok((
        !is_seam_or_bound(&pt1, &pt2, &end.end, &inputs.bounds)?,
        sq_min,
    ))
}

/// The lines connected to the first end (`aListFC`) and to the last end (`aListLC`) of
/// the line `n1`.
fn connected_lines(
    inputs: &JoinInputs<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    n1: i32,
    (first, last): (&LineEnd<'_>, &LineEnd<'_>),
) -> Result<(Vec<i32>, Vec<i32>), Failure> {
    let (mut list_fc, mut list_lc) = (Vec::new(), Vec::new());
    for n2 in 1..=slin.len() as i32 {
        if n2 == n1 {
            continue;
        }
        let wline2: Handle<IntPatchWLine> = wline_at(slin, n2)?;
        if wline2.is_null() {
            continue;
        }
        let (first_connected, sq_min_f) = end_connection(inputs, first, &wline2)?;
        let (last_connected, sq_min_l) = end_connection(inputs, last, &wline2)?;
        match (first_connected, last_connected) {
            (true, true) if sq_min_f < sq_min_l => list_fc.push(n2),
            (true, true) => list_lc.push(n2),
            (true, false) => list_fc.push(n2),
            (false, true) => list_lc.push(n2),
            (false, false) => {}
        }
    }
    Ok((list_fc, list_lc))
}

/// Joins `wline2` to the end `end` of `wline1` when the curvature check allows it;
/// returns whether the lines have been joined.
fn join_at_end(
    inputs: &JoinInputs<'_>,
    (wline1, wline2): (&IntPatchWLine, &IntPatchWLine),
    at_first: bool,
    end: &LineEnd<'_>,
) -> Result<bool, Failure> {
    let (sq_dist_f, sq_dist_l) = end.distances(wline2)?;
    let is_fm: bool = sq_dist_f < sq_dist_l;
    let pt1: IntSurfPntOn2S = end.next()?;
    let pt2: IntSurfPntOn2S = near_neighbour(wline2, is_fm)?;
    let polygon: [Point3; 3] = [pt1.value(), end.end.value(), pt2.value()];
    if !check_arguments_to_join((inputs.s1, inputs.s2), &end.end, polygon, inputs.min_rad)? {
        return Ok(false);
    }
    wline1.clear_vertexes();
    let curve = wline1.curve();
    let nb2: i32 = wline2.nb_pnts()?;
    // First-First and Last-First connections take the points of wline2 from its first
    // point, the two others from its last point.
    let order: Vec<i32> = match is_fm {
        true => (1..=nb2).collect(),
        false => (1..=nb2).rev().collect(),
    };
    for n in order {
        let pt: IntSurfPntOn2S = wline2.point(n)?;
        match at_first {
            true => curve.insert_before(1, &pt)?,
            false => curve.add(&pt),
        }
    }
    Ok(true)
}

/// `JoinWLines` for one line `n1`: the line and the index of the line joined to it, if
/// any.
fn join_line(
    inputs: &JoinInputs<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    spnt: &mut Vec<IntPatchPoint>,
    n1: i32,
) -> Result<Option<(Handle<IntPatchWLine>, i32)>, Failure> {
    let wline1: Handle<IntPatchWLine> = wline_at(slin, n1)?;
    if wline1.is_null() {
        // We must have failed to join not-point-lines.
        return Ok(None);
    }
    let nb1: i32 = wline1.nb_pnts()?;
    let first = LineEnd {
        wline: &wline1,
        end: wline1.point(1)?,
        next_index: 2,
    };
    let last = LineEnd {
        wline: &wline1,
        end: wline1.point(nb1)?,
        next_index: nb1 - 1,
    };
    remove_end_points(spnt, (&first.end, &last.end));
    let (list_fc, list_lc) = connected_lines(inputs, slin, n1, (&first, &last))?;
    let (first_connected, last_connected) = (list_fc.len() == 1, list_lc.len() == 1);
    if !(first_connected || last_connected) {
        return Ok(None);
    }
    let index_wl2: i32 = if first_connected {
        list_fc[0]
    } else {
        list_lc[0]
    };
    let wline2: Handle<IntPatchWLine> = wline_at(slin, index_wl2)?;
    let end: &LineEnd<'_> = if first_connected { &first } else { &last };
    if !join_at_end(inputs, (&wline1, &wline2), first_connected, end)? {
        return Ok(None);
    }
    Ok(Some((wline1.clone(), index_wl2)))
}

/// `IntPatch_WLineTool::JoinWLines` (see [`IntPatchWLineTool::join_wlines`]).
pub(super) fn join_wlines(
    slin: &mut Vec<Handle<dyn IntPatchLine>>,
    spnt: &mut Vec<IntPatchPoint>,
    inputs: &JoinInputs<'_>,
    tol3d: f64,
) -> Result<(), Failure> {
    let mut n1: i32 = 1;
    while n1 <= slin.len() as i32 {
        if let Some((wline1, index_wl2)) = join_line(inputs, slin, spnt, n1)? {
            wline1.compute_vertex_parameters(tol3d)?;
            slin.remove(usize::try_from(index_wl2 - 1).unwrap_or_default());
            n1 -= 1;
        }
        n1 += 1;
    }
    Ok(())
}
