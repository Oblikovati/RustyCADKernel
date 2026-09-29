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
// Rust translation of the two-circle constructor of GccAna_Circ2d2TanRad.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to two circles. Each qualifier pair either
//! settles a single tangent configuration or reduces the problem to the
//! intersections of pairs of circles concentric to the arguments (the `C(1..8)`
//! array); the cases live in `circ_circ_cases` and `circ_circ_cases2`.
//!
//! The native state is kept as is: the swapped radii, centres and direction of the
//! argument-exchanging cases stay swapped for the qualifier tests of the
//! intersection solutions, and the second argument used for the final parameters
//! is the variable `C2`, which one case replaces by C1.
use super::{Circ2d2TanRad, NAME};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{circle_at, qualify};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

/// The mutable state of the native constructor.
pub(super) struct Work {
    pub(super) q1: QualifiedCirc,
    pub(super) q2: QualifiedCirc,
    /// C(1..8), the pairs of circles to intersect (index 0..8).
    pub(super) c: [Circle2; 8],
    /// The variable C2, the second argument of the final parameters.
    pub(super) c2: Circle2,
    pub(super) r1: f64,
    pub(super) r2: f64,
    pub(super) dist: f64,
    pub(super) center1: Point2,
    pub(super) center2: Point2,
    pub(super) dir1: Direction2,
    pub(super) invers: bool,
    pub(super) nbsol: usize,
    pub(super) radius: f64,
    pub(super) tol: f64,
}

impl Work {
    /// Exchange the two arguments (C(1), C(2), radii, centres, direction) and ask
    /// for the tangency points to be swapped back at the end.
    pub(super) fn swap_arguments(&mut self) {
        (self.c[0], self.c[1]) = (self.q2.qualified(), self.q1.qualified());
        (self.r1, self.r2) = (self.r2, self.r1);
        (self.center1, self.center2) = (self.center2, self.center1);
        self.dir1.reverse();
        self.invers = true;
    }

    /// C(k) = the circle of `radius` concentric to C(k) (1-based).
    pub(super) fn ring(&mut self, k: usize, centre: Point2, radius: f64) -> Result<(), Failure> {
        self.c[k - 1] = circle_at(centre, radius)?;
        Ok(())
    }

    /// The pairs to intersect, (C(1), C(2)) then (C(3), C(4))...
    pub(super) fn pairs(&mut self, pairs: &[(f64, f64)]) -> Result<(), Failure> {
        let (l1, l2) = (self.c[0].location(), self.c[1].location());
        for (n, &(radius1, radius2)) in pairs.iter().enumerate() {
            self.ring(2 * n + 1, l1, radius1)?;
            self.ring(2 * n + 2, l2, radius2)?;
        }
        self.nbsol = pairs.len();
        Ok(())
    }
}

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Qualified1, Qualified2, Radius, Tolerance): circles of
    /// radius `radius` tangent to two qualified circles, up to 8;
    /// `Circ2d2TanRad::circ_circ(q1, q2, radius, tol)?`.
    pub fn circ_circ(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(8);
        check_qualifier(qualified1.qualifier(), NAME)?;
        check_qualifier(qualified2.qualifier(), NAME)?;
        let (c1, c2) = (qualified1.qualified(), qualified2.qualified());
        let mut work = Work {
            q1: qualified1,
            q2: qualified2,
            c: [
                c1,
                c2,
                Circle2::default(),
                Circle2::default(),
                Circle2::default(),
                Circle2::default(),
                Circle2::default(),
                Circle2::default(),
            ],
            c2,
            r1: c1.radius(),
            r2: c2.radius(),
            dist: 0.0,
            center1: c1.location(),
            center2: c2.location(),
            dir1: Direction2::default(),
            invers: false,
            nbsol: 0,
            radius,
            tol: tolerance.abs(),
        };
        Self::check_radius(radius)?;
        if c2
            .location()
            .is_equal(c1.location(), precision::confusion())
        {
            circ.well_done = true;
        } else {
            work.dir1 = Direction2::from_xy(c2.location().xy() - c1.location().xy())?;
            work.dist = work.center2.distance(work.center1);
            circ.dispatch(&mut work)?;
            circ.intersect_rings(&mut work)?;
        }
        circ.final_parameters(c1, &work)
    }

    fn dispatch(&mut self, w: &mut Work) -> Result<(), Failure> {
        let (q1, q2) = (w.q1, w.q2);
        let pair = |a: fn(&QualifiedCirc) -> bool, b: fn(&QualifiedCirc) -> bool| {
            (a(&q1) && b(&q2)) || (b(&q1) && a(&q2))
        };
        if q1.is_enclosed() && q2.is_enclosed() {
            self.enclosed_enclosed(w)
        } else if pair(QualifiedCirc::is_enclosed, QualifiedCirc::is_enclosing) {
            self.enclosed_enclosing(w)
        } else if pair(QualifiedCirc::is_enclosed, QualifiedCirc::is_outside) {
            self.enclosed_outside(w)
        } else if pair(QualifiedCirc::is_enclosed, QualifiedCirc::is_unqualified) {
            self.enclosed_unqualified(w)
        } else if q1.is_enclosing() && q2.is_enclosing() {
            self.enclosing_enclosing(w)
        } else if pair(QualifiedCirc::is_enclosing, QualifiedCirc::is_outside) {
            self.enclosing_outside(w)
        } else if pair(QualifiedCirc::is_enclosing, QualifiedCirc::is_unqualified) {
            self.enclosing_unqualified(w)
        } else if q1.is_outside() && q2.is_outside() {
            self.outside_outside(w)
        } else if pair(QualifiedCirc::is_outside, QualifiedCirc::is_unqualified) {
            self.outside_unqualified(w)
        } else {
            self.unqualified_unqualified(w)
        }
    }

    /// The single solution of a settled case: cirsol(1) at `center`, the given
    /// qualifiers, NbrSol = 1 and WellDone.
    pub(super) fn single(&mut self, w: &Work, center: Point2) -> Result<(), Failure> {
        self.well_done = true;
        self.nbr_sol = 1;
        self.cirsol.set(1, circle_at(center, w.radius)?)?;
        self.set_qualifiers(1, w.q1.qualifier(), w.q2.qualifier())
    }

    // The intersections of each pair of rings are the solution centres.
    fn intersect_rings(&mut self, w: &mut Work) -> Result<(), Failure> {
        for j in 1..=w.nbsol {
            let intp = AnaIntersection::circles(w.c[2 * j - 2], w.c[2 * j - 1])?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for i in 1..=intp.nb_points()? {
                    self.ring_solution(w, intp.point(i)?.value())?;
                }
            }
            self.well_done = true;
        }
        Ok(())
    }

    fn ring_solution(&mut self, w: &mut Work, center: Point2) -> Result<(), Failure> {
        let index: i32 = self.push_circle(center, w.radius)?;
        w.dir1 = Direction2::from_xy(center.xy() - w.center1.xy())?;
        let dir2 = Direction2::from_xy(center.xy() - w.center2.xy())?;
        let (distcc1, distcc2) = (center.distance(w.center1), center.distance(w.center2));
        let q1: Position = qualify(w.q1.qualifier(), distcc1, w.radius, w.r1, w.tol, false);
        let q2: Position = qualify(w.q2.qualifier(), distcc2, w.radius, w.r2, w.tol, false);
        self.set_qualifiers(index, q1, q2)?;
        let p1: Point2 = ring_tangency(center, w.center1, w.r1, w.radius, w.dir1);
        let p2: Point2 = ring_tangency(center, w.center2, w.r2, w.radius, dir2);
        self.tangency1.point.set(index, p1)?;
        self.tangency2.point.set(index, p2)?;
        self.tangency1.same.set(index, 0)?;
        self.tangency2.same.set(index, 0)
    }

    // Swap the tangency points back when the arguments were exchanged, then the
    // parameters (the argument ones only where TheSame is 0).
    fn final_parameters(mut self, c1: Circle2, w: &Work) -> Result<Self, Failure> {
        if w.invers {
            for i in 1..=self.nbr_sol {
                let psav: Point2 = self.tangency1.point.get(i)?;
                self.tangency1.point.set(i, self.tangency2.point.get(i)?)?;
                self.tangency2.point.set(i, psav)?;
            }
        }
        for i in 1..=self.nbr_sol {
            let solution: Circle2 = self.cirsol.get(i)?;
            let p1: Point2 = self.tangency1.point.get(i)?;
            self.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, p1)?)?;
            if self.tangency1.same.get(i)? == 0 {
                self.tangency1.par_arg.set(i, elclib::parameter(&c1, p1)?)?;
            }
            let p2: Point2 = self.tangency2.point.get(i)?;
            self.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, p2)?)?;
            if self.tangency2.same.get(i)? == 0 {
                self.tangency2
                    .par_arg
                    .set(i, elclib::parameter(&w.c2, p2)?)?;
            }
        }
        Ok(self)
    }
}

// The tangency point of a solution centred at `center` on the argument circle
// (centre, r): towards the argument unless the solution lies inside it.
pub(super) fn ring_tangency(
    center: Point2,
    centre: Point2,
    r: f64,
    radius: f64,
    dir: Direction2,
) -> Point2 {
    let d: f64 = center.distance(centre);
    let inside: bool = !((d > r) && (radius < d + r)) && (d < r) && (radius < r);
    if inside {
        return Point2::from(center.xy() + radius * dir.xy());
    }
    Point2::from(center.xy() - radius * dir.xy())
}
