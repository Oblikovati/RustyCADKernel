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
// Rust translation of IntCurve_Polygon2dGen.gxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurve_Polygon2dGen`: the polygon of a parametric curve that the polygon
//! interference of IntCurve_IntPolyPolyGen reads.
use crate::int_imp_par_gen::ParTool;
use crate::int_res2d::Domain;
use crate::intf::Polygon2d;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};
use std::marker::PhantomData;

/// `MAJORATION_DEFLECTION`: the factor applied to the measured deflection.
const MAJORATION_DEFLECTION: f64 = 1.5;

/// `IntCurve_Polygon2dGen` (`Geom2dInt_ThePolygon2dOfTheIntPCurvePCurveOfGInter`):
/// points of constant parameter step on a domain of the curve, with the maximal
/// deflection of the chords; an `Intf_Polygon2d` through [`Polygon2d`]. The point,
/// parameter and index arrays have twice the sample count so that
/// [`Polygon2dGen::compute_with_box`] can refine the chords near another box. Array
/// accesses out of range return the native Standard_OutOfRange.
/// `ThePolygon2dOfTheIntPCurvePCurveOfGInter::new(&adaptor, 20, &domain, 1e-7)?`.
pub struct Polygon2dGen<T: ParTool + ?Sized> {
    my_box: BoundingBox2,
    the_deflection: f64,
    nb_pnt_in: i32,
    the_max_nb_points: i32,
    the_pnts: Array1<'static, Point2>,
    the_params: Array1<'static, f64>,
    the_index: Array1<'static, i32>,
    closed_polygon: bool,
    binf: f64,
    bsup: f64,
    tool: PhantomData<fn() -> Box<T>>,
}

// The chord deflection of `pm` from the line `p1`-`p2`, 0 when the chord is shorter
// than 1e-12 in the L1 norm (the native test before building the gp_Lin2d).
fn chord_deflection(p1: Point2, p2: Point2, pm: Point2) -> Result<Option<f64>, Failure> {
    let mut dx: f64 = p1.x() - p2.x();
    if dx < 0.0 {
        dx = -dx;
    }
    let mut dy: f64 = p1.y() - p2.y();
    if dy < 0.0 {
        dy = -dy;
    }
    if dx + dy > 1e-12 {
        let l = Line2::new(p1, Direction2::from_vector(Vector2::between(p1, p2))?);
        return Ok(Some(l.distance(pm)));
    }
    Ok(None)
}

fn out_of_range(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    error.into_failure("IntCurve_Polygon2dGen")
}

impl<T: ParTool + ?Sized> Polygon2dGen<T> {
    /// `IntCurve_Polygon2dGen(C, NbPnt, D, Tol)`: `max(NbPnt, 3)` points of constant
    /// step on the domain (Standard_DomainError when it is not bounded), the
    /// deflection starting at `min(1e-9, Tol / 100)`, the box enlarged by 1.5 times
    /// the deflection. `Polygon2dGen::<Geom2dCurveTool>::new(&adaptor, 20, &domain, 1e-7)?`.
    pub fn new(c: &T::Curve, t_nb_pts: i32, d: &Domain, tol: f64) -> Result<Self, Failure> {
        let size: i32 = if t_nb_pts < 3 { 6 } else { t_nb_pts + t_nb_pts };
        let nb_pts: i32 = if t_nb_pts < 3 { 3 } else { t_nb_pts };
        let mut poly = Self {
            my_box: BoundingBox2::default(),
            the_deflection: 0.0,
            nb_pnt_in: nb_pts,
            the_max_nb_points: nb_pts + nb_pts,
            the_pnts: Array1::with_bounds(1, size).map_err(out_of_range)?,
            the_params: Array1::with_bounds(1, size).map_err(out_of_range)?,
            the_index: Array1::with_bounds(1, size).map_err(out_of_range)?,
            closed_polygon: false,
            binf: d.first_parameter()?,
            bsup: d.last_parameter()?,
            tool: PhantomData,
        };
        let du: f64 = poly.sample(c, nb_pts)?;
        poly.measure_deflection(c, d, nb_pts, du, tol)?;
        poly.my_box
            .enlarge(poly.the_deflection * MAJORATION_DEFLECTION);
        poly.closed_polygon = false;
        Ok(poly)
    }

    // The points of constant step on [Binf, Bsup]; returns the step.
    fn sample(&mut self, c: &T::Curve, nb_pts: i32) -> Result<f64, Failure> {
        let mut u: f64 = self.binf;
        let u1: f64 = self.bsup;
        let du: f64 = (u1 - u) / f64::from(nb_pts - 1);
        for i in 1..=nb_pts {
            let p: Point2 = T::value(c, u)?;
            self.my_box.add_point(p);
            self.the_index.set_value(i, i).map_err(out_of_range)?;
            self.the_pnts.set_value(i, p).map_err(out_of_range)?;
            self.the_params.set_value(i, u).map_err(out_of_range)?;
            u += du;
        }
        Ok(du)
    }

    // The maximal distance of the mid-parameter points from their chords.
    fn measure_deflection(
        &mut self,
        c: &T::Curve,
        d: &Domain,
        nb_pts: i32,
        du: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let tol100: f64 = tol / 100.0;
        // std::min(1e-9, Tol / 100.) keeps 1e-9 unless Tol / 100 is smaller.
        self.the_deflection = if tol100 < 0.000000001 {
            tol100
        } else {
            0.000000001
        };
        let mut u: f64 = d.first_parameter()?;
        u += du * 0.5;
        let mut i: i32 = 1;
        loop {
            let pm: Point2 = T::value(c, u)?;
            let p1: Point2 = self.pnt(i)?;
            let p2: Point2 = self.pnt(i + 1)?;
            u += du;
            i += 1;
            if let Some(t) = chord_deflection(p1, p2, pm)?
                && t > self.the_deflection
            {
                self.the_deflection = t;
            }
            if i >= nb_pts {
                return Ok(());
            }
        }
    }

    fn pnt(&self, i: i32) -> Result<Point2, Failure> {
        self.the_pnts.value(i).copied().map_err(out_of_range)
    }
    fn param(&self, i: i32) -> Result<f64, Failure> {
        self.the_params.value(i).copied().map_err(out_of_range)
    }
    fn index(&self, i: i32) -> Result<i32, Failure> {
        self.the_index.value(i).copied().map_err(out_of_range)
    }
    fn set_index(&mut self, i: i32, value: i32) -> Result<(), Failure> {
        self.the_index.set_value(i, value).map_err(out_of_range)
    }

    /// `ComputeWithBox(C, OtherBox)`: keeps only the chords that cross the other box
    /// (enlarged by the deflection), splitting those whose middle point deviates by
    /// more than half the deflection, for up to three passes; a polygon outside the
    /// box keeps two points and a void box.
    /// `poly.compute_with_box(&adaptor, other.bounding())?`.
    pub fn compute_with_box(
        &mut self,
        c: &T::Curve,
        box_other_polygon: &BoundingBox2,
    ) -> Result<(), Failure> {
        if self.my_box.is_out_box(box_other_polygon) {
            self.nb_pnt_in = 2;
            self.my_box.set_void();
        } else {
            let [mut bx0, mut by0, mut bx1, mut by1] = box_other_polygon.get()?;
            bx0 -= self.the_deflection;
            by0 -= self.the_deflection;
            bx1 += self.the_deflection;
            by1 += self.the_deflection;
            let bounds = [bx0, bx1, by0, by1];
            let (nbp, max_index_used) = self.keep_crossing_chords(bounds)?;
            if nbp == 1 {
                self.nb_pnt_in = 2;
                self.my_box.set_void();
            } else {
                self.refine_chords(c, bounds, nbp, max_index_used)?;
            }
            self.the_deflection *= MAJORATION_DEFLECTION;
            self.my_box.enlarge(self.the_deflection);
        }
        self.closed_polygon = false;
        self.dump();
        Ok(())
    }

    // The first pass of ComputeWithBox: the chords whose ends are not in a common
    // outer region of the box; returns their point count and the largest index.
    fn keep_crossing_chords(
        &mut self,
        [bx0, bx1, by0, by1]: [f64; 4],
    ) -> Result<(i32, i32), Failure> {
        let mut max_index_used: i32 = 1;
        let mut nbp: i32 = 0;
        let first: Point2 = self.pnt(self.index(1)?)?;
        let mut rprec: i32 = self.calcul_region(first.x(), first.y(), bx0, bx1, by0, by1);
        for i in 2..=self.nb_pnt_in {
            let p2d: Point2 = self.pnt(self.index(i)?)?;
            let ri: i32 = self.calcul_region(p2d.x(), p2d.y(), bx0, bx1, by0, by1);
            if (ri & rprec) == 0 {
                let previous: i32 = self.index(i - 1)?;
                if nbp == 0 || self.index(nbp)? != previous {
                    nbp += 1;
                    self.set_index(nbp, previous)?;
                }
                nbp += 1;
                let current: i32 = self.index(i)?;
                self.set_index(nbp, current)?;
                if current > max_index_used {
                    max_index_used = current;
                }
            }
            rprec = ri;
        }
        Ok((nbp, max_index_used))
    }

    // The deflection passes of ComputeWithBox over the kept chords.
    fn refine_chords(
        &mut self,
        c: &T::Curve,
        bounds: [f64; 4],
        mut nbp: i32,
        mut max_index_used: i32,
    ) -> Result<(), Failure> {
        self.my_box.set_void();
        if nbp != 0 {
            let first: Point2 = self.pnt(self.index(1)?)?;
            self.my_box.add_point(first);
        }
        let mut nbpassagedeflection: i32 = 0;
        loop {
            nbpassagedeflection += 1;
            let mut new_deflection: f64 = self.the_deflection;
            let mut i: i32 = 2;
            while i <= nbp {
                if self.split_chord(
                    c,
                    bounds,
                    i,
                    &mut nbp,
                    &mut max_index_used,
                    &mut new_deflection,
                )? {
                    i -= 1;
                }
                i += 1;
            }
            // `if (NewDeflection)`: any non-zero value, NaN included.
            let ratio_deflection: f64 = if new_deflection != 0.0 {
                self.the_deflection / new_deflection
            } else {
                10.0
            };
            self.the_deflection = new_deflection;
            self.nb_pnt_in = nbp;
            if !((ratio_deflection < 3.0)
                && (nbpassagedeflection < 3)
                && (max_index_used < (self.the_max_nb_points - 2)))
            {
                return Ok(());
            }
        }
    }

    // One chord of a deflection pass: adds its end to the box and, when it crosses
    // the box and deviates by more than half the deflection, inserts its middle point
    // (true: the chord is visited again).
    fn split_chord(
        &mut self,
        c: &T::Curve,
        [bx0, bx1, by0, by1]: [f64; 4],
        i: i32,
        nbp: &mut i32,
        max_index_used: &mut i32,
        new_deflection: &mut f64,
    ) -> Result<bool, Failure> {
        let ii: i32 = self.index(i)?;
        let iim1: i32 = self.index(i - 1)?;
        let pi: Point2 = self.pnt(ii)?;
        let pim1: Point2 = self.pnt(iim1)?;
        self.my_box.add_point(pi);
        let regi: i32 = self.calcul_region(pi.x(), pi.y(), bx0, bx1, by0, by1);
        let regim1: i32 = self.calcul_region(pim1.x(), pim1.y(), bx0, bx1, by0, by1);
        if (regi & regim1) != 0 {
            return Ok(false);
        }
        let u: f64 = 0.5 * (self.param(ii)? + self.param(iim1)?);
        let pm: Point2 = T::value(c, u)?;
        let Some(t) = chord_deflection(pim1, pi, pm)? else {
            // A null chord: the native `t` stays 0.
            if 0.0 > *new_deflection {
                *new_deflection = 0.0;
            }
            return Ok(false);
        };
        if !((*max_index_used < (self.the_max_nb_points - 1)) && (t > (self.the_deflection * 0.5)))
        {
            return Ok(false);
        }
        *nbp += 1;
        let mut j: i32 = *nbp;
        while j > i {
            let moved: i32 = self.index(j - 1)?;
            self.set_index(j, moved)?;
            j -= 1;
        }
        *max_index_used += 1;
        self.set_index(i, *max_index_used)?;
        self.the_pnts
            .set_value(*max_index_used, pm)
            .map_err(out_of_range)?;
        self.the_params
            .set_value(*max_index_used, u)
            .map_err(out_of_range)?;
        // The native code evaluates the curve at the quarter parameter and the
        // distance to the new chord, and discards both.
        let u1m: f64 = 0.5 * (u + self.param(self.index(i - 1)?)?);
        let p1m: Point2 = T::value(c, u1m)?;
        let _ =
            Line2::new(pim1, Direction2::from_vector(Vector2::between(pim1, pm))?).distance(p1m);
        Ok(true)
    }

    /// `DeflectionOverEstimation()`. `poly.deflection_over_estimation()`.
    pub fn deflection_over_estimation(&self) -> f64 {
        self.the_deflection
    }

    /// `SetDeflectionOverEstimation(x)`: also enlarges the box by `x`.
    /// `poly.set_deflection_over_estimation(1e-7)`.
    pub fn set_deflection_over_estimation(&mut self, x: f64) {
        self.the_deflection = x;
        self.my_box.enlarge(self.the_deflection);
    }

    /// `Closed(clos)`: whether the polygon is closed. `poly.set_closed(true)`.
    pub fn set_closed(&mut self, flag: bool) {
        self.closed_polygon = flag;
    }

    /// `Closed()`. `poly.closed()`.
    pub fn closed(&self) -> bool {
        self.closed_polygon
    }

    /// `NbSegments()`: the point count, less one when the polygon is open.
    /// `poly.nb_segments()`.
    pub fn nb_segments(&self) -> i32 {
        if self.closed_polygon {
            self.nb_pnt_in
        } else {
            self.nb_pnt_in - 1
        }
    }

    /// `Segment(theIndex, theBegin, theEnd)`: the ends of segment `index`; past the
    /// last point Standard_OutOfRange unless the polygon is closed.
    /// `let (begin, end) = poly.segment(1)?;`.
    pub fn segment(&self, the_index: i32) -> Result<(Point2, Point2), Failure> {
        let mut ind: i32 = the_index;
        let the_begin: Point2 = self.pnt(self.index(the_index)?)?;
        if the_index >= self.nb_pnt_in {
            if !self.closed_polygon {
                return Err(Failure::with_type(
                    "Standard_OutOfRange",
                    "IntCurve_Polygon2dGen::Segment!",
                    "",
                ));
            }
            ind = 0;
        }
        let the_end: Point2 = self.pnt(self.index(ind + 1)?)?;
        Ok((the_begin, the_end))
    }

    /// `InfParameter()`: the parameter of the first point. `poly.inf_parameter()?`.
    pub fn inf_parameter(&self) -> Result<f64, Failure> {
        self.param(self.index(1)?)
    }

    /// `SupParameter()`: the parameter of the last point. `poly.sup_parameter()?`.
    pub fn sup_parameter(&self) -> Result<f64, Failure> {
        self.param(self.index(self.nb_pnt_in)?)
    }

    /// `AutoIntersectionIsPossible()`: whether a chord turns back against the first
    /// one. `poly.auto_intersection_is_possible()?`.
    pub fn auto_intersection_is_possible(&self) -> Result<bool, Failure> {
        let vref = Vector2::between(self.pnt(self.index(1)?)?, self.pnt(self.index(2)?)?);
        for i in 3..=self.nb_pnt_in {
            let v = Vector2::between(self.pnt(self.index(i - 1)?)?, self.pnt(self.index(i)?)?);
            if v.dot(vref) < 0.0 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `ApproxParamOnCurve(Index, ParamOnLine)`: the curve parameter at `param_on_line`
    /// (0 to 1) along chord `index`, by linear interpolation of the end parameters;
    /// the last point with 0 is the end of the previous chord and index 0 the start
    /// of the first one. `poly.approx_param_on_curve(2, 0.5)?`.
    pub fn approx_param_on_curve(
        &self,
        aindex: i32,
        the_param_on_line: f64,
    ) -> Result<f64, Failure> {
        let mut index: i32 = aindex;
        let mut param_on_line: f64 = the_param_on_line;
        // Natively an index past the last point prints "OutOfRange
        // Polygon2d::ApproxParamOnCurve" on std::cout and goes on.
        if (index == self.nb_pnt_in) && (param_on_line == 0.0) {
            index -= 1;
            param_on_line = 1.0;
        }
        if index == 0 {
            index = 1;
            param_on_line = 0.0;
        }
        let indexp1: i32 = self.index(index + 1)?;
        let index: i32 = self.index(index)?;
        let du: f64 = self.param(indexp1)? - self.param(index)?;
        let u: f64 = self.param(index)? + param_on_line * du;
        Ok(u)
    }

    /// `CalculRegion(x, y, x1, x2, y1, y2)`: the outcode of a point against a box:
    /// 1 left, 2 right, 4 below, 8 above. `poly.calcul_region(0.0, 2.0, -1.0, 1.0, -1.0, 1.0) == 8`.
    pub fn calcul_region(&self, x: f64, y: f64, x1: f64, x2: f64, y1: f64, y2: f64) -> i32 {
        let mut r: i32 = if x < x1 {
            1
        } else if x > x2 {
            2
        } else {
            0
        };
        if y < y1 {
            r |= 4;
        } else if y > y2 {
            r |= 8;
        }
        r
    }

    /// `Dump()`: prints nothing, as the native `debug` flag is 0. `poly.dump()`.
    pub fn dump(&self) {}
}

impl<T: ParTool + ?Sized> Polygon2d for Polygon2dGen<T> {
    fn bounding(&self) -> &BoundingBox2 {
        &self.my_box
    }
    fn closed(&self) -> bool {
        self.closed_polygon
    }
    fn deflection_over_estimation(&self) -> f64 {
        self.the_deflection
    }
    fn nb_segments(&self) -> i32 {
        Polygon2dGen::nb_segments(self)
    }
    fn segment(&self, index: i32) -> Result<(Point2, Point2), Failure> {
        Polygon2dGen::segment(self, index)
    }
}
