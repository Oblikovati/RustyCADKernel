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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecAna.cxx (Perform, case 2: circle - line and case 3: line - line) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bisector of a circle and a line, and of two lines.
use super::perform_circles::Chosen;
use super::perform_curves::CurvesContext;
use super::solutions::{
    Trimmed, degenerate, half_line_like, line_solution, line_to_center, trimmed,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dParabola};
use rustycad_tkgeomalgo::gcc_ana::CircLin2dBisec;
use rustycad_tkgeomalgo::gcc_int::IType;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

impl CurvesContext {
    /// Case 2 of Perform: a circle and a line.
    pub(super) fn bisect_circle_line(
        &self,
        c1: Circle2,
        l2: Line2,
    ) -> Result<Option<Trimmed>, Failure> {
        let circle1: Circle2 = self.reframe_circle(c1, l2)?;
        let bisector = CircLin2dBisec::new(circle1, l2);
        let Some(chosen) = self.choose_circle_line_solution(&bisector)? else {
            return Ok(None);
        };
        match chosen.solution.arc_type() {
            IType::Lin => {
                let line: Line2 = chosen.solution.line()?;
                line_to_center(line, circle1.location(), chosen.first, chosen.sense).map(Some)
            }
            IType::Par => self.trim_parabola(&chosen, circle1.radius()).map(Some),
            _ => Ok(None),
        }
    }

    // On the curve, a circle almost tangent to the line becomes tangent (KHROMOV,
    // Oct 2002: the tangents must be parallel too).
    fn reframe_circle(&self, mut circle1: Circle2, l2: Line2) -> Result<Circle2, Failure> {
        if !self.oncurve {
            return Ok(circle1);
        }
        let radius1: f64 = circle1.radius();
        let d1: f64 = l2.distance(circle1.location()) - radius1;
        if d1.abs() < precision::confusion() && self.tan1.is_parallel(self.tan2, 1.0e-8)? {
            circle1.set_radius(radius1 + d1)?;
        }
        Ok(circle1)
    }

    // The loop over the GccAna solutions of a circle and a line: the status pass
    // always runs, and a solution off the curve is kept whatever its status. The
    // minimum is widened by 1e-8, so a later solution at the same distance wins.
    fn choose_circle_line_solution(
        &self,
        bisector: &CircLin2dBisec,
    ) -> Result<Option<Chosen>, Failure> {
        if !bisector.is_done() {
            return Ok(None);
        }
        let mut chosen: Option<Chosen> = None;
        let mut distancemini: f64 = precision::infinite();
        for i in 1..=bisector.nb_solutions() {
            let solution = degenerate(bisector.this_solution(i)?, self.tolerance)?;
            let status = self.status_pass(&*solution)?;
            if !(status.ok || !self.oncurve) {
                continue;
            }
            let result = self.sense_pass(&*solution)?;
            if result.distance <= distancemini {
                distancemini = result.distance + 1.0e-8;
                chosen = Some(Chosen {
                    solution,
                    first: result.parameter,
                    sense: result.sense,
                    distance: distancemini,
                });
            }
        }
        Ok(chosen)
    }

    // A parabola solution: trimmed at its apex for the joins by intersection when the
    // first point is too far from the apex.
    fn trim_parabola(&self, chosen: &Chosen, radius: f64) -> Result<Trimmed, Failure> {
        let curve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dParabola::from_parab2d(chosen.solution.parabola()?)).cast();
        let apex: Point2 = curve.value(0.)?;
        let firstpnt: Point2 = curve.value(chosen.first)?;
        let chord_len: f64 = apex.distance(firstpnt);
        let tol_par: f64 = 1.0e-5;
        let intersection: bool = self.ajointype == JoinType::Intersection;
        let mut second: f64 = precision::infinite();
        if !chosen.sense {
            if intersection && tol_par < chosen.first && chord_len >= radius {
                second = 0.;
            }
            // `-secondparameter`: a trim at the apex ends at -0.0.
            return trimmed(curve, chosen.first, -second, true);
        }
        if intersection && chosen.first < -tol_par && chord_len >= radius {
            second = 0.;
        }
        trimmed(curve, chosen.first, second, true)
    }

    /// Case 3 of Perform: two lines, parallel (KHROMOV, Sep 2002: within 1e-8 as in
    /// MAT2d_Circuit::InitOpen) or secant.
    pub(super) fn bisect_lines(&self, l1: Line2, l2: Line2) -> Result<Trimmed, Failure> {
        let (direc1, direc2) = (l1.direction(), l2.direction());
        if direc1.is_parallel(direc2, 1.0e-8) {
            return self.parallel_lines(l1, l2);
        }
        let line = Line2::new(self.apoint, Direction2::from_xy(direc2.xy() - direc1.xy())?);
        let result = self.lines_pass(&*line_solution(line))?;
        // A solution whose status is wrong on the curve keeps the sense false.
        let sense: bool = (result.ok || !self.oncurve) && result.sense;
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
        half_line_like(curve, 0., sense)
    }

    // Parallel lines: the normal through the point when they are the same line,
    // their direction otherwise (skv, IDEM(Airbus) Jul 2004: no status pass).
    fn parallel_lines(&self, l1: Line2, l2: Line2) -> Result<Trimmed, Failure> {
        let direction: Direction2 = if l1.distance(l2.location()) / 2. <= precision::confusion() {
            Direction2::new(-l1.direction().y(), l1.direction().x())?
        } else {
            l2.direction()
        };
        let line = Line2::new(self.apoint, direction);
        let result = self.sense_pass(&*line_solution(line))?;
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(line)).cast();
        half_line_like(curve, result.parameter, result.sense)
    }
}
