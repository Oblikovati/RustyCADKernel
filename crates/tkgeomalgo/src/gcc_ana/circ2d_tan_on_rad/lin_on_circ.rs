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
// Rust translation of GccAna_Circ2dTanOnRad_4.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a line with their centre on a circle: the
//! intersections of the locus with the parallels to the line at the radius.
//!
//! Native quirk kept: `sign = (int)dc1.Dot(normL1)` truncates the side test to an
//! integer (0 unless the dot product is exactly +-1) and the loop over the
//! parallels reuses that variable, so it drives both the next parallel and the
//! tangency point.
use super::{Circ2dTanOnRad, NAME};
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Qualified1, OnCirc, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified line, centred on `on_circ`;
    /// `Circ2dTanOnRad::lin_on_circ(l1, circle, radius, tol)?`.
    pub fn lin_on_circ(
        qualified1: QualifiedLin,
        on_circ: Circle2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let tol: f64 = tolerance.abs();
        if !(qualified1.is_enclosed() || qualified1.is_outside() || qualified1.is_unqualified()) {
            return Err(bad_qualifier(&format!(
                "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
                qualified1.qualifier()
            )));
        }
        let l1: Line2 = qualified1.qualified();
        let dist1: f64 = l1.distance(on_circ.location()) - on_circ.radius();
        let dist2: f64 = l1.distance(on_circ.location()) + on_circ.radius();
        Self::check_radius(radius)?;
        if (dist1 - radius > tol) || (tol < radius - dist2) {
            circ.well_done = true;
            return Ok(circ);
        }
        let (nbsol, mut sign): (i32, i32) = if qualified1.is_enclosed() {
            (1, -1)
        } else if qualified1.is_outside() {
            (1, 1)
        } else {
            (2, 1)
        };
        let dir1: Direction2 = l1.direction();
        let origin1: Point2 = l1.location();
        for _ in 1..=nbsol {
            sign = -sign;
            let s: f64 = f64::from(sign);
            let parallel = Line2::new(
                Point2::new(
                    origin1.x() - s * radius * dir1.y(),
                    origin1.y() + s * radius * dir1.x(),
                ),
                dir1,
            );
            let intp = AnaIntersection::line_circle(parallel, on_circ)?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for i in 1..=intp.nb_points()? {
                    sign = circ.parallel_on_circle(
                        qualified1,
                        on_circ,
                        radius,
                        intp.point(i)?.value(),
                    )?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    // One solution; returns the truncated side test that native stores in `sign`.
    fn parallel_on_circle(
        &mut self,
        qualified1: QualifiedLin,
        on_circ: Circle2,
        radius: f64,
        center: Point2,
    ) -> Result<i32, Failure> {
        let l1: Line2 = qualified1.qualified();
        let dir1: Direction2 = l1.direction();
        let norm_l1 = Direction2::new(-dir1.y(), dir1.x())?;
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        // (int) truncation toward zero, as the native cast.
        let sign: i32 = dc1.dot(norm_l1) as i32;
        let position = if !qualified1.is_unqualified() {
            qualified1.qualifier()
        } else if dc1.dot(norm_l1) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.qualifier1.set(index, position)?;
        let centre: Point2 = solution.location();
        // `-sign` is the integer negation natively: for sign 0 it is +0, not -0.
        let (s, minus_s): (f64, f64) = (f64::from(sign), f64::from(-sign));
        let point =
            Point2::from(centre.xy() + Xy::new(s * radius * dir1.y(), minus_s * radius * dir1.x()));
        let par_arg: f64 = elclib::parameter(&l1, point)?;
        let par_sol: f64 = elclib::parameter(&solution, point)?;
        self.tangency1.record(index, point, par_sol, par_arg)?;
        self.centre_on(index, centre, elclib::parameter(&on_circ, centre)?)?;
        Ok(sign)
    }
}
