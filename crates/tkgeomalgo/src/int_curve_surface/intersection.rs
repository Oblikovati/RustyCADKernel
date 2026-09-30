// Created on: 1993-04-07
// Created by: Laurent BUCHARD
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of IntCurveSurface_Intersection.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_Intersection`: the root class of the curve-surface
//! intersection algorithms, holding their points and segments.
use super::failures::{not_done, out_of_range};
use super::{IntersectionPoint, IntersectionSegment};
use rustycad_tkernel::standard::Failure;
use std::fmt;

/// `PARAMEQUAL(a, b)`: two parameters closer than 1e-8.
fn param_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-8
}

/// `IntCurveSurface_Intersection`: the points and segments of a curve-surface
/// intersection. The algorithms hold one and expose it through `Deref`; the
/// protected native members are public for algorithms of other crates.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::Intersection;
/// let inter = Intersection::new();
/// assert!(!inter.is_done() && inter.nb_points().is_err());
/// ```
#[derive(Clone, Debug, Default)]
pub struct Intersection {
    /// `done`: the computation succeeded (protected native field).
    pub done: bool,
    /// `myIsParallel`: the curve is parallel to the surface or lies on it (protected
    /// native field; recognized for some analytic pairs only).
    pub my_is_parallel: bool,
    lpnt: Vec<IntersectionPoint>,
    lseg: Vec<IntersectionSegment>,
}

impl Intersection {
    /// `IntCurveSurface_Intersection()` (protected): not done, not parallel;
    /// `Intersection::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IsDone()`; `inter.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: the curve is parallel to the surface or lies on it;
    /// `inter.is_parallel()`.
    pub fn is_parallel(&self) -> bool {
        self.my_is_parallel
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            Ok(())
        } else {
            Err(not_done(member))
        }
    }
    /// `NbPoints()`; StdFail_NotDone when not done; `inter.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("IntCurveSurface_Intersection::NbPoints")?;
        Ok(self.lpnt.len() as i32)
    }
    /// `NbSegments()`; StdFail_NotDone when not done; `inter.nb_segments()?`.
    pub fn nb_segments(&self) -> Result<i32, Failure> {
        self.check_done("IntCurveSurface_Intersection::NbSegments")?;
        Ok(self.lseg.len() as i32)
    }
    /// `Point(Index)`: StdFail_NotDone when not done, Standard_OutOfRange outside
    /// `1..=NbPoints()`; `inter.point(1)?`.
    pub fn point(&self, index: i32) -> Result<IntersectionPoint, Failure> {
        self.check_done("IntCurveSurface_Intersection::Point")?;
        rank(&self.lpnt, index, "IntCurveSurface_Intersection::Point")
    }
    /// `Segment(Index)`: StdFail_NotDone when not done, Standard_OutOfRange outside
    /// `1..=NbSegments()`; `inter.segment(1)?`.
    pub fn segment(&self, index: i32) -> Result<IntersectionSegment, Failure> {
        self.check_done("IntCurveSurface_Intersection::Segment")?;
        rank(&self.lseg, index, "IntCurveSurface_Intersection::Segment")
    }
    /// `SetValues(Inter)` (protected): copies a done result with its parallel flag,
    /// or marks this one not done and not parallel; `inter.set_values(&other)`.
    pub fn set_values(&mut self, other: &Intersection) {
        if !other.done {
            (self.done, self.my_is_parallel) = (false, false);
            return;
        }
        self.lpnt = other.lpnt.clone();
        self.lseg = other.lseg.clone();
        (self.done, self.my_is_parallel) = (true, other.my_is_parallel);
    }
    /// `Append(Inter, FirstParamOnCurve, LastParamOnCurve)` (protected): appends the
    /// points (through [`Intersection::append_point`]) and segments of a done result;
    /// the parameter range is unused natively; `inter.append(&other, 0.0, 1.0)`.
    pub fn append(&mut self, other: &Intersection, _first: f64, _last: f64) {
        if !other.done {
            return;
        }
        other
            .lpnt
            .iter()
            .for_each(|&point| self.append_point(point));
        other
            .lseg
            .iter()
            .for_each(|&segment| self.append_segment(segment));
    }
    /// `Append(Pt)` (protected): appends the point unless one with the same (u, v, w)
    /// within 1e-8 and the same transition is already there; `inter.append_point(p)`.
    pub fn append_point(&mut self, other: IntersectionPoint) {
        let same = |known: &IntersectionPoint| {
            param_equal(other.u(), known.u())
                && param_equal(other.v(), known.v())
                && param_equal(other.w(), known.w())
                && known.transition() == other.transition()
        };
        if !self.lpnt.iter().any(same) {
            self.lpnt.push(other);
        }
    }
    /// `Append(Seg)` (protected): appends the segment; `inter.append_segment(s)`.
    pub fn append_segment(&mut self, segment: IntersectionSegment) {
        self.lseg.push(segment);
    }
    /// `ResetFields()` (protected): a done result is emptied and marked not done
    /// and not parallel; a not-done one is left as it is; `inter.reset_fields()`.
    pub fn reset_fields(&mut self) {
        if self.done {
            self.lseg.clear();
            self.lpnt.clear();
            (self.done, self.my_is_parallel) = (false, false);
        }
    }
    /// `Dump()`: every point and segment, or ` Intersection NotDone`;
    /// `inter.dump(&mut text)?`.
    pub fn dump(&self, out: &mut dyn fmt::Write) -> fmt::Result {
        if !self.done {
            return writeln!(out, " Intersection NotDone");
        }
        for point in &self.lpnt {
            point.dump(out)?;
        }
        self.lseg.iter().try_for_each(|segment| segment.dump(out))
    }
}

/// The element of rank `index` (1-based) of a sequence, as NCollection_Sequence::Value.
fn rank<T: Copy>(items: &[T], index: i32, member: &str) -> Result<T, Failure> {
    let found = usize::try_from(index - 1).ok().and_then(|at| items.get(at));
    found
        .copied()
        .ok_or_else(|| out_of_range(member, index, items.len()))
}
