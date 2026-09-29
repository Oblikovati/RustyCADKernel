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
// Rust translation of GccAna_Circ2d3Tan_4.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to two circles and through a point: the intersections of
//! the bisectors of (C1, C2) and of (C1, Point3), then two native clean-ups (the
//! radius is reset to the distance to the point, and coincident centres are
//! removed).
use super::candidates::RadiusCandidates;
use super::{Argument, Circ2d3Tan};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{Circ2dBisec, CircPnt2dBisec, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

// `static int MaxSol = 20` of GccAna_Circ2d3Tan_4.cxx.
const MAX_SOL: i32 = 20;

// Intp.Perform for the circle, ellipse, line and hyperbola bisectors, with the
// native argument order of each pair; None keeps the previous result.
fn intersect(
    s1: &Handle<dyn Bisec>,
    s2: &Handle<dyn Bisec>,
) -> Result<Option<AnaIntersection>, Failure> {
    use IType::{Cir, Ell, Hpr, Lin};
    let result = match (s1.arc_type(), s2.arc_type()) {
        (Cir, Cir) => AnaIntersection::circles(s1.circle()?, s2.circle()?)?,
        (Cir, Lin) => AnaIntersection::line_circle(s2.line()?, s1.circle()?)?,
        (Cir, Hpr) => {
            AnaIntersection::circle_conic(s1.circle()?, &Conic::from_hyperbola(s2.hyperbola()?))?
        }
        (Cir, Ell) => {
            AnaIntersection::circle_conic(s1.circle()?, &Conic::from_ellipse(s2.ellipse()?))?
        }
        (Ell, Cir) => {
            AnaIntersection::circle_conic(s2.circle()?, &Conic::from_ellipse(s1.ellipse()?))?
        }
        (Ell, Lin) => AnaIntersection::line_conic(s2.line()?, &Conic::from_ellipse(s1.ellipse()?))?,
        (Ell, Hpr) => {
            AnaIntersection::ellipse_conic(s1.ellipse()?, &Conic::from_hyperbola(s2.hyperbola()?))?
        }
        (Ell, Ell) => {
            AnaIntersection::ellipse_conic(s1.ellipse()?, &Conic::from_ellipse(s2.ellipse()?))?
        }
        (Lin, Cir) => AnaIntersection::line_circle(s1.line()?, s2.circle()?)?,
        (Lin, Lin) => AnaIntersection::lines(s1.line()?, s2.line()?),
        (Lin, Hpr) => {
            AnaIntersection::line_conic(s1.line()?, &Conic::from_hyperbola(s2.hyperbola()?))?
        }
        (Lin, Ell) => AnaIntersection::line_conic(s1.line()?, &Conic::from_ellipse(s2.ellipse()?))?,
        (Hpr, Cir) => {
            AnaIntersection::circle_conic(s2.circle()?, &Conic::from_hyperbola(s1.hyperbola()?))?
        }
        (Hpr, Lin) => {
            AnaIntersection::line_conic(s2.line()?, &Conic::from_hyperbola(s1.hyperbola()?))?
        }
        (Hpr, Hpr) => AnaIntersection::hyperbola_conic(
            s2.hyperbola()?,
            &Conic::from_hyperbola(s1.hyperbola()?),
        )?,
        (Hpr, Ell) => {
            AnaIntersection::ellipse_conic(s2.ellipse()?, &Conic::from_hyperbola(s1.hyperbola()?))?
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedCirc,
    point3: Point2,
    tolerance: f64,
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Point3, Tolerance): the circles
    /// tangent to two circles and through a point, up to 20 solutions;
    /// `Circ2d3Tan::circ_circ_pnt(q1, q2, point, tol)?`.
    pub fn circ_circ_pnt(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(MAX_SOL as usize);
        check_qualifier(qualified1.qualifier(), super::NAME)?;
        check_qualifier(qualified2.qualifier(), super::NAME)?;
        let args = Args {
            q1: qualified1,
            q2: qualified2,
            point3,
            tolerance,
        };
        let bis1 = Circ2dBisec::new(qualified1.qualified(), qualified2.qualified());
        let bis2 = CircPnt2dBisec::new(qualified1.qualified(), point3);
        if bis1.is_done() && bis2.is_done() {
            circ.scan_4(&args, &bis1, &bis2)?;
        }
        circ.reset_radii(point3, tolerance.abs())?;
        circ.remove_duplicates(tolerance.abs())?;
        Ok(circ)
    }

    fn scan_4(
        &mut self,
        a: &Args,
        bis1: &Circ2dBisec,
        bis2: &CircPnt2dBisec,
    ) -> Result<(), Failure> {
        let mut radius = RadiusCandidates::new();
        for i in 1..=bis1.nb_solutions()? {
            let sol1: Handle<dyn Bisec> = bis1.this_solution(i)?;
            let mut intp = AnaIntersection::new();
            for k in 1..=bis2.nb_solutions() {
                let sol2: Handle<dyn Bisec> = bis2.this_solution(k)?;
                if let Some(result) = intersect(&sol1, &sol2)? {
                    intp = result;
                }
                if intp.is_done() {
                    if !intp.is_empty()? {
                        for j in 1..=intp.nb_points()? {
                            self.centre_4(a, intp.point(j)?.value(), &mut radius)?;
                        }
                    }
                    self.well_done = true;
                }
                if self.nbr_sol == MAX_SOL {
                    break;
                }
            }
            if self.nbr_sol == MAX_SOL {
                break;
            }
        }
        Ok(())
    }

    fn centre_4(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut RadiusCandidates,
    ) -> Result<(), Failure> {
        let (c1, c2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = center.distance(c2.location());
        let dist3: f64 = center.distance(a.point3);
        let nbsol1: i32 = radius.first_circle(a.q1, dist1, a.tolerance, true);
        let nbsol2: i32 = radius.second_circle(a.q2, dist2, nbsol1, (a.tolerance, tol));
        let mut nbsol3: i32 = 0;
        for ii in 1..=nbsol2 {
            if (dist3 - radius.at(ii)).abs() <= tol {
                nbsol3 += 1;
                radius.ok = true;
            }
        }
        if !radius.ok {
            return Ok(());
        }
        // The native loop does not stop the scan of the other points at MaxSol.
        for k1 in 1..=nbsol3 {
            self.solution_4(a, center, radius.at(k1))?;
            self.well_done = true;
            if self.nbr_sol == MAX_SOL {
                break;
            }
        }
        Ok(())
    }

    fn solution_4(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, c2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        let index: i32 = self.push_circle(center, r)?;
        let q1: Position = qualify(
            a.q1.qualifier(),
            center.distance(c1.location()),
            r,
            c1.radius(),
            tol,
            false,
        );
        self.set_qualifier(1, index, q1)?;
        let q2: Position = qualify(
            a.q2.qualifier(),
            center.distance(c2.location()),
            r,
            c2.radius(),
            tol,
            false,
        );
        self.set_qualifier(2, index, q2)?;
        self.set_qualifier(3, index, Position::NoQualifier)?;
        if center.distance(c1.location()) <= a.tolerance && (r - c1.radius()).abs() <= a.tolerance {
            self.mark_same(1, index)?;
        } else {
            // A solution inside circle 1 touches it on the far side.
            let mut dc = Direction2::from_xy(c1.location().xy() - center.xy())?;
            if q1 == Position::Enclosed {
                dc.reverse();
            }
            self.touch(
                1,
                index,
                Point2::from(center.xy() + r * dc.xy()),
                Argument::Circle(c1),
            )?;
        }
        self.second_tangency_4(index, center, r, (c1, c2), a.tolerance)?;
        self.touch(3, index, a.point3, Argument::Point)
    }

    // For concentric arguments the second tangency point is on the other side of
    // the solution.
    fn second_tangency_4(
        &mut self,
        index: i32,
        center: Point2,
        r: f64,
        (c1, c2): (Circle2, Circle2),
        tolerance: f64,
    ) -> Result<(), Failure> {
        if center.distance(c2.location()) <= tolerance && (r - c2.radius()).abs() <= tolerance {
            return self.mark_same(2, index);
        }
        let dc = Direction2::from_xy(c2.location().xy() - center.xy())?;
        let alpha: f64 = if c1.location().distance(c2.location()) <= tolerance {
            -1.0
        } else {
            1.0
        };
        let point = Point2::from(center.xy() + alpha * r * dc.xy());
        self.touch(2, index, point, Argument::Circle(c2))
    }

    // "Debug to create the point on the solution circles": a radius off the
    // distance to Point3 by more than Tol becomes that distance.
    fn reset_radii(&mut self, point3: Point2, tol: f64) -> Result<(), Failure> {
        for kk in 1..=self.nbr_sol {
            let mut cc: Circle2 = self.cirsol.get(kk)?;
            let nr: f64 = cc.location().distance(point3);
            if (nr - cc.radius()).abs() > tol {
                cc.set_radius(nr)?;
                self.cirsol.set(kk, cc)?;
            }
        }
        Ok(())
    }

    // "Debug to eliminate multiple solution" (line-hyperbola intersections): a
    // later solution whose centre is within Tol of an earlier one is removed by
    // shifting the arrays down (TheSame is not shifted natively; the slot after a
    // removal is skipped).
    fn remove_duplicates(&mut self, tol: f64) -> Result<(), Failure> {
        let tol2: f64 = tol * tol;
        let mut kk: i32 = 1;
        while kk < self.nbr_sol {
            let pk: Point2 = self.cirsol.get(kk)?.location();
            let mut ll: i32 = kk + 1;
            while ll <= self.nbr_sol {
                let pl: Point2 = self.cirsol.get(ll)?.location();
                if pk.square_distance(pl) < tol2 {
                    for mm in ll + 1..=self.nbr_sol {
                        self.shift_down(mm)?;
                    }
                    self.nbr_sol -= 1;
                }
                ll += 1;
            }
            kk += 1;
        }
        Ok(())
    }

    // Slot mm - 1 takes the solution, points, parameters and qualifiers of slot mm.
    fn shift_down(&mut self, mm: i32) -> Result<(), Failure> {
        self.cirsol.set(mm - 1, self.cirsol.get(mm)?)?;
        for which in 1..=3 {
            let t = self.tangency_mut(which);
            t.point.set(mm - 1, t.point.get(mm)?)?;
            t.par_sol.set(mm - 1, t.par_sol.get(mm)?)?;
            t.par_arg.set(mm - 1, t.par_arg.get(mm)?)?;
        }
        for which in 1..=3 {
            let q = self.qualifier_mut(which);
            q.set(mm - 1, q.get(mm)?)?;
        }
        Ok(())
    }
}
