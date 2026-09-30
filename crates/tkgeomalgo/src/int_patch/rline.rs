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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_RLine.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_RLine`: an intersection line that is a restriction of one of the surfaces.
use super::IntPatchIType;
use super::failures::domain_error;
use super::line::{IntPatchLine, IntPatchLineBase, IntPatchLineTransitions};
use super::point::IntPatchPoint;
use super::point_line::IntPatchPointLine;
use super::rline_compute::compute_vertex_parameters;
use super::vertices::VertexList;
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfSituation, IntSurfTypeTrans};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Point2, Point3};

/// The fields of `IntPatch_RLine` (`ParamInf1`...`ParamSup2` are never set natively
/// beyond their 0 initial values and have no reader, so they are not kept).
#[derive(Clone)]
struct RLineState {
    arc_on_s1: TheArc,
    arc_on_s2: TheArc,
    on_s1: bool,
    on_s2: bool,
    curv: Handle<IntSurfLineOn2S>,
    vertices: VertexList,
}

/// `IntPatch_RLine`: a restriction line, with its arcs, its optional point set and its
/// vertices.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchRLine;
/// let line = IntPatchRLine::new_undecided(false);
/// assert!(!line.is_arc_on_s1() && !line.has_polygon());
/// assert_eq!(line.param_on_s1(), (0.0, 0.0));
/// ```
pub struct IntPatchRLine {
    transient: StandardTransient,
    base: IntPatchLineBase,
    state: SharedState<RLineState>,
}
impl_transient!(IntPatchRLine, "IntPatch_RLine", dyn IntPatchPointLine, transient,
    interfaces: [dyn IntPatchLine, dyn IntPatchPointLine]);

pub(super) const RLINE: &str = "IntPatch_RLine";

impl IntPatchRLine {
    fn with_transitions(tang: bool, transitions: IntPatchLineTransitions) -> Self {
        let state = RLineState {
            arc_on_s1: TheArc::default(),
            arc_on_s2: TheArc::default(),
            on_s1: false,
            on_s2: false,
            curv: Handle::default(),
            vertices: VertexList::default(),
        };
        Self {
            transient: StandardTransient::default(),
            base: IntPatchLineBase::new(tang, transitions, IntPatchIType::Restriction),
            state: SharedState::new(state),
        }
    }

    /// `IntPatch_RLine(Tang, Trans1, Trans2)`: IN or OUT transitions.
    /// `IntPatchRLine::new(false, IntSurfTypeTrans::In, IntSurfTypeTrans::Out)`.
    pub fn new(tang: bool, trans1: IntSurfTypeTrans, trans2: IntSurfTypeTrans) -> Self {
        Self::with_transitions(tang, IntPatchLineTransitions::Types(trans1, trans2))
    }

    /// `IntPatch_RLine(Tang, Situ1, Situ2)`: TOUCH transitions.
    /// `IntPatchRLine::new_touch(true, IntSurfSituation::Inside, IntSurfSituation::Unknown)`.
    pub fn new_touch(tang: bool, situ1: IntSurfSituation, situ2: IntSurfSituation) -> Self {
        Self::with_transitions(tang, IntPatchLineTransitions::Touch(situ1, situ2))
    }

    /// `IntPatch_RLine(Tang)`: undecided transitions. `IntPatchRLine::new_undecided(false)`.
    pub fn new_undecided(tang: bool) -> Self {
        Self::with_transitions(tang, IntPatchLineTransitions::Undecided)
    }

    /// A copy of the line (the implicit copy constructor `IntPatch_RLine(const
    /// IntPatch_RLine&)`): the arcs and the point set stay shared handles, the vertices
    /// and the fields are copied. `let copy = line.duplicate();`.
    pub(crate) fn duplicate(&self) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: self.base.duplicate(),
            state: SharedState::new(self.state.read().clone()),
        }
    }

    /// The point set, or the Standard_DomainError of a line without one.
    fn points(&self, member: &str) -> Result<Handle<IntSurfLineOn2S>, Failure> {
        let curv: Handle<IntSurfLineOn2S> = self.state.read().curv.clone();
        if curv.is_null() {
            let member: String = format!("{RLINE}::{member}");
            return Err(domain_error(
                &member,
                "the restriction line has no point set",
            ));
        }
        Ok(curv)
    }

    /// `Replace(Index, Pnt)`: Standard_OutOfRange outside `1..=NbVertex()`.
    /// `line.replace(1, &vertex)?`.
    pub fn replace(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        let member: String = format!("{RLINE}::Replace");
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

    /// `Add(L)`: the point set of the line. `line.add(&points)`.
    pub fn add(&self, l: &Handle<IntSurfLineOn2S>) {
        self.state.write().curv = l.clone();
    }

    /// `IsArcOnS1()`. `line.is_arc_on_s1()`.
    pub fn is_arc_on_s1(&self) -> bool {
        self.state.read().on_s1
    }

    /// `IsArcOnS2()`. `line.is_arc_on_s2()`.
    pub fn is_arc_on_s2(&self) -> bool {
        self.state.read().on_s2
    }

    /// `SetArcOnS1(A)`. `line.set_arc_on_s1(&arc)`.
    pub fn set_arc_on_s1(&self, a: &TheArc) {
        let mut state = self.state.write();
        (state.arc_on_s1, state.on_s1) = (a.clone(), true);
    }

    /// `SetArcOnS2(A)`. `line.set_arc_on_s2(&arc)`.
    pub fn set_arc_on_s2(&self, a: &TheArc) {
        let mut state = self.state.write();
        (state.arc_on_s2, state.on_s2) = (a.clone(), true);
    }

    /// `ArcOnS1()`: the arc (a null handle before `SetArcOnS1`). `line.arc_on_s1()`.
    pub fn arc_on_s1(&self) -> TheArc {
        self.state.read().arc_on_s1.clone()
    }

    /// `ArcOnS2()`: the arc (a null handle before `SetArcOnS2`). `line.arc_on_s2()`.
    pub fn arc_on_s2(&self) -> TheArc {
        self.state.read().arc_on_s2.clone()
    }

    /// `ParamOnS1(p1, p2)`: the range of the vertex parameters on the line when the line
    /// is on an arc of S1 (`RealLast()` and `-RealLast()` without vertices), `(0, 0)`
    /// otherwise. `line.param_on_s1()`.
    pub fn param_on_s1(&self) -> (f64, f64) {
        let state = self.state.read();
        param_range(state.on_s1, &state.vertices)
    }

    /// `ParamOnS2(p1, p2)`: as [`Self::param_on_s1`] for S2. `line.param_on_s2()`.
    pub fn param_on_s2(&self) -> (f64, f64) {
        let state = self.state.read();
        param_range(state.on_s2, &state.vertices)
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
        self.state.read().vertices.first_point(RLINE).cloned()
    }

    /// `LastPoint()`: Standard_DomainError without a last point. `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.last_point(RLINE).cloned()
    }

    /// `HasPolygon()`: whether the line has a point set. `line.has_polygon()`.
    pub fn has_polygon(&self) -> bool {
        !self.state.read().curv.is_null()
    }

    /// `SetPoint(Index, Pnt)`: the point of range `index` of the point set becomes the
    /// point of `pnt`. `line.set_point(1, &vertex)?`.
    pub fn set_point(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        self.points("SetPoint")?.set_value(index, &pnt.pnt_on_2s())
    }

    /// `ComputeVertexParameters(Tol)`: filters the duplicated vertices of the
    /// restrictions, sorts the vertices and makes them the first and the last points
    /// (the tolerance is not used). `line.compute_vertex_parameters(1e-7)?`.
    pub fn compute_vertex_parameters(&self, _tol: f64) -> Result<(), Failure> {
        compute_vertex_parameters(&mut self.state.write().vertices)
    }

    /// `SetCurve(theNewCurve)`. `line.set_curve(&points)`.
    pub fn set_curve(&self, new_curve: &Handle<IntSurfLineOn2S>) {
        self.state.write().curv = new_curve.clone();
    }

    /// `Dump(theMode)`: as `IntPatch_WLine::Dump`; Standard_DomainError without a point
    /// set, after the banner. `line.dump(0, &mut text)?`.
    pub fn dump(&self, mode: i32, out: &mut dyn std::fmt::Write) -> Result<(), Failure> {
        super::point_line_dump::dump_point_line(self, "R", mode, out)
    }
}

/// The body of `ParamOnS1`/`ParamOnS2`, over the vertices from the last one.
fn param_range(on: bool, vertices: &VertexList) -> (f64, f64) {
    if !on {
        return (0.0, 0.0);
    }
    let mut a: f64 = real_last();
    let mut b: f64 = -a;
    for vertex in vertices.svtx.iter().rev() {
        let p: f64 = vertex.parameter_on_line();
        if p < a {
            a = p;
        }
        if p > b {
            b = p;
        }
    }
    (a, b)
}

impl IntPatchLine for IntPatchRLine {
    fn line_base(&self) -> &IntPatchLineBase {
        &self.base
    }
}

impl IntPatchPointLine for IntPatchRLine {
    fn add_vertex(&self, pnt: &IntPatchPoint, prepend: bool) {
        self.state.write().vertices.add(pnt, prepend);
    }
    /// Standard_DomainError without a point set.
    fn nb_pnts(&self) -> Result<i32, Failure> {
        Ok(self.points("NbPnts")?.nb_points())
    }
    fn nb_vertex(&self) -> i32 {
        self.state.read().vertices.nb_vertex()
    }
    /// Standard_DomainError without a point set.
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        self.points("Point")?.value(index)
    }
    fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        let member: String = format!("{RLINE}::Vertex");
        self.state.read().vertices.value(index, &member).cloned()
    }
    fn change_vertex(
        &self,
        index: i32,
        edit: &mut dyn FnMut(&mut IntPatchPoint),
    ) -> Result<(), Failure> {
        let member: String = format!("{RLINE}::ChangeVertex");
        edit(self.state.write().vertices.value_mut(index, &member)?);
        Ok(())
    }
    fn clear_vertexes(&self) {
        self.state.write().vertices.svtx.clear();
    }
    fn remove_vertex(&self, index: i32) -> Result<(), Failure> {
        self.state.write().vertices.remove_vertex(index, RLINE)
    }
    fn curve(&self) -> Handle<IntSurfLineOn2S> {
        self.state.read().curv.clone()
    }
    /// Without a point set the native call dereferences a null handle; the port raises
    /// Standard_DomainError, as `NbPnts` does.
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
