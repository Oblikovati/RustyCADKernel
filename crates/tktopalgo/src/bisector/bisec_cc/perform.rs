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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (Perform and the polygon of the bisector) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the bisector of two curves: its first point from the
//! origin, the polygon that samples it, then its extensions.
use super::state::{CcState, OnCurves};
use crate::bisector::bisec_pc::copy_curve;
use crate::bisector::{PointOnBis, is_convex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Point2;

// The number of samples of the polygon.
const NB_PNTS: i32 = 21;

impl CcState {
    /// `Perform(Cu1, Cu2, Side1, Side2, Origin, DistMax)` on the current fields (the
    /// intervals and the polygon are appended to, as natively).
    pub(crate) fn perform(
        &mut self,
        curves: (&Handle<dyn Geom2dCurve>, &Handle<dyn Geom2dCurve>),
        sides: (f64, f64),
        origin: Point2,
        dist_max: f64,
    ) -> Result<(), Failure> {
        self.is_empty = false;
        self.dist_max = dist_max;
        self.curve1 = copy_curve(curves.0)?;
        self.curve2 = copy_curve(curves.1)?;
        (self.sign1, self.sign2) = sides;
        self.is_convex1 = is_convex(&self.curve1, self.sign1)?;
        self.is_convex2 = is_convex(&self.curve2, self.sign2)?;
        let Some(u) = super::proj::proj_on_curve(origin, &self.curve1)? else {
            self.is_empty = true;
            return Ok(());
        };
        let mut on_curves = OnCurves::default();
        let Some((ori_in_poly, first_point)) = self.first_point(u, origin, &mut on_curves)? else {
            self.is_empty = true;
            return Ok(());
        };
        let ya_poly: bool = self.my_polygon.length() != 0;
        if ya_poly {
            self.sup_last_parameter()?;
            self.sample_polygon(first_point, &mut on_curves)?;
        }
        self.extensions(origin, ya_poly, ori_in_poly)?;
        if !ya_poly && !self.extension_start && !self.extension_end {
            self.is_empty = true;
        }
        // NIZHNY-EAP, Jan 2000: a polygon of two points or less is empty.
        if self.my_polygon.length() <= 2 {
            self.is_empty = true;
        }
        Ok(())
    }

    // The first point of the polygon: the point of the origin's parameter, or the
    // first valid point found by dichotomy when the origin is on the extension.
    // `None` when the point is on the curves but farther than the origin; else
    // whether the origin is the first point, and the last point computed.
    fn first_point(
        &mut self,
        mut u: f64,
        origin: Point2,
        on_curves: &mut OnCurves,
    ) -> Result<Option<(bool, Point2)>, Failure> {
        let mut p: Point2 = self.value_by_int(u, on_curves)?;
        if on_curves.dist < precision::confusion() && self.is_farther(p, origin, on_curves)? {
            return Ok(None);
        }
        if on_curves.dist < precision::infinite() {
            self.my_polygon.append(PointOnBis::new(
                on_curves.u1,
                on_curves.u2,
                u,
                on_curves.dist,
                p,
            ))?;
            self.start_intervals.append(u);
            return Ok(Some((p.is_equal(origin, precision::confusion()), p)));
        }
        let du: f64 = (self.curve1.last_parameter() - u) / f64::from(NB_PNTS - 1);
        u += du;
        for _ in 1..NB_PNTS {
            p = self.value_by_int(u, on_curves)?;
            if on_curves.dist < precision::infinite() {
                let u_sol: f64 = self.search_bound(u - du, u)?;
                p = self.value_by_int(u_sol, on_curves)?;
                self.start_intervals.append(u_sol);
                self.my_polygon.append(PointOnBis::new(
                    on_curves.u1,
                    on_curves.u2,
                    u_sol,
                    on_curves.dist,
                    p,
                ))?;
                break;
            }
            u += du;
        }
        Ok(Some((false, p)))
    }

    // True when the curve points of a point on the curves are nearer to the origin.
    fn is_farther(&self, p: Point2, origin: Point2, on_curves: &OnCurves) -> Result<bool, Failure> {
        let a_p1: Point2 = self.curve1.value(on_curves.u1)?;
        let a_p2: Point2 = self.curve2.value(on_curves.u2)?;
        let dp: f64 = a_p1.distance(p) + a_p2.distance(p);
        let dorig: f64 = a_p1.distance(origin) + a_p2.distance(origin);
        Ok(dp < dorig)
    }

    // The samples of the polygon from its first point (NIZHNY-EAP, Jan 2000: a
    // point at the same place as the previous one is not added).
    #[allow(clippy::neg_cmp_op_on_partial_ord)] // The negated native test keeps its NaN answer.
    fn sample_polygon(
        &mut self,
        first_point: Point2,
        on_curves: &mut OnCurves,
    ) -> Result<(), Failure> {
        let mut u: f64 = self.first_parameter()?;
        let span: f64 = self.last_parameter()? - u;
        let nb_pnts: i32 = if span < 10. * precision::confusion() {
            3
        } else {
            NB_PNTS
        };
        let du: f64 = span / f64::from(nb_pnts - 1);
        u += du;
        let mut prev_pnt: Point2 = first_point;
        for _ in 1..nb_pnts {
            let mut p: Point2 = self.value_by_int(u, on_curves)?;
            let mut u_point: f64 = u;
            let bound: bool = !(on_curves.dist < precision::infinite());
            if bound {
                u_point = self.search_bound(u - du, u)?;
                p = self.value_by_int(u_point, on_curves)?;
                self.end_intervals.set_value(1, u_point)?;
            }
            if p.distance(prev_pnt) > precision::confusion() {
                self.my_polygon.append(PointOnBis::new(
                    on_curves.u1,
                    on_curves.u2,
                    u_point,
                    on_curves.dist,
                    p,
                ))?;
            }
            if bound {
                break;
            }
            u += du;
            prev_pnt = p;
        }
        Ok(())
    }

    // The start and end points and the extensions: the curve is extended at a
    // common end where one curve is concave and the curves open by PI; the start
    // extension counts only when the origin is not the first point.
    fn extensions(
        &mut self,
        origin: Point2,
        ya_poly: bool,
        ori_in_poly: bool,
    ) -> Result<(), Failure> {
        self.extension_start = false;
        self.extension_end = false;
        self.point_start = origin;
        if self.is_convex1 && self.is_convex2 {
            if ya_poly {
                self.point_end = self.my_polygon.last().point();
            }
            return Ok(());
        }
        self.extension_start =
            !ori_in_poly && super::proj::test_extension(&self.curve1, &self.curve2, 1)?;
        self.extension_end = super::proj::test_extension(&self.curve1, &self.curve2, 2)?;
        if self.extension_end {
            self.point_end = self.curve1.value(self.curve1.last_parameter())?;
        } else if ya_poly {
            self.point_end = self.my_polygon.last().point();
        } else {
            self.compute_point_end()?;
        }
        self.update_intervals(ya_poly)
    }

    // The limits of the intervals with the extensions; without polygon the
    // bisector is the segment from the start point to the end point.
    fn update_intervals(&mut self, ya_poly: bool) -> Result<(), Failure> {
        if !ya_poly {
            self.start_intervals.append(0.);
            self.end_intervals
                .append(self.point_end.distance(self.point_start));
            return Ok(());
        }
        if self.extension_start {
            let p1: Point2 = self.my_polygon.first().point();
            let u_first: f64 = self.start_intervals.first()? - self.point_start.distance(p1);
            self.start_intervals.prepend(u_first);
            self.end_intervals.prepend(self.start_intervals.value(2)?);
        }
        if self.extension_end {
            let p1: Point2 = self.my_polygon.last().point();
            let u_first: f64 = self.end_intervals.last()?;
            let u_last: f64 = u_first + self.point_end.distance(p1);
            self.start_intervals.append(u_first);
            self.end_intervals.append(u_last);
        }
        Ok(())
    }

    /// `SupLastParameter()`: the end of the bisector, where it meets the normal of
    /// C1 at the parameter of the end of C2 (the one the side chooses).
    pub(crate) fn sup_last_parameter(&mut self) -> Result<(), Failure> {
        self.end_intervals.append(self.curve1.last_parameter());
        let u_on_c2: f64 = if self.sign1 == self.sign2 {
            self.curve2.first_parameter()
        } else {
            self.curve2.last_parameter()
        };
        let start = super::value_by_int::PointByIntResult {
            found: false,
            u_on_b: 0.,
            dist: 0.,
        };
        let result = super::value_by_int::point_by_int(
            &self.curve2,
            &self.curve1,
            (self.sign2, self.sign1),
            u_on_c2,
            start,
        )?;
        if result.found
            && result.u_on_b > self.start_intervals.first()?
            && result.u_on_b < self.end_intervals.last()?
        {
            self.end_intervals.set_value(1, result.u_on_b)?;
        }
        Ok(())
    }

    /// `SearchBound(U1, U2)`: the dichotomy between a parameter with a bisector point
    /// and one without.
    pub(crate) fn search_bound(&self, u1: f64, u2: f64) -> Result<f64, Failure> {
        let (tol_pnt, tol_par) = (precision::confusion(), precision::p_confusion());
        let finite = |d: f64| d < precision::infinite();
        // The parameters on the curves that ValueByInt writes are not read here.
        let mut scratch = OnCurves::default();
        let (mut u11, mut u22) = (u1, u2);
        let (mut pbis_prec, mut dist1) = self.point_and_dist(u11, &mut scratch)?;
        let (mut pbis, mut dist2) = self.point_and_dist(u22, &mut scratch)?;
        while (u22 - u11) > tol_par
            || (finite(dist1) && finite(dist2) && !pbis.is_equal(pbis_prec, tol_pnt))
        {
            pbis_prec = pbis;
            let umid: f64 = 0.5 * (u22 + u11);
            let dist_mid: f64;
            (pbis, dist_mid) = self.point_and_dist(umid, &mut scratch)?;
            if finite(dist1) == finite(dist_mid) {
                u11 = umid;
                dist1 = dist_mid;
            } else {
                u22 = umid;
                dist2 = dist_mid;
            }
        }
        (_, dist1) = self.point_and_dist(u11, &mut scratch)?;
        Ok(if finite(dist1) { u11 } else { u22 })
    }

    // `ValueByInt` with its distance output.
    fn point_and_dist(&self, u: f64, scratch: &mut OnCurves) -> Result<(Point2, f64), Failure> {
        let p: Point2 = self.value_by_int(u, scratch)?;
        Ok((p, scratch.dist))
    }

    /// `ComputePointEnd()`: the end point of a bisector without polygon, at the
    /// radius of curvature of the concave curve from the start of C1.
    pub(crate) fn compute_point_end(&mut self) -> Result<(), Failure> {
        let u1: f64 = self.curve1.first_parameter();
        let u2: f64 = if self.sign1 == self.sign2 {
            self.curve2.last_parameter()
        } else {
            self.curve2.first_parameter()
        };
        let k1: f64 = super::state::curvature(&*self.curve1, u1, precision::confusion())?;
        let k2: f64 = super::state::curvature(&*self.curve2, u2, precision::confusion())?;
        let kc: f64 = if !self.is_convex1 && !self.is_convex2 {
            if k1 < k2 { k1 } else { k2 }
        } else if !self.is_convex1 {
            k1
        } else {
            k2
        };
        let (pf, mut tf) = self.curve1.d1(u1)?;
        tf.normalize()?;
        let rc: f64 = if kc != 0. {
            (1. / kc).abs()
        } else {
            precision::infinite()
        };
        self.point_end = Point2::new(
            pf.x() - self.sign1 * rc * tf.y(),
            pf.y() + self.sign1 * rc * tf.x(),
        );
        Ok(())
    }
}
