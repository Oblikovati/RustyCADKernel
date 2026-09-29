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
// Rust translation of GccAna_Circ2d2TanOn_6.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to two circles with their centre on a circle: a boundary case
//! centred where the line from C1 to the locus centre meets the locus, else the
//! intersections of the locus with the bisectors of the circles.
//!
//! Native quirk kept: an unqualified first circle writes Rbid(1) twice and leaves
//! Rbid(2) unset (0 here, uninitialized natively).
use super::circ_circ_on_line::agreeing_radii;
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{Slots, circle_at, qualify};
use crate::gcc_ana::{Circ2dBisec, check_qualifier};
use crate::gcc_ent::QualifiedCirc;
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedCirc,
    on_circ: Circle2,
    tolerance: f64,
}

// The four tangency defects of a centre at `pinterm`; all must be within Tol.
fn defects_ok(a: &Args, pinterm: Point2, tol: f64) -> bool {
    let (c1, c2) = (a.q1.qualified(), a.q2.qualified());
    let (r1, r2) = (c1.radius(), c2.radius());
    let distcc2: f64 = pinterm.distance(c2.location());
    let distcc1: f64 = pinterm.distance(c1.location());
    let d1: f64 = (distcc2 - r2 - (distcc1 - r1).abs()).abs();
    let d2: f64 = (distcc2 + r2 - (distcc1 - r1).abs()).abs();
    let d3: f64 = (distcc2 - r2 - (distcc1 + r1)).abs();
    let d4: f64 = (distcc2 + r2 - (distcc1 + r1)).abs();
    !(d1 > tol || d2 > tol || d3 > tol || d4 > tol)
}

// The radii a circle allows at a centre at `distcc` from it (Radius / Rradius).
fn boundary_radii(q: QualifiedCirc, distcc: f64) -> Vec<f64> {
    let r: f64 = q.qualified().radius();
    if q.is_enclosed() || q.is_outside() {
        vec![(distcc - r).abs()]
    } else if q.is_enclosing() {
        vec![r + distcc]
    } else {
        vec![(distcc - r).abs(), r + distcc]
    }
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnCirc, Tolerance): circles
    /// tangent to two qualified circles, centred on `on_circ`, up to 8;
    /// `Circ2d2TanOn::circ_circ_on_circ(q1, q2, circle, tol)?`.
    pub fn circ_circ_on_circ(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        on_circ: Circle2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(8);
        check_qualifier(qualified1.qualifier(), NAME)?;
        check_qualifier(qualified2.qualifier(), NAME)?;
        let a = Args {
            q1: qualified1,
            q2: qualified2,
            on_circ,
            tolerance,
        };
        let mut radius: Slots<f64> = Slots::new(2);
        if circ.boundary_6(&a, &mut radius)? {
            return Ok(circ);
        }
        let bis = Circ2dBisec::new(qualified1.qualified(), qualified2.qualified());
        if !bis.is_done() {
            return Ok(circ);
        }
        let (mut rbid1, mut rbid2): (Slots<f64>, Slots<f64>) = (Slots::new(2), Slots::new(2));
        for i in 1..=bis.nb_solutions()? {
            let sol: Handle<dyn Bisec> = bis.this_solution(i)?;
            let intp = match sol.arc_type() {
                IType::Cir => AnaIntersection::circles(on_circ, sol.circle()?)?,
                IType::Lin => AnaIntersection::line_circle(sol.line()?, on_circ)?,
                IType::Hpr => AnaIntersection::circle_conic(
                    on_circ,
                    &Conic::from_hyperbola(sol.hyperbola()?),
                )?,
                IType::Ell => {
                    AnaIntersection::circle_conic(on_circ, &Conic::from_ellipse(sol.ellipse()?))?
                }
                _ => AnaIntersection::new(),
            };
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? && !intp.parallel_elements()? && !intp.identical_elements()? {
                for j in 1..=intp.nb_points()? {
                    let center: Point2 = intp.point(j)?.value();
                    let dists = (
                        center.distance(qualified1.qualified().location()),
                        center.distance(qualified2.qualified().location()),
                    );
                    let nnsol: i32 = agreeing_radii(
                        (qualified1, qualified2),
                        dists,
                        tolerance.abs(),
                        (&mut rbid1, &mut rbid2),
                        &mut radius,
                    )?;
                    for k in 1..=nnsol {
                        circ.bisector_solution_6(&a, center, radius.get(k)?)?;
                    }
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    // The boundary case; true when it produced the solutions (native returns).
    fn boundary_6(&mut self, a: &Args, radius: &mut Slots<f64>) -> Result<bool, Failure> {
        let (c1, tol) = (a.q1.qualified(), a.tolerance.abs());
        let center1: Point2 = c1.location();
        let ron: f64 = a.on_circ.radius();
        let distcco: f64 = a.on_circ.location().distance(center1);
        let dircc = Direction2::from_xy(a.on_circ.location().xy() - center1.xy())?;
        let mut pinterm = Point2::from(center1.xy() + (distcco - ron) * dircc.xy());
        if !defects_ok(a, pinterm, tol) {
            pinterm = Point2::from(center1.xy() + (distcco + ron) * dircc.xy());
            if !defects_ok(a, pinterm, tol) {
                return Ok(false);
            }
        }
        let distcc1: f64 = pinterm.distance(center1);
        let distcc2: f64 = pinterm.distance(a.q2.qualified().location());
        let radii1: Vec<f64> = boundary_radii(a.q1, distcc1);
        let radii2: Vec<f64> = boundary_radii(a.q2, distcc2);
        for (n, &value) in radii1.iter().enumerate() {
            radius.set(n as i32 + 1, value)?;
        }
        for &r in &radii1 {
            for &rr in &radii2 {
                if (r - rr).abs() <= tol {
                    self.well_done = true;
                    self.boundary_solution_6(a, pinterm, r)?;
                }
            }
        }
        Ok(self.well_done)
    }

    fn boundary_solution_6(&mut self, a: &Args, pinterm: Point2, r: f64) -> Result<(), Failure> {
        let (c1, c2, tol) = (a.q1.qualified(), a.q2.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(pinterm, r)?;
        self.cirsol.set(index, solution)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - pinterm.xy())?;
        let dc2 = Direction2::from_xy(c2.location().xy() - pinterm.xy())?;
        let (distcc1, distcc2) = (
            pinterm.distance(c1.location()),
            pinterm.distance(c2.location()),
        );
        let q1 = qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false);
        let q2 = qualify(a.q2.qualifier(), distcc2, r, c2.radius(), tol, false);
        self.set_qualifiers(index, q1, q2)?;
        let p1 = Point2::from(pinterm.xy() + r * dc1.xy());
        let p2 = Point2::from(pinterm.xy() + r * dc2.xy());
        let centre: Point2 = solution.location();
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&c1, p1)?,
        )?;
        self.tangency2.record(
            index,
            p2,
            elclib::parameter(&solution, p2)?,
            elclib::parameter(&c2, p2)?,
        )?;
        self.centre_on(index, centre, elclib::parameter(&a.on_circ, centre)?)
    }

    fn bisector_solution_6(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, c2, tol) = (a.q1.qualified(), a.q2.qualified(), a.tolerance.abs());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, r)?;
        self.cirsol.set(index, solution)?;
        let (distcc1, distcc2) = (
            center.distance(c1.location()),
            center.distance(c2.location()),
        );
        let q1 = qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false);
        let q2 = qualify(a.q2.qualifier(), distcc2, r, c2.radius(), tol, false);
        self.set_qualifiers(index, q1, q2)?;
        for (which, argument) in [(1, c1), (2, c2)] {
            if center.distance(argument.location()) <= a.tolerance
                && (r - argument.radius()).abs() <= a.tolerance
            {
                self.tangency_of(which).same.set(index, 1)?;
                continue;
            }
            let dc = Direction2::from_xy(argument.location().xy() - center.xy())?;
            let p = Point2::from(center.xy() + r * dc.xy());
            let (par_sol, par_arg) = (
                elclib::parameter(&solution, p)?,
                elclib::parameter(&argument, p)?,
            );
            self.tangency_of(which).record(index, p, par_sol, par_arg)?;
        }
        self.centre_on(index, center, elclib::parameter(&a.on_circ, center)?)
    }
}
