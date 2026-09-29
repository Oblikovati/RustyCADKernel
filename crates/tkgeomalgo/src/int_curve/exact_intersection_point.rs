// Copyright (c) 1999-1999 Matra Datavision
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
// Rust translation of IntCurve_ExactIntersectionPoint.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_ExactIntersectionPoint`: the Newton refinement of a polygon
//! intersection point into a common point of two parametric curves.
use super::{DistBetweenPCurvesGen, PCurveTool, Polygon2dGen};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{FunctionSet, FunctionSetRoot, Vector};

/// `IntCurve_ExactIntersectionPoint`
/// (`Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter`): solves
/// `C1(u) = C2(v)` by math_FunctionSetRoot (60 iterations, the EpsX of each curve as
/// tolerance) inside bounds taken from the polygons, and keeps the root when the
/// residual is at most the tolerance. The error flag stays set once a solve fails.
/// `let mut eip = ExactIntersectionPointOfTheIntPCurvePCurveOfGInter::new(&a1, &a2, 1e-7)?;`.
pub struct ExactIntersectionPoint<'a, T: PCurveTool> {
    done: bool,
    nbroots: i32,
    my_tol: f64,
    fct_dist: DistBetweenPCurvesGen<'a, T>,
    tolerance_vector: Vector<'static>,
    binf_vector: Vector<'static>,
    bsup_vector: Vector<'static>,
    starting_point: Vector<'static>,
    root: Vector<'static>,
    an_error_occurred: bool,
}

/// The chord of a polygon interference point, rewritten by
/// [`ExactIntersectionPoint::perform_polygons`] as the native in-out arguments:
/// `(segment index, parameter on the segment)`.
pub type PolygonSeat = (i32, f64);

impl<'a, T: PCurveTool> ExactIntersectionPoint<'a, T> {
    /// `IntCurve_ExactIntersectionPoint(C1, C2, Tol)`: the square of `tol` bounds the
    /// residual of a root. `ExactIntersectionPoint::<Geom2dCurveTool>::new(&a1, &a2, 1e-7)?`.
    pub fn new(c1: &'a T::Curve, c2: &'a T::Curve, tol: f64) -> Result<Self, Failure> {
        let mut tolerance_vector: Vector<'static> = Vector::with_bounds(1, 2)?;
        tolerance_vector[1] = T::eps_x(c1);
        tolerance_vector[2] = T::eps_x(c2);
        Ok(Self {
            done: false,
            nbroots: 0,
            my_tol: tol * tol,
            fct_dist: DistBetweenPCurvesGen::new(c1, c2),
            tolerance_vector,
            binf_vector: Vector::with_bounds(1, 2)?,
            bsup_vector: Vector::with_bounds(1, 2)?,
            starting_point: Vector::with_bounds(1, 2)?,
            root: Vector::with_bounds(1, 2)?,
            an_error_occurred: false,
        })
    }

    /// `Perform(Poly1, Poly2, NumSegOn1, NumSegOn2, ParamOnSeg1, ParamOnSeg2)`: starts
    /// at the polygon point, searches from the chord before to the chord after next,
    /// then widens the bounds on the first curve and on the second one, left and
    /// right, while no root is found. A seat at the end of a polygon is moved onto
    /// its last chord and a seat before the first chord onto the first one, in place.
    /// `eip.perform_polygons(&p1, &p2, &mut (3, 0.5), &mut (4, 0.25))?`.
    pub fn perform_polygons(
        &mut self,
        poly1: &Polygon2dGen<T>,
        poly2: &Polygon2dGen<T>,
        seat1: &mut PolygonSeat,
        seat2: &mut PolygonSeat,
    ) -> Result<(), Failure> {
        normalize_seat(poly1, seat1);
        normalize_seat(poly2, seat2);
        self.set_search_range(1, poly1, *seat1)?;
        self.set_search_range(2, poly2, *seat2)?;
        self.math_perform()?;
        if self.nbroots != 0 {
            return Ok(());
        }
        // The native code reads both deflections here and discards them.
        self.widen(1, poly1, seat1.0)?;
        if self.nbroots == 0 {
            self.widen(2, poly2, seat2.0)?;
        }
        Ok(())
    }

    // The start point and the bounds of variable `k` from its polygon seat.
    fn set_search_range(
        &mut self,
        k: i32,
        poly: &Polygon2dGen<T>,
        (num_seg, param_on_seg): PolygonSeat,
    ) -> Result<(), Failure> {
        self.starting_point[k] = poly.approx_param_on_curve(num_seg, param_on_seg)?;
        self.binf_vector[k] = if num_seg <= 2 {
            poly.inf_parameter()?
        } else {
            poly.approx_param_on_curve(num_seg - 1, 0.0)?
        };
        self.bsup_vector[k] = if num_seg >= (poly.nb_segments() - 2) {
            poly.sup_parameter()?
        } else {
            poly.approx_param_on_curve(num_seg + 2, 0.0)?
        };
        Ok(())
    }

    // The two widening loops of variable `k` ("On elargit les bornes par la gauche",
    // then "par la droite"): the upper bound of `k` is restored at the end, the lower
    // one only before the right loop (a root found on the left keeps it widened).
    fn widen(&mut self, k: i32, poly: &Polygon2dGen<T>, num_seg: i32) -> Result<(), Failure> {
        let an_binf_vector: f64 = self.binf_vector[k];
        let an_bsup_vector: f64 = self.bsup_vector[k];
        let mut diff: i32 = 1;
        loop {
            diff += 1;
            if (num_seg - diff) <= 1 {
                self.binf_vector[k] = poly.inf_parameter()?;
                diff = 0;
            } else {
                self.binf_vector[k] = poly.approx_param_on_curve(num_seg - diff, 0.0)?;
            }
            self.math_perform()?;
            //-- le 18 nov 97
            if diff > 3 {
                diff += num_seg / 2;
            }
            if !(self.nbroots == 0 && diff != 0) {
                break;
            }
        }
        if self.nbroots == 0 {
            self.binf_vector[k] = an_binf_vector;
            diff = 1;
            loop {
                diff += 1;
                if (num_seg + diff) >= (poly.nb_segments() - 1) {
                    self.bsup_vector[k] = poly.sup_parameter()?;
                    diff = 0;
                } else {
                    self.bsup_vector[k] = poly.approx_param_on_curve(num_seg + 1 + diff, 0.0)?;
                }
                self.math_perform()?;
                //-- le 18 nov 97
                if diff > 3 {
                    diff += 1 + (poly.nb_segments() - num_seg) / 2;
                }
                if !(self.nbroots == 0 && diff != 0) {
                    break;
                }
            }
        }
        self.bsup_vector[k] = an_bsup_vector;
        Ok(())
    }

    /// `Perform(Uo, Vo, UInf, VInf, USup, VSup)`: one solve from `(uo, vo)` inside
    /// the given bounds. `eip.perform(0.5, 0.5, 0.0, 0.0, 1.0, 1.0)?`.
    pub fn perform(
        &mut self,
        uo: f64,
        vo: f64,
        u_inf: f64,
        v_inf: f64,
        u_sup: f64,
        v_sup: f64,
    ) -> Result<(), Failure> {
        self.done = true;
        self.binf_vector[1] = u_inf;
        self.binf_vector[2] = v_inf;
        self.bsup_vector[1] = u_sup;
        self.bsup_vector[2] = v_sup;
        self.starting_point[1] = uo;
        self.starting_point[2] = vo;
        self.math_perform()
    }

    /// `NbRoots()`: 1 when the last solve found a root, 0 otherwise. `eip.nb_roots()`.
    pub fn nb_roots(&self) -> i32 {
        self.nbroots
    }

    /// `Roots(U, V)`: the last root found (kept when a later solve fails).
    /// `let (u, v) = eip.roots();`.
    pub fn roots(&self) -> (f64, f64) {
        (self.root[1], self.root[2])
    }

    /// `AnErrorOccurred()`: whether a solve did not converge since construction.
    /// `eip.an_error_occurred()`.
    pub fn an_error_occurred(&self) -> bool {
        self.an_error_occurred
    }

    // `MathPerform()`: the solve; a curve exception inside it is returned.
    fn math_perform(&mut self) -> Result<(), Failure> {
        let mut fct = FunctionSetRoot::new(&self.fct_dist, &self.tolerance_vector, 60)?;
        let solved = fct.perform_bounded(
            &mut self.fct_dist,
            &self.starting_point,
            &self.binf_vector,
            &self.bsup_vector,
            false,
        );
        if let Some(failure) = self.fct_dist.take_failure() {
            return Err(failure);
        }
        solved?;
        if !fct.is_done() {
            self.an_error_occurred = true;
            self.nbroots = 0;
            return Ok(());
        }
        fct.copy_root(&mut self.root)?;
        self.nbroots = 1;
        let mut xy: Vector<'static> = Vector::with_bounds(1, 2)?;
        self.fct_dist.value(&self.root, &mut xy);
        if let Some(failure) = self.fct_dist.take_failure() {
            return Err(failure);
        }
        let dist2: f64 = (xy[1] * xy[1]) + (xy[2] * xy[2]);
        if dist2 > self.my_tol {
            self.nbroots = 0;
        }
        Ok(())
    }
}

// The seat corrections of Perform(Poly1, Poly2, ...): the start of the chord after
// the last one is the end of the last chord, and an index before the first chord
// is the start of the first one.
fn normalize_seat<T: PCurveTool>(poly: &Polygon2dGen<T>, seat: &mut PolygonSeat) {
    if seat.0 >= poly.nb_segments() && seat.1 == 0.0 {
        seat.0 -= 1;
        seat.1 = 1.0;
    }
    if seat.0 <= 0 {
        seat.0 = 1;
        seat.1 = 0.0;
    }
}
