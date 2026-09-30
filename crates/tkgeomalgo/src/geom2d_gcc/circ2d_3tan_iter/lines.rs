// Rust translation of the line constructors of Geom2dGcc_Circ2d3TanIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a line and two curves, two lines and a curve, a line, a
//! curve and a point, or a circle, a line and a curve.
use super::{
    Circ2d3TanIter, Search, all_normal, circle_point, circle_through, cosine, solve, unit_tangent,
};
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ana::{check_line_position, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::circle_solutions::{circle_side, curve_side, line_side};
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkmath::gp::{Circle2, Line2, Point2, Vector2, Xy};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d3TanIter";

// The point of a line at `u`: location + u direction.
fn line_point(line: &Line2, u: f64) -> Point2 {
    Point2::from(line.location().xy() + u * line.direction().xy())
}

// The normal (-dy, dx) of a line direction, as a gp_XY.
fn line_normal(line: &Line2) -> Xy {
    Xy::new(-line.direction().y(), line.direction().x())
}

impl Circ2d3TanIter {
    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedLin Qualified1, Qualified2, Qualified3,
    /// Param1, Param2, Param3, Tolerance)`; GccEnt_BadQualifier for an enclosing line;
    /// `Circ2d3TanIter::lin_curve_curve(ql, &q2, &q3, [0.0; 3], 1e-9)?`.
    pub fn lin_curve_curve(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        qualified3: &QCurve,
        params: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        check_qualifier(qualified3.qualifier(), MEMBER)?;
        let l1: Line2 = qualified1.qualified();
        let (cu2, cu3) = (qualified2.curve(), qualified3.curve());
        let mut func = FunctionTanCuCuCu::lin_curve_curve(l1, cu2, cu3);
        let search = Search {
            umin: [
                real_first(),
                CurveTool::first_parameter(cu2)?,
                CurveTool::first_parameter(cu3)?,
            ],
            umax: [
                real_last(),
                CurveTool::last_parameter(cu2)?,
                CurveTool::last_parameter(cu3)?,
            ],
            ufirst: params,
            tol: [
                1.0e-15,
                CurveTool::eps_x(cu2, tolerance.abs())?,
                CurveTool::eps_x(cu3, tolerance.abs())?,
            ],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.line_two_curves(qualified1, [qualified2, qualified3], u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the line-curve-curve search at `u`.
    fn line_two_curves(
        &mut self,
        ql1: QualifiedLin,
        qs: [&QCurve; 2],
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let l1: Line2 = ql1.qualified();
        let point1: Point2 = line_point(&l1, u[0]);
        let (point2, tan2) = CurveTool::d1(qs[0].curve(), u[1])?;
        let (point3, tan3) = CurveTool::d1(qs[1].curve(), u[2])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        // creation variables intermediaires pour WNT
        let pscal: f64 = (centre.xy() - l1.location().xy()).dot(line_normal(&l1));
        let tan1 = Vector2::from(l1.direction().xy());
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !line_side(ql1.qualifier(), pscal)
            || !curve_side(qs[0], vec2.angle(tan2)?)
            || !curve_side(qs[1], vec3.angle(tan3)?)
        {
            return Ok(());
        }
        let qualifiers = [ql1.qualifier(), qs[0].qualifier(), qs[1].qualifier()];
        self.accept(qualifiers, [point1, point2, point3], u, [0.0; 3]);
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedLin Qualified1, GccEnt_QualifiedLin
    /// Qualified2, Qualified3, Param1, Param2, Param3, Tolerance)`. Natively both line
    /// side tests take the centre's product with the first line's normal (without
    /// its location), the first one before the tangency test;
    /// `Circ2d3TanIter::lin_lin_curve(ql1, ql2, &q3, [0.0; 3], 1e-9)?`.
    pub fn lin_lin_curve(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        qualified3: &QCurve,
        params: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_line_position(qualified2.qualifier(), MEMBER)?;
        check_qualifier(qualified3.qualifier(), MEMBER)?;
        let (l1, l2): (Line2, Line2) = (qualified1.qualified(), qualified2.qualified());
        let cu3 = qualified3.curve();
        let mut func = FunctionTanCuCuCu::lin_lin_curve(l1, l2, cu3);
        let search = Search {
            umin: [real_first(), real_first(), CurveTool::first_parameter(cu3)?],
            umax: [real_last(), real_last(), CurveTool::last_parameter(cu3)?],
            ufirst: params,
            tol: [1.0e-15, 1.0e-15, CurveTool::eps_x(cu3, tolerance.abs())?],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.two_lines_curve([qualified1, qualified2], qualified3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the line-line-curve search at `u`.
    fn two_lines_curve(
        &mut self,
        [ql1, ql2]: [QualifiedLin; 2],
        q3: &QCurve,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let (l1, l2): (Line2, Line2) = (ql1.qualified(), ql2.qualified());
        let (point1, point2) = (line_point(&l1, u[0]), line_point(&l2, u[1]));
        let (point3, tan3) = CurveTool::d1(q3.curve(), u[2])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let pscal: f64 = centre.xy().dot(line_normal(&l1));
        if !line_side(ql1.qualifier(), pscal) {
            return Ok(());
        }
        let (tan1, tan2) = (
            Vector2::from(l1.direction().xy()),
            Vector2::from(l2.direction().xy()),
        );
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !line_side(ql2.qualifier(), pscal)
            || !curve_side(q3, vec3.angle(tan3)?)
        {
            return Ok(());
        }
        let qualifiers = [ql1.qualifier(), ql2.qualifier(), q3.qualifier()];
        self.accept(qualifiers, [point1, point2, point3], u, [0.0; 3]);
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedLin Qualified1, Qualified2, Point3,
    /// Param1, Param2, Tolerance)`: the point is a circle of radius 0 (first function
    /// argument, from PI). Natively the curve point is taken at the line parameter;
    /// `Circ2d3TanIter::lin_curve_pnt(ql, &q2, point, [0.0; 2], 1e-9)?`.
    pub fn lin_curve_pnt(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        point3: Point2,
        [param1, param2]: [f64; 2],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let l1: Line2 = qualified1.qualified();
        let cu2 = qualified2.curve();
        let c3 = circle_at(point3, 0.0)?;
        let mut func = FunctionTanCuCuCu::circ_lin_curve(c3, l1, cu2);
        let search = Search {
            umin: [0.0, real_first(), CurveTool::first_parameter(cu2)?],
            umax: [2.0 * PI, real_last(), CurveTool::last_parameter(cu2)?],
            ufirst: [PI, param1, param2],
            tol: [2.0e-15, 1.0e-15, CurveTool::eps_x(cu2, tolerance.abs())?],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.line_curve_point(qualified1, qualified2, point3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the line-curve-point search at `u` (point circle first).
    fn line_curve_point(
        &mut self,
        ql1: QualifiedLin,
        q2: &QCurve,
        point3: Point2,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let l1: Line2 = ql1.qualified();
        let point1: Point2 = line_point(&l1, u[1]);
        let (point2, tan2) = CurveTool::d1(q2.curve(), u[1])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let pscal: f64 = centre.xy().dot(line_normal(&l1));
        let (tan1, tan3) = (Vector2::from(l1.direction().xy()), unit_tangent(u[0]));
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !line_side(ql1.qualifier(), pscal)
            || !curve_side(q2, vec2.angle(tan2)?)
        {
            return Ok(());
        }
        let qualifiers = [ql1.qualifier(), q2.qualifier(), Position::NoQualifier];
        self.accept(
            qualifiers,
            [point1, point2, point3],
            [u[1], u[2], 0.0],
            [0.0; 3],
        );
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(GccEnt_QualifiedCirc Qualified1, GccEnt_QualifiedLin
    /// Qualified2, Qualified3, Param1, Param2, Param3, Tolerance)`. Natively the
    /// function is evaluated at the start point before the root is read;
    /// `Circ2d3TanIter::circ_lin_curve(qc, ql, &q3, [0.0; 3], 1e-9)?`.
    pub fn circ_lin_curve(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        qualified3: &QCurve,
        params: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_line_position(qualified2.qualifier(), MEMBER)?;
        check_qualifier(qualified3.qualifier(), MEMBER)?;
        let (c1, l2): (Circle2, Line2) = (qualified1.qualified(), qualified2.qualified());
        let cu3 = qualified3.curve();
        let mut func = FunctionTanCuCuCu::circ_lin_curve(c1, l2, cu3);
        let search = Search {
            umin: [0.0, real_first(), CurveTool::first_parameter(cu3)?],
            umax: [2.0 * PI, real_last(), CurveTool::last_parameter(cu3)?],
            ufirst: params,
            tol: [
                2.0e-15 * PI,
                1.0e-15,
                CurveTool::eps_x(cu3, tolerance.abs())?,
            ],
        };
        if let Some(u) = solve(&mut func, &search, true)? {
            circ.circle_line_curve(qualified1, qualified2, qualified3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the circle-line-curve search at `u`.
    fn circle_line_curve(
        &mut self,
        qc1: QualifiedCirc,
        ql2: QualifiedLin,
        q3: &QCurve,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let (c1, l2): (Circle2, Line2) = (qc1.qualified(), ql2.qualified());
        let point1: Point2 = circle_point(&c1, u[0]);
        let point2: Point2 = line_point(&l2, u[1]);
        let (point3, tan3) = CurveTool::d1(q3.curve(), u[2])?;
        let Some(cirsol) = circle_through([point1, point2, point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let (tan1, tan2) = (unit_tangent(u[0]), Vector2::from(l2.direction().xy()));
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        let dist: f64 = c1.location().distance(centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)])
            || !circle_side(qc1.qualifier(), cirsol.radius(), c1.radius(), dist)
            || !line_side(ql2.qualifier(), centre.xy().dot(line_normal(&l2)))
            || !curve_side(q3, vec3.angle(tan3)?)
        {
            return Ok(());
        }
        let qualifiers = [qc1.qualifier(), ql2.qualifier(), q3.qualifier()];
        self.accept(qualifiers, [point1, point2, point3], u, [0.0; 3]);
        Ok(())
    }
}
