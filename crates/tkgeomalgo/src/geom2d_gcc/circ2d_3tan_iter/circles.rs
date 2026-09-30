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
// Rust translation of the circle constructors of Geom2dGcc_Circ2d3TanIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a circle and two curves, two circles and a curve, or a
//! circle, a curve and a point.
use super::{
    Circ2d3TanIter, Search, all_normal, circle_point, circle_through, cosine, solve, unit_tangent,
};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::geom2d_gcc::circle_solutions::{circle_side, curve_side};
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Circle2, Point2, Vector2};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d3TanIter";

// Vec . Tan / |Vec| for a unit tangent, or 0 below gp::Resolution().
fn unit_cosine(vec: Vector2, tan: Vector2) -> f64 {
    let normevec: f64 = vec.magnitude();
    if normevec >= gp::resolution() {
        return vec.dot(tan) / normevec;
    }
    0.0
}

impl Circ2d3TanIter {
    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedCirc Qualified1, Qualified2,
    /// Qualified3, Param1, Param2, Param3, Tolerance)`;
    /// `Circ2d3TanIter::circ_curve_curve(qc, &q2, &q3, [0.0; 3], 1e-9)?`.
    pub fn circ_curve_curve(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        qualified3: &QCurve,
        params: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        check_qualifier(qualified3.qualifier(), MEMBER)?;
        let c1: Circle2 = qualified1.qualified();
        let (cu2, cu3) = (qualified2.curve(), qualified3.curve());
        let mut func = FunctionTanCuCuCu::circ_curve_curve(c1, cu2, cu3);
        let search = Search {
            umin: [
                0.0,
                CurveTool::first_parameter(cu2)?,
                CurveTool::first_parameter(cu3)?,
            ],
            umax: [
                2.0 * PI,
                CurveTool::last_parameter(cu2)?,
                CurveTool::last_parameter(cu3)?,
            ],
            ufirst: params,
            tol: [
                2.0e-15 * PI,
                CurveTool::eps_x(cu2, tolerance.abs())?,
                CurveTool::eps_x(cu3, tolerance.abs())?,
            ],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.circle_two_curves(qualified1, [qualified2, qualified3], u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the circle-curve-curve search at `u`.
    fn circle_two_curves(
        &mut self,
        qualified1: QualifiedCirc,
        qs: [&QCurve; 2],
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let (point1, tan1) = (circle_point(&c1, u[0]), unit_tangent(u[0]));
        let (point2, tan2) = CurveTool::d1(qs[0].curve(), u[1])?;
        let (point3, tan3) = CurveTool::d1(qs[1].curve(), u[2])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let dist: f64 = c1.location().distance(centre);
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !circle_side(qualified1.qualifier(), cirsol.radius(), c1.radius(), dist)
            || !curve_side(qs[0], vec2.angle(tan2)?)
            || !curve_side(qs[1], vec3.angle(tan3)?)
        {
            return Ok(());
        }
        let qualifiers = [qualified1.qualifier(), qs[0].qualifier(), qs[1].qualifier()];
        self.accept(qualifiers, [point1, point2, point3], u, [0.0; 3]);
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedCirc Qualified1, GccEnt_QualifiedCirc
    /// Qualified2, Qualified3, Param1, Param2, Param3, Tolerance)`. Natively the
    /// second circle is tested with the first qualifier;
    /// `Circ2d3TanIter::circ_circ_curve(qc1, qc2, &q3, [0.0; 3], 1e-9)?`.
    pub fn circ_circ_curve(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        qualified3: &QCurve,
        params: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        check_qualifier(qualified3.qualifier(), MEMBER)?;
        let (c1, c2): (Circle2, Circle2) = (qualified1.qualified(), qualified2.qualified());
        let cu3 = qualified3.curve();
        let mut func = FunctionTanCuCuCu::circ_circ_curve(c1, c2, cu3);
        let search = Search {
            umin: [0.0, 0.0, CurveTool::first_parameter(cu3)?],
            umax: [2.0 * PI, 2.0 * PI, CurveTool::last_parameter(cu3)?],
            ufirst: params,
            tol: [
                2.0e-15 * PI,
                2.0e-15 * PI,
                CurveTool::eps_x(cu3, tolerance.abs())?,
            ],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.two_circles_curve([qualified1, qualified2], qualified3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the circle-circle-curve search at `u`.
    fn two_circles_curve(
        &mut self,
        [qualified1, qualified2]: [QualifiedCirc; 2],
        q3: &QCurve,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let (c1, c2): (Circle2, Circle2) = (qualified1.qualified(), qualified2.qualified());
        let (point1, tan1) = (circle_point(&c1, u[0]), unit_tangent(u[0]));
        let (point2, tan2) = (circle_point(&c2, u[1]), unit_tangent(u[1]));
        let (point3, tan3) = CurveTool::d1(q3.curve(), u[2])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let rsol: f64 = cirsol.radius();
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        let q1: Position = qualified1.qualifier();
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !circle_side(q1, rsol, c1.radius(), c1.location().distance(centre))
            || !circle_side(q1, rsol, c2.radius(), c2.location().distance(centre))
            || !curve_side(q3, Vector2::between(point3, centre).angle(tan3)?)
        {
            return Ok(());
        }
        let qualifiers = [q1, qualified2.qualifier(), q3.qualifier()];
        self.accept(qualifiers, [point1, point2, point3], u, [0.0; 3]);
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedCirc Qualified1, Qualified2, Point3,
    /// Param1, Param2, Tolerance)`: the point is a circle of radius 0 (second
    /// function argument, from PI). Natively the curve point is taken at that point
    /// circle's parameter and the point tangent at the curve's one;
    /// `Circ2d3TanIter::circ_curve_pnt(qc, &q2, point, [0.0; 2], 1e-9)?`.
    pub fn circ_curve_pnt(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        point3: Point2,
        [param1, param2]: [f64; 2],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let c1: Circle2 = qualified1.qualified();
        let cu2 = qualified2.curve();
        let c3 = circle_at(point3, 0.0)?;
        let mut func = FunctionTanCuCuCu::circ_circ_curve(c1, c3, cu2);
        let search = Search {
            umin: [0.0, 0.0, CurveTool::first_parameter(cu2)?],
            umax: [2.0 * PI, 2.0 * PI, CurveTool::last_parameter(cu2)?],
            ufirst: [param1, PI, param2],
            tol: [
                2.0e-15 * PI,
                2.0e-15 * PI,
                CurveTool::eps_x(cu2, tolerance.abs())?,
            ],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.circle_curve_point(qualified1, qualified2, point3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the circle-curve-point search at `u` (with the native
    // parameter slips: the curve at u[1], the point tangent at u[2]).
    fn circle_curve_point(
        &mut self,
        qualified1: QualifiedCirc,
        q2: &QCurve,
        point3: Point2,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let point1: Point2 = circle_point(&c1, u[0]);
        let (point2, tan2) = CurveTool::d1(q2.curve(), u[1])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let (tan1, tan3) = (unit_tangent(u[0]), unit_tangent(u[2]));
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        let dist: f64 = c1.location().distance(centre);
        if !all_normal([
            unit_cosine(vec1, tan1),
            cosine(vec2, tan2),
            unit_cosine(vec3, tan3),
        ]) || !circle_side(qualified1.qualifier(), cirsol.radius(), c1.radius(), dist)
            || !curve_side(q2, vec2.angle(tan2)?)
        {
            return Ok(());
        }
        let qualifiers = [
            qualified1.qualifier(),
            q2.qualifier(),
            Position::NoQualifier,
        ];
        self.accept(
            qualifiers,
            [point1, point2, point3],
            [u[0], u[1], 0.0],
            [0.0; 3],
        );
        Ok(())
    }
}
