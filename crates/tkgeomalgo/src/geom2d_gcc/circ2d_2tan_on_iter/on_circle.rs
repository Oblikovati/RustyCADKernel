// Rust translation of the constructors on a circle of Geom2dGcc_Circ2d2TanOnIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles centred on a circle.
use super::{Arg, Candidate, Circ2d2TanOnIter, ParSol, Search, solve, start_radius};
use crate::gcc_ana::{check_line_position, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::domains::std_max;
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuOnCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2, Xy};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnIter";

// `OnCirc.Location() + R (cos u, sin u)`, as the curve-curve constructor writes it.
fn on_circle_point(on_circ: &Circle2, u: f64) -> Point2 {
    Point2::from(on_circ.location().xy() + on_circ.radius() * Xy::new(u.cos(), u.sin()))
}

impl Circ2d2TanOnIter {
    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedCirc Qualified1, Qualified2, OnCirc,
    /// Param1, Param2, Param3, Tolerance)`. Natively the function is built with the
    /// curve-curve type, so it evaluates its empty first curve (a null handle
    /// natively, Standard_NullObject here);
    /// `Circ2d2TanOnIter::circ_curve_on_circle(qc, &q2, on, [0.0; 3], 1e-9)`.
    pub fn circ_curve_on_circle(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        on_circ: Circle2,
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
                2.0e-15 * PI,
                tol / 10.0,
            ],
        };
        let point1: Point2 = elclib::value(param1, &c1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, elclib::value(param3, &on_circ));
        let mut func = FunctionTanCuCuOnCu::circ_curve_on_circle(
            c1,
            cu2,
            on_circ,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, _) = elclib::d1(u[0], &c1);
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Circle(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: elclib::value(u[2], &on_circ),
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedLin Qualified1, Qualified2, OnCirc,
    /// Param1, Param2, Param3, Tolerance)`;
    /// `Circ2d2TanOnIter::lin_curve_on_circle(ql, &q2, on, [0.0; 3], 1e-9)?`.
    pub fn lin_curve_on_circle(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        on_circ: Circle2,
        [param1, param2, param3]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
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
            tol: vec![
                1.0e-15,
                CurveTool::eps_x(cu2, tol)?,
                2.0e-15 * PI,
                tol / 10.0,
            ],
        };
        let point1: Point2 = elclib::value(param1, &l1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, elclib::value(param3, &on_circ));
        let mut func = FunctionTanCuCuOnCu::lin_curve_on_circle(
            l1,
            cu2,
            on_circ,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1new, _) = elclib::d1(u[0], &l1);
        let (point2new, tan2) = CurveTool::d1(cu2, u[1])?;
        let cand = Candidate {
            points: [point1new, point2new],
            args: [Arg::Line(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: elclib::value(u[2], &on_circ),
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Qualified2, OnCirc, Param1, Param2,
    /// Param3, Tolerance)`: the solution parameters are 0 and the chord length;
    /// `Circ2d2TanOnIter::curve_curve_on_circle(&q1, &q2, on, [0.0; 3], 1e-9)?`.
    pub fn curve_curve_on_circle(
        qualified1: &QCurve,
        qualified2: &QCurve,
        on_circ: Circle2,
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
                2.0e-15 * PI,
                tol / 10.0,
            ],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, on_circle_point(&on_circ, param3));
        let mut func = FunctionTanCuCuOnCu::curve_curve_on_circle(
            cu1,
            cu2,
            on_circ,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: on_circle_point(&on_circ, u[2]),
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::Chord)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Point2, OnCirc, Param1, Param2,
    /// Tolerance)`: Standard_DimensionError from math_FunctionSetRoot, whose start point
    /// is a vector of 3 for 4 unknowns, as natively;
    /// `Circ2d2TanOnIter::curve_pnt_on_circle(&q1, point, on, [0.0; 2], 1e-9)`.
    pub fn curve_pnt_on_circle(
        qualified1: &QCurve,
        point2: Point2,
        on_circ: Circle2,
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
            tol: vec![CurveTool::eps_x(cu1, tol)?, 2.0e-15 * PI, tol / 10.0],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        search.ufirst[2] = start_radius(point1, point2, elclib::value(param2, &on_circ));
        let mut func = FunctionTanCuCuOnCu::curve_pnt_on_circle(
            cu1,
            point2,
            on_circ,
            std_max(search.ufirst[2], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1new, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point3new, _) = elclib::d1(u[1], &on_circ);
        let cand = Candidate {
            points: [point1new, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Point],
            pararg: [u[0], 0.0],
            centre: point3new,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }
}
