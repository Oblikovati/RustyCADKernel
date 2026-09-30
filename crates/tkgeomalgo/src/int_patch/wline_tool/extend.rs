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
// Rust translation of the static helpers ExtendFirst, ExtendLast and ExtendTwoWLFirstFirst,
// ExtendTwoWLFirstLast, ExtendTwoWLLastFirst, ExtendTwoWLLastLast of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extension of two walking lines to a common point and their joining there. The
//! native code keeps references to the end points of the lines while it inserts points
//! before them (docs/geomalgo-design.md section 18): [`PointRefs`] keeps the ranks of
//! those points and shifts them with each insertion at the front of their line.
use super::extend_checks::{ConnectionType, ExtendInputs, check_arguments_to_extend};
use crate::int_patch::{IntPatchPoint, IntPatchPointLine, IntPatchSpecialPoints, IntPatchWLine};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Vector3;

/// `IntPatchWT_DisLastLast`...: the extensions disabled for a pair of lines.
pub(super) const DIS_LAST_LAST: u32 = 0x01;
pub(super) const DIS_LAST_FIRST: u32 = 0x02;
pub(super) const DIS_FIRST_LAST: u32 = 0x04;
pub(super) const DIS_FIRST_FIRST: u32 = 0x08;

/// References to points of the lines, as the native `const IntSurf_PntOn2S&` to the
/// items of their sequences: each one is the rank of the point in its line, shifted by
/// the insertions at the front of the line.
#[derive(Default)]
pub(super) struct PointRefs {
    refs: Vec<(Handle<IntSurfLineOn2S>, i32)>,
}

impl PointRefs {
    /// A reference to the point `index` of the line (read at once, as the native
    /// reference binding reads the sequence).
    pub(super) fn add(&mut self, wline: &IntPatchWLine, index: i32) -> Result<usize, Failure> {
        wline.point(index)?;
        self.refs.push((wline.curve(), index));
        Ok(self.refs.len() - 1)
    }

    /// The point the reference `id` reads now.
    pub(super) fn get(&self, id: usize) -> Result<IntSurfPntOn2S, Failure> {
        let (curve, index) = &self.refs[id];
        curve.value(*index)
    }

    /// The references to `curve` follow `n` insertions at its front.
    fn shift_front(&mut self, curve: &Handle<IntSurfLineOn2S>, n: i32) {
        for (c, index) in &mut self.refs {
            if c == curve {
                *index += n;
            }
        }
    }

    /// `curve->InsertBefore(1, p)`.
    fn insert_front(
        &mut self,
        curve: &Handle<IntSurfLineOn2S>,
        p: &IntSurfPntOn2S,
    ) -> Result<(), Failure> {
        curve.insert_before(1, p)?;
        self.shift_front(curve, 1);
        Ok(())
    }
}

/// The end vertices at the parameter `on_line` of the line get the point `added`.
fn move_vertex(vertex: &mut IntPatchPoint, added: &IntSurfPntOn2S) {
    let (u1, v1, u2, v2) = added.parameters();
    vertex.set_parameters(u1, v1, u2, v2);
    vertex.set_value(added.value());
}

/// `ExtendFirst`: `added` becomes the first point of the line (replacing it when they
/// are the same point), with the first vertices.
fn extend_first(
    wline: &IntPatchWLine,
    added: &IntSurfPntOn2S,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let curve: Handle<IntSurfLineOn2S> = wline.curve();
    if added.is_same(&wline.point(1)?, precision::confusion(), -1.0) {
        curve.set_value(1, added)?;
        for i in 1..=wline.nb_vertex() {
            if wline.vertex(i)?.parameter_on_line() != 1.0 {
                break;
            }
            wline.change_vertex(i, &mut |v| move_vertex(v, added))?;
        }
        return Ok(());
    }
    refs.insert_front(&curve, added)?;
    for i in 1..=wline.nb_vertex() {
        wline.change_vertex(i, &mut |v| {
            if v.parameter_on_line() == 1.0 {
                move_vertex(v, added);
            } else {
                v.set_parameter(v.parameter_on_line() + 1.0);
            }
        })?;
    }
    Ok(())
}

/// `ExtendLast`: `added` becomes the last point of the line (replacing it when they are
/// the same point), with the last vertices.
fn extend_last(wline: &IntPatchWLine, added: &IntSurfPntOn2S) -> Result<(), Failure> {
    let nb_pnts: i32 = wline.nb_pnts()?;
    if added.is_same(&wline.point(nb_pnts)?, precision::confusion(), -1.0) {
        wline.curve().set_value(nb_pnts, added)?;
    } else {
        wline.curve().add(added);
    }
    let new_nb: f64 = f64::from(wline.nb_pnts()?);
    for i in (1..=wline.nb_vertex()).rev() {
        if wline.vertex(i)?.parameter_on_line() != f64::from(nb_pnts) {
            break;
        }
        wline.change_vertex(i, &mut |v| {
            move_vertex(v, added);
            v.set_parameter(new_nb);
        })?;
    }
    Ok(())
}

/// Removes the vertices at the parameter of the first (`first`) or last vertex.
fn remove_end_vertices(wline: &IntPatchWLine, first: bool) -> Result<(), Failure> {
    let rank = |w: &IntPatchWLine| if first { 1 } else { w.nb_vertex() };
    let prm: f64 = wline.vertex(rank(wline))?.parameter_on_line();
    while wline.vertex(rank(wline))?.parameter_on_line() == prm {
        wline.remove_vertex(rank(wline))?;
    }
    Ok(())
}

/// Which ends of the two lines are extended.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EndPair {
    FirstFirst,
    FirstLast,
    LastFirst,
    LastLast,
}

impl EndPair {
    /// Whether the extension is at the first point of each line.
    fn at_first(self) -> (bool, bool) {
        match self {
            EndPair::FirstFirst => (true, true),
            EndPair::FirstLast => (true, false),
            EndPair::LastFirst => (false, true),
            EndPair::LastLast => (false, false),
        }
    }

    /// The extensions disabled once these ends connect.
    fn disabled(self) -> u32 {
        match self {
            EndPair::FirstFirst => DIS_FIRST_LAST | DIS_LAST_FIRST,
            _ => DIS_LAST_LAST,
        }
    }
}

/// The two lines, their end points (references) and the state of the pair.
pub(super) struct PairState<'a> {
    pub(super) wline1: &'a IntPatchWLine,
    pub(super) wline2: &'a IntPatchWLine,
    pub(super) check_result: &'a mut u32,
    pub(super) has_been_joined: &'a mut bool,
}

/// The points of `wline2` from `from` to `to` (inclusive, in that order) added to the
/// front (`front`) or to the end of `wline1`.
fn move_points(
    (wline1, wline2): (&IntPatchWLine, &IntPatchWLine),
    ranks: Vec<i32>,
    front: bool,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let curve1: Handle<IntSurfLineOn2S> = wline1.curve();
    for n in ranks {
        let pt: IntSurfPntOn2S = wline2.point(n)?;
        match front {
            true => refs.insert_front(&curve1, &pt)?,
            false => curve1.add(&pt),
        }
    }
    Ok(())
}

/// The joining of `wline2` into `wline1` once both are extended to their common point.
fn join_extended(pair: &PairState<'_>, ends: EndPair, refs: &mut PointRefs) -> Result<(), Failure> {
    let (w1, w2) = (pair.wline1, pair.wline2);
    let (first1, first2) = ends.at_first();
    remove_end_vertices(w1, first1)?;
    remove_end_vertices(w2, first2)?;
    match ends {
        EndPair::FirstFirst => join_first_first(w1, w2, refs),
        EndPair::FirstLast => join_first_last(w1, w2, refs),
        EndPair::LastFirst => join_last_first(w1, w2, refs),
        EndPair::LastLast => join_last_last(w1, w2, refs),
    }
}

fn join_first_first(
    w1: &IntPatchWLine,
    w2: &IntPatchWLine,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let nb_pts: i32 = w2.nb_pnts()?;
    move_points((w1, w2), (2..=nb_pts).collect(), true, refs)?;
    for i in 1..=w1.nb_vertex() {
        w1.change_vertex(i, &mut |v| {
            v.set_parameter(f64::from(nb_pts) + v.parameter_on_line() - 1.0)
        })?;
    }
    for i in 1..=w2.nb_vertex() {
        w2.change_vertex(i, &mut |v| {
            v.set_parameter(f64::from(nb_pts) - v.parameter_on_line() + 1.0)
        })?;
        w1.add_vertex(&w2.vertex(i)?, true);
    }
    Ok(())
}

fn join_first_last(
    w1: &IntPatchWLine,
    w2: &IntPatchWLine,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let nb_pts: i32 = w2.nb_pnts()?;
    move_points((w1, w2), (1..nb_pts).rev().collect(), true, refs)?;
    for i in 1..=w1.nb_vertex() {
        w1.change_vertex(i, &mut |v| {
            v.set_parameter(f64::from(nb_pts) + v.parameter_on_line() - 1.0)
        })?;
    }
    for i in (1..=w2.nb_vertex()).rev() {
        w1.add_vertex(&w2.vertex(i)?, true);
    }
    Ok(())
}

fn join_last_first(
    w1: &IntPatchWLine,
    w2: &IntPatchWLine,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let nb_pts: i32 = w1.nb_pnts()?;
    // The count of the second line is read at each step, as natively.
    let mut n: i32 = 2;
    while n <= w2.nb_pnts()? {
        move_points((w1, w2), vec![n], false, refs)?;
        n += 1;
    }
    for i in 1..=w2.nb_vertex() {
        w2.change_vertex(i, &mut |v| {
            v.set_parameter(f64::from(nb_pts) + v.parameter_on_line() - 1.0)
        })?;
        w1.add_vertex(&w2.vertex(i)?, false);
    }
    Ok(())
}

fn join_last_last(
    w1: &IntPatchWLine,
    w2: &IntPatchWLine,
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let nb_pts: i32 = w1.nb_pnts()? + w2.nb_pnts()?;
    move_points((w1, w2), (1..w2.nb_pnts()?).rev().collect(), false, refs)?;
    for i in (1..=w2.nb_vertex()).rev() {
        w2.change_vertex(i, &mut |v| {
            v.set_parameter(f64::from(nb_pts) - v.parameter_on_line())
        })?;
        w1.add_vertex(&w2.vertex(i)?, false);
    }
    Ok(())
}

/// `ExtendTwoWLFirstFirst`, `ExtendTwoWLFirstLast`, `ExtendTwoWLLastFirst` and
/// `ExtendTwoWLLastLast`: the lines are extended through the given ends to the middle
/// of their end points when the check allows it, then joined unless they already were
/// or meet on a boundary.
pub(super) fn extend_two_wl(
    inputs: &ExtendInputs<'_>,
    pair: &mut PairState<'_>,
    ends: EndPair,
    (pt_wl1, pt_wl2): (usize, usize),
    vecs: [Vector3; 3],
    refs: &mut PointRefs,
) -> Result<(), Failure> {
    let mut pon2s = IntSurfPntOn2S::new();
    let (p1, p2) = (refs.get(pt_wl1)?, refs.get(pt_wl2)?);
    let check_res: ConnectionType =
        check_arguments_to_extend(inputs, (&p1, &p2), &mut pon2s, vecs)?;
    if check_res == ConnectionType::NotConnected {
        return Ok(());
    }
    *pair.check_result |= ends.disabled();
    let (first1, first2) = ends.at_first();
    IntPatchSpecialPoints::adjust_point_and_vertex(
        &refs.get(pt_wl1)?,
        &inputs.periods,
        &mut pon2s,
        None,
    );
    match first1 {
        true => extend_first(pair.wline1, &pon2s, refs)?,
        false => extend_last(pair.wline1, &pon2s)?,
    }
    IntPatchSpecialPoints::adjust_point_and_vertex(
        &refs.get(pt_wl2)?,
        &inputs.periods,
        &mut pon2s,
        None,
    );
    match first2 {
        true => extend_first(pair.wline2, &pon2s, refs)?,
        false => extend_last(pair.wline2, &pon2s)?,
    }
    if *pair.has_been_joined || check_res == ConnectionType::Singular {
        return Ok(());
    }
    join_extended(pair, ends, refs)?;
    *pair.has_been_joined = true;
    Ok(())
}
