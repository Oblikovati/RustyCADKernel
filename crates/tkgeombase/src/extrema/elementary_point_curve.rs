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
// Rust translation of the result members of Extrema_ExtPElC.cxx and
// Extrema_ExtPElC2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TheExtPElC` parameter of Extrema_GGExtPC ([`ElementaryPointCurve`]) and the
//! result storage that Extrema_ExtPElC and Extrema_ExtPElC2d share.
use super::curve_tool::ExtremaCurveTool;
use super::failures::{extremum_index, not_done};
use super::gp_space::GpPoint;
use super::pon::POnCurve;
use rustycad_tkernel::standard::Failure;

/// The point-to-elementary-curve extrema of one dimension, as Extrema_GGExtPC calls
/// them: one `Perform` per conic and the result accessors;
/// `extrema.perform_line(point, &line, 1e-7, 0.0, 1.0)?`.
pub trait ElementaryPointCurve<T: ExtremaCurveTool + ?Sized> {
    /// `Perform(P, gp_Lin, Tol, Uinf, Usup)`.
    fn perform_line(
        &mut self,
        point: T::Point,
        line: &T::Line,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure>;
    /// `Perform(P, gp_Circ, Tol, Uinf, Usup)`.
    fn perform_circle(
        &mut self,
        point: T::Point,
        circle: &T::Circle,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure>;
    /// `Perform(P, gp_Elips, Tol, Uinf, Usup)`.
    fn perform_ellipse(
        &mut self,
        point: T::Point,
        ellipse: &T::Ellipse,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure>;
    /// `Perform(P, gp_Hypr, Tol, Uinf, Usup)`.
    fn perform_hyperbola(
        &mut self,
        point: T::Point,
        hyperbola: &T::Hyperbola,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure>;
    /// `Perform(P, gp_Parab, Tol, Uinf, Usup)`.
    fn perform_parabola(
        &mut self,
        point: T::Point,
        parabola: &T::Parabola,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure>;
    /// The shared result.
    fn point_extrema(&self) -> &PointCurveExtrema<T::Point>;
}

/// The result of Extrema_ExtPElC and Extrema_ExtPElC2d: at most four extrema with
/// their square distance, minimum flag and point.
#[derive(Clone, Debug)]
pub struct PointCurveExtrema<P> {
    class_name: &'static str,
    pub(super) done: bool,
    pub(super) nb_ext: i32,
    pub(super) sq_dist: [f64; 4],
    pub(super) is_min: [bool; 4],
    pub(super) points: [POnCurve<P>; 4],
}

impl<P: Copy + Default> PointCurveExtrema<P> {
    /// The default constructor: not done, square distances at RealLast().
    pub(super) fn new(class_name: &'static str) -> Self {
        Self {
            class_name,
            done: false,
            nb_ext: 0,
            sq_dist: [f64::MAX; 4],
            is_min: [false; 4],
            points: [POnCurve::default(); 4],
        }
    }
    /// Appends one extremum at `myNbExt` and increments it.
    pub(super) fn push(&mut self, sq_dist: f64, is_min: bool, point: POnCurve<P>) {
        let index: usize = self.nb_ext as usize;
        self.sq_dist[index] = sq_dist;
        self.is_min[index] = is_min;
        self.points[index] = point;
        self.nb_ext += 1;
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done(&format!("{}::NbExt()", self.class_name)));
        }
        Ok(self.nb_ext)
    }
    /// `SquareDistance(N)`; Standard_OutOfRange outside `[1, NbExt()]`;
    /// `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        Ok(self.sq_dist[self.index("SquareDistance", n)?])
    }
    /// `IsMin(N)`; `extrema.is_min(1)?`.
    pub fn is_min(&self, n: i32) -> Result<bool, Failure> {
        Ok(self.is_min[self.index("IsMin", n)?])
    }
    /// `Point(N)`; `extrema.point(1)?.parameter()`.
    pub fn point(&self, n: i32) -> Result<POnCurve<P>, Failure> {
        Ok(self.points[self.index("Point", n)?])
    }
    /// The hyperbola and parabola loops: appends each parameter of `[uinf, usup]`
    /// whose curve point is not within `sq_tol` (square distance) of a point already
    /// kept (the native `TbExt` array); `is_min` gets the parameter and the square
    /// distance.
    pub(super) fn push_distinct(
        &mut self,
        point: P,
        parameters: &[f64],
        (uinf, usup): (f64, f64),
        sq_tol: f64,
        curve_value: impl Fn(f64) -> P,
        is_min: impl Fn(f64, f64) -> bool,
    ) where
        P: GpPoint,
    {
        for &us in parameters {
            if !((us >= uinf) && (us <= usup)) {
                continue;
            }
            let cu: P = curve_value(us);
            let kept = &self.points[..self.nb_ext as usize];
            if kept
                .iter()
                .any(|p| p.value().square_distance_to(&cu) < sq_tol)
            {
                continue;
            }
            let sq_dist: f64 = cu.square_distance_to(&point);
            self.push(sq_dist, is_min(us, sq_dist), POnCurve::new(us, cu));
        }
    }
    fn index(&self, member: &str, n: i32) -> Result<usize, Failure> {
        let name: String = format!("{}::{member}()", self.class_name);
        extremum_index(&name, n, || self.nb_ext())
    }
}
