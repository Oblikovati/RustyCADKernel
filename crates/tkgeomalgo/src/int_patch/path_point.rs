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
// Rust translation of IntPatch_ThePathPointOfTheSOnBounds.hxx/_0.cxx and
// IntPatch_TheSegmentOfTheSOnBounds.hxx/_0.cxx (ModelingAlgorithms/TKGeomAlgo/IntPatch) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solution points and segments of `IntPatch_TheSOnBounds`: the IntStart
//! `IntStart_ThePathPoint` and `IntStart_TheSegment` of the IntPatch instance.
use super::failures::domain_error;
use crate::int_start::{TheArc, ThePathPoint, TheSegment, TheVertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `IntPatch_ThePathPointOfTheSOnBounds`: a solution point on an arc, on a vertex of the
/// domain or new.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchThePathPointOfTheSOnBounds;
/// let p = IntPatchThePathPointOfTheSOnBounds::new();
/// assert!(p.is_new() && p.vertex().is_err());
/// ```
#[derive(Clone)]
pub struct IntPatchThePathPointOfTheSOnBounds {
    point: Point3,
    tol: f64,
    isnew: bool,
    vtx: TheVertex,
    arc: TheArc,
    param: f64,
}

impl Default for IntPatchThePathPointOfTheSOnBounds {
    fn default() -> Self {
        Self::new()
    }
}

impl IntPatchThePathPointOfTheSOnBounds {
    /// The empty constructor: a new point at the origin. `IntPatchThePathPointOfTheSOnBounds::new()`.
    pub fn new() -> Self {
        Self {
            point: Point3::default(),
            tol: 0.0,
            isnew: true,
            vtx: TheVertex::default(),
            arc: TheArc::default(),
            param: 0.0,
        }
    }

    /// `(P, Tol, V, A, Parameter)`: a point on the vertex `v` of the arc.
    /// `IntPatchThePathPointOfTheSOnBounds::on_vertex(p, 1e-7, &v, &a, 0.5)`.
    pub fn on_vertex(p: Point3, tol: f64, v: &TheVertex, a: &TheArc, parameter: f64) -> Self {
        let mut point: Self = Self::new();
        point.set_value_on_vertex(p, tol, v, a, parameter);
        point
    }

    /// `(P, Tol, A, Parameter)`: a new point on the arc.
    /// `IntPatchThePathPointOfTheSOnBounds::on_arc(p, 1e-7, &a, 0.5)`.
    pub fn on_arc(p: Point3, tol: f64, a: &TheArc, parameter: f64) -> Self {
        let mut point: Self = Self::new();
        point.set_value_on_arc(p, tol, a, parameter);
        point
    }

    /// `SetValue(P, Tol, V, A, Parameter)`: not new. `p.set_value_on_vertex(pt, tol, &v, &a, 0.5)`.
    pub fn set_value_on_vertex(
        &mut self,
        p: Point3,
        tol: f64,
        v: &TheVertex,
        a: &TheArc,
        parameter: f64,
    ) {
        (self.isnew, self.point, self.tol, self.param) = (false, p, tol, parameter);
        (self.vtx, self.arc) = (v.clone(), a.clone());
    }

    /// `SetValue(P, Tol, A, Parameter)`: new; the vertex is kept, as natively.
    /// `p.set_value_on_arc(pt, tol, &a, 0.5)`.
    pub fn set_value_on_arc(&mut self, p: Point3, tol: f64, a: &TheArc, parameter: f64) {
        (self.isnew, self.point, self.tol, self.param) = (true, p, tol, parameter);
        self.arc = a.clone();
    }

    /// `Value()`. `p.value()`.
    pub fn value(&self) -> Point3 {
        self.point
    }

    /// `Tolerance()`. `p.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }

    /// `IsNew()`. `p.is_new()`.
    pub fn is_new(&self) -> bool {
        self.isnew
    }

    /// `Vertex()`: Standard_DomainError on a new point. `p.vertex()?`.
    pub fn vertex(&self) -> Result<TheVertex, Failure> {
        if self.isnew {
            let member = "IntPatch_ThePathPointOfTheSOnBounds::Vertex";
            return Err(domain_error(
                member,
                "the point is new; expected a point on a vertex",
            ));
        }
        Ok(self.vtx.clone())
    }

    /// `Arc()`. `p.arc()`.
    pub fn arc(&self) -> TheArc {
        self.arc.clone()
    }

    /// `Parameter()`. `p.parameter()`.
    pub fn parameter(&self) -> f64 {
        self.param
    }
}

/// `IntStart_ThePathPoint` of the IntPatch instance.
impl ThePathPoint for IntPatchThePathPointOfTheSOnBounds {
    fn on_vertex(p: Point3, tol: f64, vertex: &TheVertex, arc: &TheArc, param: f64) -> Self {
        IntPatchThePathPointOfTheSOnBounds::on_vertex(p, tol, vertex, arc, param)
    }
    fn on_arc(p: Point3, tol: f64, arc: &TheArc, param: f64) -> Self {
        IntPatchThePathPointOfTheSOnBounds::on_arc(p, tol, arc, param)
    }
    fn set_value_on_vertex(
        &mut self,
        p: Point3,
        tol: f64,
        vertex: &TheVertex,
        arc: &TheArc,
        param: f64,
    ) {
        IntPatchThePathPointOfTheSOnBounds::set_value_on_vertex(self, p, tol, vertex, arc, param);
    }
    fn set_value_on_arc(&mut self, p: Point3, tol: f64, arc: &TheArc, param: f64) {
        IntPatchThePathPointOfTheSOnBounds::set_value_on_arc(self, p, tol, arc, param);
    }
    fn is_new(&self) -> bool {
        self.isnew
    }
    fn vertex(&self) -> Result<TheVertex, Failure> {
        IntPatchThePathPointOfTheSOnBounds::vertex(self)
    }
    fn arc(&self) -> TheArc {
        self.arc.clone()
    }
    fn parameter(&self) -> f64 {
        self.param
    }
}

/// `IntPatch_TheSegmentOfTheSOnBounds`: a part of an arc that is solution, with its
/// optional first and last points.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchTheSegmentOfTheSOnBounds;
/// let s = IntPatchTheSegmentOfTheSOnBounds::new();
/// assert!(!s.has_first_point() && s.first_point().is_err());
/// ```
#[derive(Clone, Default)]
pub struct IntPatchTheSegmentOfTheSOnBounds {
    arc: TheArc,
    first: Option<IntPatchThePathPointOfTheSOnBounds>,
    last: Option<IntPatchThePathPointOfTheSOnBounds>,
}

const SEGMENT: &str = "IntPatch_TheSegmentOfTheSOnBounds";

impl IntPatchTheSegmentOfTheSOnBounds {
    /// The empty constructor: no arc, no point. `IntPatchTheSegmentOfTheSOnBounds::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetValue(A)`: the arc; forgets the points (their values stay unread).
    /// `s.set_value(&arc)`.
    pub fn set_value(&mut self, a: &TheArc) {
        (self.first, self.last, self.arc) = (None, None, a.clone());
    }

    /// `SetLimitPoint(V, First)`: the first (or last) point. `s.set_limit_point(&p, true)`.
    pub fn set_limit_point(&mut self, v: &IntPatchThePathPointOfTheSOnBounds, first: bool) {
        if first {
            self.first = Some(v.clone());
        } else {
            self.last = Some(v.clone());
        }
    }

    /// `Curve()`: the arc. `s.curve()`.
    pub fn curve(&self) -> TheArc {
        self.arc.clone()
    }

    /// `HasFirstPoint()`. `s.has_first_point()`.
    pub fn has_first_point(&self) -> bool {
        self.first.is_some()
    }

    /// `FirstPoint()`: Standard_DomainError without a first point. `s.first_point()?`.
    pub fn first_point(&self) -> Result<&IntPatchThePathPointOfTheSOnBounds, Failure> {
        let member: String = format!("{SEGMENT}::FirstPoint");
        self.first
            .as_ref()
            .ok_or_else(|| domain_error(&member, "the segment has no first point"))
    }

    /// `HasLastPoint()`. `s.has_last_point()`.
    pub fn has_last_point(&self) -> bool {
        self.last.is_some()
    }

    /// `LastPoint()`: Standard_DomainError without a last point. `s.last_point()?`.
    pub fn last_point(&self) -> Result<&IntPatchThePathPointOfTheSOnBounds, Failure> {
        let member: String = format!("{SEGMENT}::LastPoint");
        self.last
            .as_ref()
            .ok_or_else(|| domain_error(&member, "the segment has no last point"))
    }
}

/// `IntStart_TheSegment` of the IntPatch instance.
impl TheSegment for IntPatchTheSegmentOfTheSOnBounds {
    type PathPoint = IntPatchThePathPointOfTheSOnBounds;
    fn set_value(&mut self, arc: &TheArc) {
        IntPatchTheSegmentOfTheSOnBounds::set_value(self, arc);
    }
    fn set_limit_point(&mut self, point: &IntPatchThePathPointOfTheSOnBounds, first: bool) {
        IntPatchTheSegmentOfTheSOnBounds::set_limit_point(self, point, first);
    }
}
