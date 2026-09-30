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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchOnBoundaries.gxx
// and IntStart_SearchOnBoundaries.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntStart_SearchOnBoundaries`: the points and the parts of the restrictions of a
//! surface domain where the function vanishes (docs/geomalgo-design.md section 12).
//!
//! The body is split by responsibility: the constructor, `Perform` and the consultation
//! (this file), the search on one bounded arc (`bounded_arc.rs`, `arc_solutions.rs`),
//! the vertex and point bookkeeping (`points.rs`), the tangent line on a cylinder and the
//! degeneracy tests (`treat_lc.rs`) and the bounds of an infinite arc (`infinite.rs`).
mod arc_solutions;
mod bounded_arc;
mod infinite;
mod points;
mod treat_lc;

use super::failures::{not_done, sequence_value};
use super::tools::{
    SearchOnBoundariesTypes, TheArc, TheArcFunction, TheSOBTool, TheSegment, TheTopolTool,
};
use super::tools::{ThePathPoint, TheVertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// `IntStart_SearchOnBoundaries`: the solution points and segments found on the arcs
/// of the domain by the last Perform.
///
/// ```ignore
/// let mut search = SearchOnBoundaries::<TheSOnBoundsTypes>::new();
/// search.perform(&mut function, &domain, 1.0e-7, 1.0e-6, false)?;
/// ```
pub struct SearchOnBoundaries<T: SearchOnBoundariesTypes> {
    done: bool,
    all: bool,
    sseg: Vec<T::Segment>,
    spnt: Vec<T::PathPoint>,
}

impl<T: SearchOnBoundariesTypes> Default for SearchOnBoundaries<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: SearchOnBoundariesTypes> SearchOnBoundaries<T> {
    /// The empty constructor: not done. `SearchOnBoundaries::<T>::new().is_done() == false`.
    pub fn new() -> Self {
        Self {
            done: false,
            all: false,
            sseg: Vec::new(),
            spnt: Vec::new(),
        }
    }

    /// `Perform(F, Domain, TolBoundary, TolTangency, RecheckOnRegularity)`: finds on every
    /// arc of `domain` the points and the parts where `func` vanishes. `tol_boundary`
    /// decides whether a part of an arc is on the quadric, `tol_tangency` whether a point
    /// is. `search.perform(&mut f, &domain, 1e-7, 1e-6, false)?`.
    pub fn perform(
        &mut self,
        func: &mut T::Function,
        domain: &T::TopolTool,
        tol_boundary: f64,
        tol_tangency: f64,
        recheck_on_regularity: bool,
    ) -> Result<(), Failure> {
        self.done = false;
        self.spnt.clear();
        self.sseg.clear();
        domain.init();
        self.all = domain.more();
        while domain.more() {
            let arc: TheArc = domain.value()?;
            if !T::SOBTool::has_been_seen(&arc)? {
                let tols: [f64; 2] = [tol_boundary, tol_tangency];
                let arcsol: bool =
                    self.new_arc((func, domain), &arc, tols, recheck_on_regularity)?;
                self.all = self.all && arcsol;
            } else {
                // As it seems this is never reached: IntPatch_HInterTool::HasBeenSeen
                // always returns false.
                self.seen_arc(&arc)?;
            }
            domain.next();
        }
        self.done = true;
        Ok(())
    }

    /// The search on an arc not seen yet: its vertices, then its bounded range.
    fn new_arc(
        &mut self,
        (func, domain): (&mut T::Function, &T::TopolTool),
        arc: &TheArc,
        [tol_boundary, tol_tangency]: [f64; 2],
        recheck_on_regularity: bool,
    ) -> Result<bool, Failure> {
        func.set_arc(arc);
        points::find_vertex::<T>(arc, domain, func, &mut self.spnt, tol_boundary)?;
        let (mut pdeb, mut pfin) = T::SOBTool::bounds(arc)?;
        if precision::is_negative_infinite(pdeb) || precision::is_positive_infinite(pfin) {
            (pdeb, pfin) = infinite::compute_bounds_from_infinite(func)?;
        }
        let mut arc_search = bounded_arc::BoundedArc::<T> {
            arc,
            domain,
            range: [pdeb, pfin],
            pnt: &mut self.spnt,
            seg: &mut self.sseg,
            tol_boundary,
            tol_tangency,
        };
        arc_search.run(func, recheck_on_regularity)
    }

    /// The points and segments the tool already knows on a seen arc.
    fn seen_arc(&mut self, arc: &TheArc) -> Result<(), Failure> {
        let nbfound: i32 = self.spnt.len() as i32;
        for i in 1..=T::SOBTool::nb_points(arc)? {
            let (pt, tol, prm) = T::SOBTool::value(arc, i)?;
            if T::SOBTool::is_vertex(arc, i)? {
                let vtx: TheVertex = T::SOBTool::vertex(arc, i)?;
                self.spnt
                    .push(T::PathPoint::on_vertex(pt, tol, &vtx, arc, prm));
            } else {
                self.spnt.push(T::PathPoint::on_arc(pt, tol, arc, prm));
            }
        }
        for i in 1..=T::SOBTool::nb_segments(arc)? {
            let mut newseg: T::Segment = T::Segment::default();
            newseg.set_value(arc);
            if let Some(index) = T::SOBTool::has_first_point(arc, i)? {
                newseg.set_limit_point(self.known_point(nbfound + index)?, true);
            }
            if let Some(index) = T::SOBTool::has_last_point(arc, i)? {
                newseg.set_limit_point(self.known_point(nbfound + index)?, false);
            }
            self.sseg.push(newseg);
        }
        // `all & IsAllSolution(A)`: the native bitwise and evaluates the tool call.
        self.all &= T::SOBTool::is_all_solution(arc)?;
        Ok(())
    }

    fn known_point(&self, index: i32) -> Result<&T::PathPoint, Failure> {
        sequence_value(&self.spnt, index, "NCollection_Sequence::Value")
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(not_done(member))
    }

    /// `IsDone()`. `search.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `AllArcSolution()`: whether every arc is solution; StdFail_NotDone before a
    /// Perform. `search.all_arc_solution()?`.
    pub fn all_arc_solution(&self) -> Result<bool, Failure> {
        self.check_done("IntStart_SearchOnBoundaries::AllArcSolution")?;
        Ok(self.all)
    }

    /// `NbPoints()`: StdFail_NotDone before a Perform. `search.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("IntStart_SearchOnBoundaries::NbPoints")?;
        Ok(self.spnt.len() as i32)
    }

    /// `Point(Index)`: StdFail_NotDone before a Perform, Standard_OutOfRange outside
    /// `1..=NbPoints()`. `search.point(1)?`.
    pub fn point(&self, index: i32) -> Result<&T::PathPoint, Failure> {
        self.check_done("IntStart_SearchOnBoundaries::Point")?;
        sequence_value(&self.spnt, index, "IntStart_SearchOnBoundaries::Point")
    }

    /// `NbSegments()`: StdFail_NotDone before a Perform. `search.nb_segments()?`.
    pub fn nb_segments(&self) -> Result<i32, Failure> {
        self.check_done("IntStart_SearchOnBoundaries::NbSegments")?;
        Ok(self.sseg.len() as i32)
    }

    /// `Segment(Index)`: StdFail_NotDone before a Perform, Standard_OutOfRange outside
    /// `1..=NbSegments()`. `search.segment(1)?`.
    pub fn segment(&self, index: i32) -> Result<&T::Segment, Failure> {
        self.check_done("IntStart_SearchOnBoundaries::Segment")?;
        sequence_value(&self.sseg, index, "IntStart_SearchOnBoundaries::Segment")
    }
}

/// `Func.Value(X, F)` where the native code ignores the returned flag: the output keeps
/// its previous value when the function answers false (the native variable is then
/// either the previous value or uninitialized).
fn value_or_keep<F: TheArcFunction>(func: &mut F, x: f64, previous: f64) -> Result<f64, Failure> {
    Ok(func.checked_value(x)?.unwrap_or(previous))
}
