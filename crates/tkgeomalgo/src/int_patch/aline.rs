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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ALine.hxx/.lxx and the
// constructors, Curve and AddVertex of IntPatch_ALine.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ALine`: an intersection line described by an analytic curve
//! (`IntAna_Curve`).
use super::IntPatchIType;
use super::aline_compute::compute_vertex_parameters;
use super::line::{IntPatchLine, IntPatchLineBase, IntPatchLineTransitions};
use super::point::IntPatchPoint;
use super::vertices::VertexList;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use rustycad_tkgeombase::int_ana::Curve as IntAnaCurve;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The fields of `IntPatch_ALine`.
struct ALineState {
    curv: IntAnaCurve,
    vertices: VertexList,
}

/// `IntPatch_ALine`: an analytic intersection line with its vertices.
///
/// ```
/// use rustycad_tkgeombase::int_ana::Curve;
/// use rustycad_tkgeomalgo::int_patch::{IntPatchALine, IntPatchIType, IntPatchLine, IntPatchLineTransitions};
/// let line = IntPatchALine::new(&Curve::new(), false, IntPatchLineTransitions::Undecided);
/// assert_eq!(line.arc_type(), IntPatchIType::Analytic);
/// assert!(line.first_parameter().is_err());
/// ```
pub struct IntPatchALine {
    transient: StandardTransient,
    base: IntPatchLineBase,
    state: SharedState<ALineState>,
}
impl_transient!(IntPatchALine, "IntPatch_ALine", dyn IntPatchLine, transient,
    interfaces: [dyn IntPatchLine]);

pub(super) const ALINE: &str = "IntPatch_ALine";

impl IntPatchALine {
    /// `IntPatch_ALine(C, Tang, ...)`, the three constructors by their transitions.
    /// `IntPatchALine::new(&curve, false, IntPatchLineTransitions::Undecided)`.
    pub fn new(c: &IntAnaCurve, tang: bool, transitions: IntPatchLineTransitions) -> Self {
        let state = ALineState {
            curv: *c,
            vertices: VertexList::default(),
        };
        Self {
            transient: StandardTransient::default(),
            base: IntPatchLineBase::new(tang, transitions, IntPatchIType::Analytic),
            state: SharedState::new(state),
        }
    }

    /// `AddVertex(Pnt)`: appends the vertex. `line.add_vertex(&vertex)`.
    pub fn add_vertex(&self, pnt: &IntPatchPoint) {
        self.state.write().vertices.svtx.push(pnt.clone());
    }

    /// `Replace(Index, Pnt)`: Standard_OutOfRange outside `1..=NbVertex()`.
    /// `line.replace(1, &vertex)?`.
    pub fn replace(&self, index: i32, pnt: &IntPatchPoint) -> Result<(), Failure> {
        let member: String = format!("{ALINE}::Replace");
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

    /// `FirstParameter(IsIncluded)`: the first parameter of the curve and whether the
    /// curve is defined there; Standard_DomainError for an unbounded curve domain.
    /// `line.first_parameter()?`.
    pub fn first_parameter(&self) -> Result<(f64, bool), Failure> {
        let curv: IntAnaCurve = self.state.read().curv;
        let (first, _) = curv.domain()?;
        Ok((first, !curv.is_first_open()))
    }

    /// `LastParameter(IsIncluded)`: the last parameter and whether the curve is defined
    /// there. `line.last_parameter()?`.
    pub fn last_parameter(&self) -> Result<(f64, bool), Failure> {
        let curv: IntAnaCurve = self.state.read().curv;
        let (_, last) = curv.domain()?;
        Ok((last, !curv.is_last_open()))
    }

    /// `Value(U)`: the point of parameter `u`; Standard_DomainError outside the domain.
    /// `line.value(0.5)?`.
    pub fn value(&self, u: f64) -> Result<Point3, Failure> {
        let curv: IntAnaCurve = self.state.read().curv;
        curv.value(u)
    }

    /// `D1(U, P, Du)`: stores the point in `p`, and the derivative in `du` when it is
    /// defined (the returned flag). `line.d1(0.5, &mut p, &mut du)?`.
    pub fn d1(&self, u: f64, p: &mut Point3, du: &mut Vector3) -> Result<bool, Failure> {
        let curv: IntAnaCurve = self.state.read().curv;
        curv.d1u(u, p, du)
    }

    /// `FindParameter(P, theParams)`: the parameters of `p` on the curve, in ascending
    /// order. `line.find_parameter(p, &mut params)?`.
    pub fn find_parameter(&self, p: Point3, params: &mut List<f64>) -> Result<(), Failure> {
        let curv: IntAnaCurve = self.state.read().curv;
        curv.find_parameter(p, params)
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
        self.state.read().vertices.first_point(ALINE).cloned()
    }

    /// `LastPoint()`: Standard_DomainError without a last point. `line.last_point()?`.
    pub fn last_point(&self) -> Result<IntPatchPoint, Failure> {
        self.state.read().vertices.last_point(ALINE).cloned()
    }

    /// `NbVertex()`. `line.nb_vertex()`.
    pub fn nb_vertex(&self) -> i32 {
        self.state.read().vertices.nb_vertex()
    }

    /// `Vertex(Index)`: Standard_OutOfRange outside `1..=NbVertex()`. `line.vertex(1)?`.
    pub fn vertex(&self, index: i32) -> Result<IntPatchPoint, Failure> {
        let member: String = format!("{ALINE}::Vertex");
        self.state.read().vertices.value(index, &member).cloned()
    }

    /// `ChangeVertex(theIndex)`: runs `edit` on the vertex of range `index` in place.
    /// `line.change_vertex(1, &mut |v| v.set_parameter(0.0))?`.
    pub fn change_vertex(
        &self,
        index: i32,
        edit: &mut dyn FnMut(&mut IntPatchPoint),
    ) -> Result<(), Failure> {
        let member: String = format!("{ALINE}::ChangeVertex");
        edit(self.state.write().vertices.value_mut(index, &member)?);
        Ok(())
    }

    /// `ComputeVertexParameters(Tol)`: completes the vertices of a line parametrized
    /// over more than 2 Pi, sorts them, filters the duplicates, closes a periodic line
    /// and merges the vertices of the same parameter. `line.compute_vertex_parameters(1e-7)?`.
    pub fn compute_vertex_parameters(&self, tol: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        let curv: IntAnaCurve = state.curv;
        compute_vertex_parameters(&mut state.vertices, &curv, tol)
    }

    /// `Curve()`: a copy of the analytic curve. `line.curve()`.
    pub fn curve(&self) -> IntAnaCurve {
        self.state.read().curv
    }
}

impl IntPatchLine for IntPatchALine {
    fn line_base(&self) -> &IntPatchLineBase {
        &self.base
    }
}
