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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLine.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_WLine`: the intersection line of two parametrised patches given by the
//! points of a walking. The vertex parameters are computed in `compute.rs`, split by
//! the phases of the native `ComputeVertexParameters`.
mod compute;
mod compute_ends;
mod compute_filters;
mod compute_points;

use super::IntPatchIType;
use super::failures::null_object;
use super::line::{IntPatchLine, IntPatchLineBase, IntPatchLineTransitions};
use super::point::IntPatchPoint;
use super::point_line::IntPatchPointLine;
use super::vertices::VertexList;
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfSituation, IntSurfTypeTrans};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point2, Point3};

/// `IntPatch_WLine::IntPatch_WLType`: the algorithm that built a walking line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum IntPatchWLType {
    /// `IntPatch_WLUnknown`.
    #[default]
    Unknown,
    /// `IntPatch_WLImpImp`: two implicit surfaces.
    ImpImp,
    /// `IntPatch_WLImpPrm`: an implicit and a parametric surface.
    ImpPrm,
    /// `IntPatch_WLPrmPrm`: two parametric surfaces.
    PrmPrm,
}

/// The fields of `IntPatch_WLine`.
pub(super) struct WLineState {
    pub(super) curv: Handle<IntSurfLineOn2S>,
    pub(super) vertices: VertexList,
    /// `u1period`, `v1period`, `u2period`, `v2period`.
    pub(super) periods: [f64; 4],
    arc_on_s1: Option<TheArc>,
    arc_on_s2: Option<TheArc>,
    purger_allowed: bool,
    pub(super) creation_way: IntPatchWLType,
}

/// `IntPatch_WLine`: a walking line (its points on both surfaces), with its vertices,
/// its periods and the arcs it follows.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::int_patch::{IntPatchPointLine, IntPatchWLine};
/// use rustycad_tkgeomalgo::int_surf::IntSurfLineOn2S;
/// let line = IntPatchWLine::new_undecided(&Handle::new(IntSurfLineOn2S::new(None)), false);
/// assert_eq!(line.nb_pnts().unwrap(), 0);
/// assert!(line.first_point().is_err());
/// ```
pub struct IntPatchWLine {
    transient: StandardTransient,
    base: IntPatchLineBase,
    state: SharedState<WLineState>,
}
impl_transient!(IntPatchWLine, "IntPatch_WLine", dyn IntPatchPointLine, transient,
    interfaces: [dyn IntPatchLine, dyn IntPatchPointLine]);

const CLASS: &str = "IntPatch_WLine";

impl IntPatchWLine {
    fn with_transitions(
        line: &Handle<IntSurfLineOn2S>,
        tang: bool,
        transitions: IntPatchLineTransitions,
    ) -> Self {
        let state = WLineState {
            curv: line.clone(),
            vertices: VertexList::default(),
            periods: [0.0; 4],
            arc_on_s1: None,
            arc_on_s2: None,
            purger_allowed: true,
            creation_way: IntPatchWLType::Unknown,
        };
        Self {
            transient: StandardTransient::default(),
            base: IntPatchLineBase::new(tang, transitions, IntPatchIType::Walking),
            state: SharedState::new(state),
        }
    }

    /// `IntPatch_WLine(Line, Tang, Trans1, Trans2)`: IN or OUT transitions.
    /// `IntPatchWLine::new(&points, false, IntSurfTypeTrans::In, IntSurfTypeTrans::Out)`.
    pub fn new(
        line: &Handle<IntSurfLineOn2S>,
        tang: bool,
        trans1: IntSurfTypeTrans,
        trans2: IntSurfTypeTrans,
    ) -> Self {
        Self::with_transitions(line, tang, IntPatchLineTransitions::Types(trans1, trans2))
    }

    /// `IntPatch_WLine(Line, Tang, Situ1, Situ2)`: TOUCH transitions.
    /// `IntPatchWLine::new_touch(&points, true, IntSurfSituation::Inside, IntSurfSituation::Outside)`.
    pub fn new_touch(
        line: &Handle<IntSurfLineOn2S>,
        tang: bool,
        situ1: IntSurfSituation,
        situ2: IntSurfSituation,
    ) -> Self {
        Self::with_transitions(line, tang, IntPatchLineTransitions::Touch(situ1, situ2))
    }

    /// `IntPatch_WLine(Line, Tang)`: undecided transitions.
    /// `IntPatchWLine::new_undecided(&points, false)`.
    pub fn new_undecided(line: &Handle<IntSurfLineOn2S>, tang: bool) -> Self {
        Self::with_transitions(line, tang, IntPatchLineTransitions::Undecided)
    }

    /// The points, or the Standard_NullObject of a line built on a null handle (the
    /// native dereference is undefined behaviour).
    fn points(&self, member: &str) -> Result<Handle<IntSurfLineOn2S>, Failure> {
        let curv: Handle<IntSurfLineOn2S> = self.state.read().curv.clone();
        if curv.is_null() {
            return Err(null_object(
                &format!("{CLASS}::{member}"),
                "IntSurf_LineOn2S",
            ));
        }
        Ok(curv)
    }

    /// `SetPoint(Index, Pnt)`: the point of range `index` of the line becomes the
    /// point and parameters of `pnt`. `line.set_point(2, &vertex)?`.
    pub fn set_point(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        self.points("SetPoint")?.set_value(index, &pnt.pnt_on_2s())
    }

    /// `Replace(Index, Pnt)`: Standard_OutOfRange outside `1..=NbVertex()`.
    /// `line.replace(1, &vertex)?`.
    pub fn replace(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        let member: String = format!("{CLASS}::Replace");
        self.state.write().vertices.replace(index, pnt, &member)
    }

    /// `SetFirstPoint(IndFirst)`. `line.set_first_point(1)`.
    pub fn set_first_point(&self, ind_first: i32) {
        self.state.write().vertices.set_first_point(ind_first);
    }

    /// `SetLastPoint(IndLast)`. `line.set_last_point(2)`.
    pub fn set_last_point(&self, ind_last: i32) {
        self.state.write().vertices.set_last_point(ind_last);
    }

    /// `HasFirstPoint()`. `line.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.state.read().vertices.fipt
    }

    /// `HasLastPoint()`. `line.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.state.read().vertices.lapt
    }

    /// `FirstPoint()`: Standard_DomainError without a first point. `line.first_point()?`.
    pub fn first_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.first_point(CLASS).cloned()
    }

    /// `LastPoint()`: Standard_DomainError without a last point. `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.last_point(CLASS).cloned()
    }

    /// `FirstPoint(Indfirst)`: the first point and its index. `line.first_point_index()?`.
    pub fn first_point_index(&self) -> Result<(IntPatchPoint, i32), Failure> {
        let state = self.state.read();
        let point: IntPatchPoint = state.vertices.first_point(CLASS)?.clone();
        Ok((point, state.vertices.indf))
    }

    /// `LastPoint(Indlast)`: the last point and its index. `line.last_point_index()?`.
    pub fn last_point_index(&self) -> Result<(IntPatchPoint, i32), Failure> {
        let state = self.state.read();
        let point: IntPatchPoint = state.vertices.last_point(CLASS)?.clone();
        Ok((point, state.vertices.indl))
    }

    /// `SetPeriod(pu1, pv1, pu2, pv2)`. `line.set_period(2.0 * PI, 0.0, 0.0, 0.0)`.
    pub fn set_period(&self, pu1: f64, pv1: f64, pu2: f64, pv2: f64) {
        self.state.write().periods = [pu1, pv1, pu2, pv2];
    }

    /// `U1Period()`. `line.u1_period()`.
    pub fn u1_period(&self) -> f64 {
        self.state.read().periods[0]
    }

    /// `V1Period()`. `line.v1_period()`.
    pub fn v1_period(&self) -> f64 {
        self.state.read().periods[1]
    }

    /// `U2Period()`. `line.u2_period()`.
    pub fn u2_period(&self) -> f64 {
        self.state.read().periods[2]
    }

    /// `V2Period()`. `line.v2_period()`.
    pub fn v2_period(&self) -> f64 {
        self.state.read().periods[3]
    }

    /// `SetArcOnS1(A)`. `line.set_arc_on_s1(&arc)`.
    pub fn set_arc_on_s1(&self, a: &TheArc) {
        self.state.write().arc_on_s1 = Some(a.clone());
    }

    /// `HasArcOnS1()`. `line.has_arc_on_s1()`.
    pub fn has_arc_on_s1(&self) -> bool {
        self.state.read().arc_on_s1.is_some()
    }

    /// `GetArcOnS1()`: the arc, a null handle when none was set. `line.get_arc_on_s1()`.
    pub fn get_arc_on_s1(&self) -> TheArc {
        self.state.read().arc_on_s1.clone().unwrap_or_default()
    }

    /// `SetArcOnS2(A)`. `line.set_arc_on_s2(&arc)`.
    pub fn set_arc_on_s2(&self, a: &TheArc) {
        self.state.write().arc_on_s2 = Some(a.clone());
    }

    /// `HasArcOnS2()`. `line.has_arc_on_s2()`.
    pub fn has_arc_on_s2(&self) -> bool {
        self.state.read().arc_on_s2.is_some()
    }

    /// `GetArcOnS2()`: the arc, a null handle when none was set. `line.get_arc_on_s2()`.
    pub fn get_arc_on_s2(&self) -> TheArc {
        self.state.read().arc_on_s2.clone().unwrap_or_default()
    }

    /// `InsertVertexBefore(theIndex, thePnt)`: appended when `index` is beyond the last
    /// vertex; otherwise inserted before the vertex `index` (Standard_OutOfRange for an
    /// index below 1 on a non-empty line: the native code clamps a copy of the index but
    /// inserts at the original one). `line.insert_vertex_before(1, &vertex)?`.
    pub fn insert_vertex_before(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        let mut state = self.state.write();
        let nb_vertexes: i32 = state.vertices.nb_vertex();
        let an_index: i32 = index.max(1);
        if an_index > nb_vertexes {
            state.vertices.svtx.push(pnt.clone());
            return Ok(());
        }
        let member: String = format!("{CLASS}::InsertVertexBefore");
        super::failures::sequence_insert_before(
            &mut state.vertices.svtx,
            index,
            pnt.clone(),
            &member,
        )
    }

    /// `EnablePurging(theIsEnabled)`. `line.enable_purging(false)`.
    pub fn enable_purging(&self, is_enabled: bool) {
        self.state.write().purger_allowed = is_enabled;
    }

    /// `IsPurgingAllowed()`. `line.is_purging_allowed()`.
    pub fn is_purging_allowed(&self) -> bool {
        self.state.read().purger_allowed
    }

    /// `GetCreatingWay()`. `line.get_creating_way()`.
    pub fn get_creating_way(&self) -> IntPatchWLType {
        self.state.read().creation_way
    }

    /// `SetCreatingWayInfo(theAlgo)`. `line.set_creating_way_info(IntPatchWLType::PrmPrm)`.
    pub fn set_creating_way_info(&self, algo: IntPatchWLType) {
        self.state.write().creation_way = algo;
    }

    /// `ComputeVertexParameters(RTol)`: sorts the vertices on the line, merges them with
    /// the walking points (inserting points where a vertex matches none), filters the
    /// duplicated vertices of the restrictions and adds the first and last vertices.
    /// `line.compute_vertex_parameters(1e-7)?`.
    pub fn compute_vertex_parameters(&self, rtol: f64) -> Result<(), Failure> {
        let curv: Handle<IntSurfLineOn2S> = self.points("ComputeVertexParameters")?;
        let mut state = self.state.write();
        compute::compute_vertex_parameters(&mut state, &curv, rtol)
    }

    /// `Dump(theMode)`: 0 the points and vertices, 1 the 3d points, 2 the points on the
    /// first surface, any other mode the points on the second surface.
    /// `line.dump(1, &mut text)?`.
    pub fn dump(&self, mode: i32, out: &mut dyn std::fmt::Write) -> Result<(), Failure> {
        super::point_line_dump::dump_point_line(self, "W", mode, out)
    }
}

impl IntPatchLine for IntPatchWLine {
    fn line_base(&self) -> &IntPatchLineBase {
        &self.base
    }
}

impl IntPatchPointLine for IntPatchWLine {
    fn add_vertex(&self, pnt: &IntPatchPoint, prepend: bool) {
        self.state.write().vertices.add(pnt, prepend);
    }
    fn nb_pnts(&self) -> Result<i32, Failure> {
        Ok(self.points("NbPnts")?.nb_points())
    }
    fn nb_vertex(&self) -> i32 {
        self.state.read().vertices.nb_vertex()
    }
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.points("Point")?.value(index)
    }
    fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        let member: String = format!("{CLASS}::Vertex");
        self.state.read().vertices.value(index, &member).cloned()
    }
    fn change_vertex(
        &self,
        index: i32,
        edit: &mut dyn FnMut(&mut IntPatchPoint),
    ) -> Result<(), Failure> {
        let member: String = format!("{CLASS}::ChangeVertex");
        edit(self.state.write().vertices.value_mut(index, &member)?);
        Ok(())
    }
    fn clear_vertexes(&self) {
        self.state.write().vertices.svtx.clear();
    }
    fn remove_vertex(&self, index: i32) -> Result<(), Failure> {
        self.state.write().vertices.remove_vertex(index, CLASS)
    }
    fn curve(&self) -> Handle<IntSurfLineOn2S> {
        self.state.read().curv.clone()
    }
    fn is_out_surf1_box(&self, p1: Point2) -> Result<bool, Failure> {
        Ok(self.points("IsOutSurf1Box")?.is_out_surf1_box(p1))
    }
    fn is_out_surf2_box(&self, p2: Point2) -> Result<bool, Failure> {
        Ok(self.points("IsOutSurf2Box")?.is_out_surf2_box(p2))
    }
    fn is_out_box(&self, p: Point3) -> Result<bool, Failure> {
        Ok(self.points("IsOutBox")?.is_out_box(p))
    }
}
