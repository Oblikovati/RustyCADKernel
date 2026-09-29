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
// Rust translation of GccAna_Circ2d2TanOn_2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to two lines with their centre on a line: the intersections of
//! the locus with the bisectors of the lines (at most two, radius below 1/Tol),
//! then a filter by the qualifiers that compacts the kept solutions.
use super::Circ2d2TanOn;
use super::circ_lin_on_line::check_line;
use crate::gcc_ana::Lin2dBisec;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

// The side of `pbid` relative to `line`: Dir(origin - pbid) . Dir(-dy, dx).
fn side_sign(line: Line2, pbid: Point2) -> Result<f64, Failure> {
    let (x, y) = (line.direction().x(), line.direction().y());
    let dc = Direction2::from_xy(line.location().xy() - pbid.xy())?;
    Ok(dc.dot(Direction2::new(-y, x)?))
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnLine, Tolerance): circles
    /// tangent to two qualified lines, centred on `on_line`, at most two;
    /// `Circ2d2TanOn::lin_lin_on_line(l1, l2, line, tol)?`.
    pub fn lin_lin_on_line(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        check_line(qualified1)?;
        check_line(qualified2)?;
        let tol: f64 = tolerance.abs();
        let (l1, l2) = (qualified1.qualified(), qualified2.qualified());
        let bis = Lin2dBisec::new(l1, l2)?;
        if bis.is_done() && (bis.nb_solutions()? == 1 || bis.nb_solutions()? == 2) {
            for k in 1..=bis.nb_solutions()? {
                let intp = AnaIntersection::lines(bis.this_solution(k)?, on_line);
                if !intp.is_done() {
                    continue;
                }
                circ.well_done = true;
                if intp.identical_elements()? || intp.parallel_elements()? || intp.is_empty()? {
                    continue;
                }
                for l in 1..=intp.nb_points()? {
                    let pt: Point2 = intp.point(l)?.value();
                    let radius: f64 = l1.distance(pt);
                    if !l1.contains(pt, tol) && radius < 1.0 / tol && circ.nbr_sol < 2 {
                        circ.nbr_sol += 1;
                        circ.cirsol.set(circ.nbr_sol, circle_at(pt, radius)?)?;
                    }
                }
            }
        }
        let mut i: i32 = 1;
        while i <= circ.nbr_sol {
            if circ.keep_between_lines(i, (qualified1, qualified2), on_line)? {
                i += 1;
                continue;
            }
            // Rejected: shift the following circles down and examine slot i again.
            for k in i + 1..=circ.nbr_sol {
                circ.cirsol.set(k - 1, circ.cirsol.get(k)?)?;
            }
            circ.nbr_sol -= 1;
        }
        Ok(circ)
    }

    // Whether solution `i` lies on the sides the qualifiers accept; if so, store
    // its tangency points, qualifiers and parameters.
    fn keep_between_lines(
        &mut self,
        i: i32,
        (q1, q2): (QualifiedLin, QualifiedLin),
        on_line: Line2,
    ) -> Result<bool, Failure> {
        let (l1, l2) = (q1.qualified(), q2.qualified());
        let solution: Circle2 = self.cirsol.get(i)?;
        let (pbid, radius) = (solution.location(), solution.radius());
        let accepts = |q: QualifiedLin, sign: f64| {
            if sign > 0.0 {
                q.is_unqualified() || q.is_outside()
            } else {
                q.is_unqualified() || q.is_enclosed()
            }
        };
        let sign1: f64 = side_sign(l1, pbid)?;
        let ok1: bool = accepts(q1, sign1);
        let sign2: f64 = side_sign(l2, pbid)?;
        if !(ok1 && accepts(q2, sign2)) {
            return Ok(false);
        }
        let side = |sign: f64| {
            if sign > 0.0 {
                Position::Outside
            } else {
                Position::Enclosed
            }
        };
        let foot1: Point2 = checked_foot(l1, pbid, radius, sign1)?;
        let foot2: Point2 = checked_foot(l2, pbid, radius, sign2)?;
        self.set_qualifiers(i, side(sign1), side(sign2))?;
        self.tangency1.record(
            i,
            foot1,
            elclib::parameter(&solution, foot1)?,
            elclib::parameter(&l1, foot1)?,
        )?;
        self.tangency2.record(
            i,
            foot2,
            elclib::parameter(&solution, foot2)?,
            elclib::parameter(&l2, foot2)?,
        )?;
        self.centre_on(i, pbid, elclib::parameter(&on_line, pbid)?)?;
        Ok(true)
    }
}

// pbid + radius * Dir(sign * (-dy, dx)), raising as natively for a null sign.
fn checked_foot(line: Line2, pbid: Point2, radius: f64, sign: f64) -> Result<Point2, Failure> {
    let (x, y) = (line.direction().x(), line.direction().y());
    let dc = Direction2::from_xy(sign * Xy::new(-y, x))?;
    Ok(Point2::from(pbid.xy() + radius * dc.xy()))
}
