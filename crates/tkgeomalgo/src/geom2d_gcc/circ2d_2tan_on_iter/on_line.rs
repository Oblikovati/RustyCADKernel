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
// Rust translation of the constructors on a line of Geom2dGcc_Circ2d2TanOnIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles centred on a line.
use super::{Arg, Candidate, Circ2d2TanOnIter, ParSol, Search, solve, start_radius};
use crate::gcc_ana::{check_line_position, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::domains::std_max;
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuOnCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnIter";

// The point of parameter `u` on the locus line (Location + u Direction).
fn on_line_point(on_line: &Line2, u: f64) -> Point2 {
    Point2::from(on_line.location().xy() + u * on_line.direction().xy())
}

impl Circ2d2TanOnIter {
    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedLin Qualified1, Qualified2, OnLine,
    /// Param1, Param2, Param3, Tolerance)`. Natively the curve tolerance is the
    /// resolution of the signed `Tolerance`;
    /// `Circ2d2TanOnIter::lin_curve_on_line(ql, &q2, on, [0.0; 3], 1e-9)?`.
    pub fn lin_curve_on_line(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        on_line: Line2,
        [param1, param2, param3]: [f64; 3],
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        let tol: f64 = tolang.abs();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let (l1, cu2) = (qualified1.qualified(), qualified2.curve());
        let mut search = Search {
            umin: vec![
                real_first(),
                CurveTool::first_parameter(cu2)?,
                real_first(),
                0.0,
            ],
            umax: vec![
                real_last(),
                CurveTool::last_parameter(cu2)?,
                real_last(),
                real_last(),
            ],
            ufirst: vec![param1, param2, param3, 0.0],
            tol: vec![1.0e-15, CurveTool::eps_x(cu2, tolang)?, 1.0e-15, 1.0e-15],
        };
        let point1: Point2 = elclib::value(param1, &l1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, elclib::value(param3, &on_line));
        let mut func = FunctionTanCuCuOnCu::lin_curve_on_line(
            l1,
            cu2,
            on_line,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, _) = elclib::d1(u[0], &l1);
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Line(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: on_line_point(&on_line, u[2]),
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Qualified2, OnLine, Param1, Param2,
    /// Param3, Tolerance)`: no solution when the mean radius is below the tolerance;
    /// `Circ2d2TanOnIter::curve_curve_on_line(&q1, &q2, on, [0.0; 3], 1e-9)?`.
    pub fn curve_curve_on_line(
        qualified1: &QCurve,
        qualified2: &QCurve,
        on_line: Line2,
        [param1, param2, param3]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let (cu1, cu2) = (qualified1.curve(), qualified2.curve());
        let mut search = Search {
            umin: vec![
                CurveTool::first_parameter(cu1)?,
                CurveTool::first_parameter(cu2)?,
                real_first(),
                0.0,
            ],
            umax: vec![
                CurveTool::last_parameter(cu1)?,
                CurveTool::last_parameter(cu2)?,
                real_last(),
                real_last(),
            ],
            ufirst: vec![param1, param2, param3, 0.0],
            tol: vec![
                CurveTool::eps_x(cu1, tol)?,
                CurveTool::eps_x(cu2, tol)?,
                1.0e-15,
                tol / 10.0,
            ],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, elclib::value(param3, &on_line));
        let mut func = FunctionTanCuCuOnCu::curve_curve_on_line(
            cu1,
            cu2,
            on_line,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let centre: Point2 = on_line_point(&on_line, u[2]);
        if (centre.distance(point1) + centre.distance(point2)) / 2.0 < tol {
            return Ok(circ);
        }
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Point2, OnLine, Param1, Param2,
    /// Tolerance)`: Standard_OutOfRange from math_FunctionSetRoot, whose 4 tolerances
    /// are read from a vector of 3, as natively;
    /// `Circ2d2TanOnIter::curve_pnt_on_line(&q1, point, on, [0.0; 2], 1e-9)`.
    pub fn curve_pnt_on_line(
        qualified1: &QCurve,
        point2: Point2,
        on_line: Line2,
        [param1, param2]: [f64; 2],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let cu1 = qualified1.curve();
        let mut search = Search {
            umin: vec![CurveTool::first_parameter(cu1)?, real_first(), 0.0],
            umax: vec![CurveTool::last_parameter(cu1)?, real_last(), real_last()],
            ufirst: vec![param1, param2, 0.0],
            tol: vec![CurveTool::eps_x(cu1, tol)?, 1.0e-15, tol / 10.0],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        search.ufirst[2] = start_radius(point1, point2, elclib::value(param2, &on_line));
        let mut func = FunctionTanCuCuOnCu::curve_pnt_on_line(
            cu1,
            point2,
            on_line,
            std_max(search.ufirst[2], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1new, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point3new, _) = elclib::d1(u[1], &on_line);
        let cand = Candidate {
            points: [point1new, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Point],
            pararg: [u[0], u[1]],
            centre: point3new,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedCirc Qualified1, Qualified2, OnLine,
    /// Param1, Param2, Param3, Tolerance)`. Natively the centre is taken on the line
    /// at the circle parameter `X(1)`, not at `X(3)`;
    /// `Circ2d2TanOnIter::circ_curve_on_line(qc, &q2, on, [0.0; 3], 1e-9)?`.
    pub fn circ_curve_on_line(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        on_line: Line2,
        [param1, param2, param3]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let (c1, cu2) = (qualified1.qualified(), qualified2.curve());
        let mut search = Search {
            umin: vec![
                real_first(),
                CurveTool::first_parameter(cu2)?,
                real_first(),
                0.0,
            ],
            umax: vec![
                real_last(),
                CurveTool::last_parameter(cu2)?,
                real_last(),
                real_last(),
            ],
            ufirst: vec![param1, param2, param3, 0.0],
            tol: vec![
                2.0e-15 * PI,
                CurveTool::eps_x(cu2, tol)?,
                1.0e-15,
                tol / 10.0,
            ],
        };
        let point1: Point2 = elclib::value(param1, &c1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, elclib::value(param3, &on_line));
        let mut func = FunctionTanCuCuOnCu::circ_curve_on_line(
            c1,
            cu2,
            on_line,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, _, _) = elclib::d2(u[0], &c1);
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Circle(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: elclib::value(u[0], &on_line),
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }
}
