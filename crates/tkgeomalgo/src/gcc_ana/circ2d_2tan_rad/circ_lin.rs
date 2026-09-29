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
// Rust translation of GccAna_Circ2d2TanRad_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a circle and a line. The qualifiers select
//! either a general case (nbsol 1: the intersections of circles concentric to C1
//! with parallels to L2) or one of three single configurations (nbsol 2, 3, 4).
//!
//! Native quirk kept: the nbsol 3 case sets TheSame1 only when R1 == Radius
//! exactly, leaving it unset otherwise (0 here, uninitialized natively).
use super::{Circ2d2TanRad, NAME};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{circle_at, qualify};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

/// The case selected by the qualifiers.
#[derive(Default)]
struct Plan {
    nbsol: i32,
    cote: f64,
    ccote: f64,
    cote1: Vec<f64>,
    cote2: Vec<f64>,
}

/// The measures the qualifier tests read.
struct Measures {
    side: f64,
    distance: f64,
    r1: f64,
    radius: f64,
    tol: f64,
}

impl Plan {
    fn single(nbsol: i32, cote: f64, ccote: f64) -> Self {
        Self {
            nbsol,
            cote,
            ccote,
            ..Self::default()
        }
    }
    fn general(cote1: &[f64], cote2: &[f64]) -> Self {
        Self {
            nbsol: 1,
            cote1: cote1.to_vec(),
            cote2: cote2.to_vec(),
            ..Self::default()
        }
    }
}

// Qualified1 enclosed: None when the configuration has no solution.
fn enclosed_plan(q2: QualifiedLin, m: &Measures) -> Option<Plan> {
    let (side, distance, r1, radius, tol) = (m.side, m.distance, m.r1, m.radius, m.tol);
    if q2.is_enclosed() {
        if (distance - r1 > tol)
            || ((side < 0.0) && (radius * 2.0 - r1 + distance > tol))
            || ((side > 0.0) && (radius * 2.0 - r1 - distance > tol))
            || (radius - r1 > tol)
        {
            return None;
        }
        if ((side < 0.0) && (radius * 2.0 > r1 - distance))
            || ((distance - r1).abs() < tol)
            || ((side > 0.0) && (radius * 2.0 > (r1 + distance + tol)))
        {
            return Some(Plan::single(3, 1.0, 1.0));
        }
        return Some(Plan::general(&[r1 - radius], &[1.0]));
    }
    if q2.is_outside() {
        if (distance > r1 + tol)
            || ((side > 0.0) && (radius * 2.0 - r1 + distance > tol))
            || ((side < 0.0) && (radius * 2.0 - r1 - distance > tol))
            || (radius - r1 > tol)
        {
            return None;
        }
        if ((side > 0.0) && (radius * 2.0 > r1 - distance))
            || ((r1 - radius).abs() < tol)
            || ((side < 0.0) && (radius * 2.0 > (r1 + distance)))
            || ((distance - r1).abs() < tol)
        {
            return Some(Plan::single(3, -1.0, 1.0));
        }
        return Some(Plan::general(&[r1 - radius], &[-1.0]));
    }
    if (distance - r1 > tol) || (radius * 2.0 - r1 - distance > tol) || (radius - r1 > tol) {
        return None;
    }
    if distance > r1 {
        return Some(Plan::single(3, if side > 0.0 { 1.0 } else { -1.0 }, 1.0));
    }
    Some(Plan::general(&[r1 - radius], &[1.0, -1.0]))
}

// Qualified1 enclosing.
fn enclosing_plan(q2: QualifiedLin, m: &Measures) -> Option<Plan> {
    let (side, distance, r1, radius, tol) = (m.side, m.distance, m.r1, m.radius, m.tol);
    if q2.is_enclosed() {
        if (distance < r1 - tol)
            || (radius < r1 - tol)
            || (radius * 2.0 < distance + r1 - tol)
            || (tol < r1 - radius)
            || (side < 0.0)
        {
            return None;
        }
        if (distance < r1) || (radius < r1) || (radius * 2.0 < distance + r1) {
            return Some(Plan::single(3, 1.0, 1.0));
        }
        return Some(Plan::general(&[radius - r1], &[1.0]));
    }
    if q2.is_outside() {
        if (tol < r1 - distance)
            || (tol < distance + r1 - radius * 2.0)
            || (tol < r1 - radius)
            || (side > 0.0)
        {
            return None;
        }
        if (distance < r1) || (radius * 2.0 < distance + r1) || (radius < r1) {
            return Some(Plan::single(3, -1.0, -1.0));
        }
        return Some(Plan::general(&[radius - r1], &[-1.0]));
    }
    if (distance < r1 - tol) || (radius * 2.0 < distance + r1 - tol) || (radius < r1 - tol) {
        return None;
    }
    if (distance < r1) || (radius * 2.0 < distance + r1) {
        let cote: f64 = if side > 0.0 { 1.0 } else { -1.0 };
        return Some(Plan::single(3, cote, cote));
    }
    Some(Plan::general(&[radius - r1], &[1.0, -1.0]))
}

// Qualified1 outside or unqualified with an enclosed or outside line. `near` is
// the side of C1 (side > 0 when `near_positive`) that native tests first; a circle
// beyond the line on that side gives cote `cote_near`, one beyond the diameter on
// the other side gives -cote_near.
fn one_side_plan(
    near_positive: bool,
    cote_near: f64,
    radii: &[f64],
    cote2: f64,
    m: &Measures,
) -> Option<Plan> {
    let (side, distance, r1, radius, tol) = (m.side, m.distance, m.r1, m.radius, m.tol);
    let (near, far) = if near_positive {
        (side > 0.0, side < 0.0)
    } else {
        (side < 0.0, side > 0.0)
    };
    if (near && (distance > r1 + tol)) || (distance > r1 + radius * 2.0 + tol) {
        return None;
    }
    if near && (distance > r1) {
        return Some(Plan::single(2, cote_near, 0.0));
    }
    if far && (distance > r1 + 2.0 * radius) {
        return Some(Plan::single(2, -cote_near, 0.0));
    }
    Some(Plan::general(radii, &[cote2]))
}

// Both lines' sides: the circle apart from the line beyond the diameter.
fn apart_plan(m: &Measures, cote_if_positive: f64, radii: &[f64]) -> Option<Plan> {
    let (side, distance, r1, radius, tol) = (m.side, m.distance, m.r1, m.radius, m.tol);
    if distance > radius * 2.0 + r1 + tol {
        return None;
    }
    if distance > radius * 2.0 + r1 {
        let cote: f64 = if side > 0.0 {
            cote_if_positive
        } else {
            -cote_if_positive
        };
        return Some(Plan::single(4, cote, 0.0));
    }
    Some(Plan::general(radii, &[1.0, -1.0]))
}

fn plan(q1: QualifiedCirc, q2: QualifiedLin, m: &Measures) -> Option<Plan> {
    let (r1, radius) = (m.r1, m.radius);
    if q1.is_enclosed() {
        return enclosed_plan(q2, m);
    }
    if q1.is_enclosing() {
        return enclosing_plan(q2, m);
    }
    let outside = [radius + r1];
    let both = [(radius - r1).abs(), radius + r1];
    match (q1.is_outside(), q2.qualifier()) {
        (true, Position::Enclosed) => one_side_plan(false, 1.0, &outside, 1.0, m),
        (true, Position::Outside) => one_side_plan(true, -1.0, &outside, -1.0, m),
        (true, _) => apart_plan(m, 1.0, &outside),
        (false, Position::Enclosed) => one_side_plan(true, 1.0, &both, 1.0, m),
        (false, Position::Outside) => one_side_plan(false, -1.0, &both, -1.0, m),
        (false, _) => apart_plan(m, -1.0, &both),
    }
}

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Qualified1, Qualified2, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified circle and a qualified line;
    /// `Circ2d2TanRad::circ_lin(q1, l2, radius, tol)?`.
    pub fn circ_lin(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let tol: f64 = tolerance.abs();
        let mut circ = Self::empty(8);
        check_qualifier(qualified1.qualifier(), NAME)?;
        if !(qualified2.is_enclosed() || qualified2.is_outside() || qualified2.is_unqualified()) {
            return Err(bad_qualifier(&format!(
                "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
                qualified2.qualifier()
            )));
        }
        let (c1, l2) = (qualified1.qualified(), qualified2.qualified());
        let (xdir, ydir) = (l2.direction().x(), l2.direction().y());
        let (lxloc, lyloc) = (l2.location().x(), l2.location().y());
        let (cxloc, cyloc) = (c1.location().x(), c1.location().y());
        let m = Measures {
            side: -ydir * (cxloc - lxloc) + xdir * (cyloc - lyloc),
            distance: l2.distance(c1.location()),
            r1: c1.radius(),
            radius,
            tol,
        };
        let dist: f64 = radius * 2.0 + m.r1;
        Self::check_radius(radius)?;
        if m.distance - dist > tol {
            circ.well_done = true;
        } else {
            match plan(qualified1, qualified2, &m) {
                None => circ.well_done = true,
                Some(p) => circ.build(qualified1, qualified2, &m, &p)?,
            }
        }
        circ.line_parameters(c1, l2)?;
        Ok(circ)
    }

    fn build(
        &mut self,
        q1: QualifiedCirc,
        q2: QualifiedLin,
        m: &Measures,
        p: &Plan,
    ) -> Result<(), Failure> {
        let (c1, l2) = (q1.qualified(), q2.qualified());
        let (xdir, ydir) = (l2.direction().x(), l2.direction().y());
        let (center1, radius) = (c1.location(), m.radius);
        let normal = Xy::new(-ydir, xdir);
        let back = Xy::new(ydir, -xdir);
        match p.nbsol {
            1 => self.general_circ_lin(q1, q2, m, p),
            2 => {
                let cen = Point2::from(center1.xy() + p.cote * (m.r1 + radius) * normal);
                self.single_circ_lin(q1, q2, cen, radius)?;
                self.tangency1
                    .point
                    .set(1, Point2::from(cen.xy() + p.cote * radius * back))?;
                self.tangency2
                    .point
                    .set(1, Point2::from(cen.xy() + radius * back))
            }
            3 => {
                let cen = Point2::from(center1.xy() + p.cote * (m.r1 - radius) * back);
                self.single_circ_lin(q1, q2, cen, radius)?;
                self.tangency2
                    .point
                    .set(1, Point2::from(cen.xy() + p.cote * radius * back))?;
                if (m.r1 - radius).abs() > 0.0 {
                    return self
                        .tangency1
                        .point
                        .set(1, Point2::from(cen.xy() + p.ccote * radius * back));
                }
                self.tangency1.same.set(1, 1)
            }
            _ => {
                let cent = Point2::from(center1.xy() + p.cote * (m.r1 + radius) * normal);
                self.single_circ_lin(q1, q2, cent, radius)?;
                self.tangency1
                    .point
                    .set(1, Point2::from(cent.xy() + p.cote * radius * back))?;
                self.tangency2
                    .point
                    .set(1, Point2::from(cent.xy() + p.cote * radius * normal))
            }
        }
    }

    fn single_circ_lin(
        &mut self,
        q1: QualifiedCirc,
        q2: QualifiedLin,
        center: Point2,
        radius: f64,
    ) -> Result<(), Failure> {
        self.cirsol.set(1, circle_at(center, radius)?)?;
        self.well_done = true;
        self.nbr_sol = 1;
        self.set_qualifiers(1, q1.qualifier(), q2.qualifier())
    }

    // The general case: centres on the circles concentric to C1 at radius cote1
    // and the parallels to L2 at signed distance cote2 * Radius.
    fn general_circ_lin(
        &mut self,
        q1: QualifiedCirc,
        q2: QualifiedLin,
        m: &Measures,
        p: &Plan,
    ) -> Result<(), Failure> {
        let (c1, l2) = (q1.qualified(), q2.qualified());
        let (xdir, ydir) = (l2.direction().x(), l2.direction().y());
        let (lxloc, lyloc) = (l2.location().x(), l2.location().y());
        for &cote1 in &p.cote1 {
            for &cote2 in &p.cote2 {
                if cote1 < m.tol {
                    continue;
                }
                let cirint: Circle2 = circle_at(c1.location(), cote1)?;
                let origin = Point2::new(
                    lxloc - cote2 * ydir * m.radius,
                    lyloc + cote2 * xdir * m.radius,
                );
                let intp =
                    AnaIntersection::line_circle(Line2::new(origin, l2.direction()), cirint)?;
                if !intp.is_done() {
                    continue;
                }
                if !intp.is_empty()? {
                    for i in 1..=intp.nb_points()? {
                        self.general_solution(q1, q2, m, cote2, intp.point(i)?.value())?;
                    }
                }
                self.well_done = true;
            }
        }
        Ok(())
    }

    fn general_solution(
        &mut self,
        q1: QualifiedCirc,
        q2: QualifiedLin,
        m: &Measures,
        cote2: f64,
        center: Point2,
    ) -> Result<(), Failure> {
        let (c1, l2) = (q1.qualified(), q2.qualified());
        let (xdir, ydir) = (l2.direction().x(), l2.direction().y());
        let radius: f64 = m.radius;
        let index: i32 = self.push_circle(center, radius)?;
        let dc2 = Direction2::from_xy(l2.location().xy() - center.xy())?;
        let distcc1: f64 = center.distance(c1.location());
        let position1: Position = qualify(q1.qualifier(), distcc1, radius, m.r1, m.tol, false);
        let normal = Direction2::new(-ydir, xdir)?;
        let position2: Position = if !q2.is_unqualified() {
            q2.qualifier()
        } else if dc2.dot(normal) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.set_qualifiers(index, position1, position2)?;
        self.tangency1.same.set(index, 0)?;
        let away = Direction2::from_xy(center.xy() - c1.location().xy())?;
        let p1 = if (radius < m.r1) && center.distance(c1.location()) <= m.r1 {
            Point2::from(center.xy() + radius * away.xy())
        } else {
            Point2::from(center.xy() - radius * away.xy())
        };
        self.tangency1.point.set(index, p1)?;
        let p2 = Point2::from(center.xy() + cote2 * radius * Xy::new(ydir, -xdir));
        self.tangency2.point.set(index, p2)
    }

    // The parameters of every solution; the circle one only where TheSame1 is 0.
    fn line_parameters(&mut self, c1: Circle2, l2: Line2) -> Result<(), Failure> {
        for i in 1..=self.nbr_sol {
            let solution: Circle2 = self.cirsol.get(i)?;
            let (p1, p2) = (self.tangency1.point.get(i)?, self.tangency2.point.get(i)?);
            self.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, p1)?)?;
            if self.tangency1.same.get(i)? == 0 {
                self.tangency1.par_arg.set(i, elclib::parameter(&c1, p1)?)?;
            }
            self.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, p2)?)?;
            self.tangency2.par_arg.set(i, elclib::parameter(&l2, p2)?)?;
        }
        Ok(())
    }
}
