// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the result members of Extrema_ExtElC.cxx and
// Extrema_ExtElC2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The result that Extrema_ExtElC (six extrema) and Extrema_ExtElC2d (eight) share:
//! a parallel flag and, per extremum, a square distance and a point on each curve.
use super::failures::{extremum_index, not_done};
use super::pon::POnCurve;
use rustycad_tkernel::standard::Failure;

/// The extrema between two elementary curves; `N` is the native array size.
#[derive(Clone, Debug)]
pub struct CurvePairExtrema<P, const N: usize> {
    class_name: &'static str,
    pub(super) done: bool,
    pub(super) is_par: bool,
    pub(super) nb_ext: i32,
    pub(super) sq_dist: [f64; N],
    pub(super) points: [[POnCurve<P>; 2]; N],
}

impl<P: Copy + Default, const N: usize> CurvePairExtrema<P, N> {
    /// The state every native constructor starts from: not done, not parallel, the
    /// square distances at RealLast() and the default points (parameter 0 at the
    /// origin), which the parallel results never assign.
    pub(super) fn new(class_name: &'static str) -> Self {
        Self {
            class_name,
            done: false,
            is_par: false,
            nb_ext: 0,
            sq_dist: [f64::MAX; N],
            points: [[POnCurve::default(); 2]; N],
        }
    }
    /// Appends one extremum at `myNbExt` and increments it.
    pub(super) fn push(&mut self, sq_dist: f64, first: POnCurve<P>, second: POnCurve<P>) {
        let index: usize = self.nb_ext as usize;
        self.sq_dist[index] = sq_dist;
        self.points[index] = [first, second];
        self.nb_ext += 1;
    }
    /// The parallel result: one extremum whose points stay unassigned.
    pub(super) fn set_parallel(&mut self, sq_dist: f64) {
        self.is_par = true;
        self.sq_dist[0] = sq_dist;
        self.nb_ext = 1;
        self.done = true;
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        self.check_done("IsParallel")?;
        Ok(self.is_par)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        self.check_done("NbExt")?;
        Ok(self.nb_ext)
    }
    /// `SquareDistance(N)` (the native default is 1); Standard_OutOfRange outside
    /// `[1, NbExt()]`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.sq_dist[self.index("SquareDistance", n)?])
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnCurve<P>, POnCurve<P>), Failure> {
        let [first, second] = self.points[self.index("Points", n)?];
        Ok((first, second))
    }
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(not_done(&format!("{}::{member}()", self.class_name)))
    }
    fn index(&self, member: &str, n: i32) -> Result<usize, Failure> {
        let name: String = format!("{}::{member}()", self.class_name);
        extremum_index(&name, n, || self.nb_ext())
    }
}
