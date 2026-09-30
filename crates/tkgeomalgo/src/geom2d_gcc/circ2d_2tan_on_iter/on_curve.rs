// Rust translation of the constructors on a curve of Geom2dGcc_Circ2d2TanOnIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles centred on a curve.
use super::{Arg, Candidate, Circ2d2TanOnIter, ParSol, Search, solve, start_radius};
use crate::gcc_ana::{check_line_position, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::geom2d_gcc::domains::std_max;
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuOnCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d2TanOnIter";

// (first, last, resolution of `tol`) of a curve.
fn range_of(c: &Geom2dAdaptorCurve, tol: f64) -> Result<(f64, f64, f64), Failure> {
    Ok((
        CurveTool::first_parameter(c)?,
        CurveTool::last_parameter(c)?,
        CurveTool::eps_x(c, tol)?,
    ))
}

impl Circ2d2TanOnIter {
    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Qualified2, OnCurv, Param1, Param2,
    /// Param3, Tolerance)`: the solution parameters are 0 and the chord length;
    /// `Circ2d2TanOnIter::curve_curve_on_curve(&q1, &q2, &on, [0.0; 3], 1e-9)?`.
    pub fn curve_curve_on_curve(
        qualified1: &QCurve,
        qualified2: &QCurve,
        on_curv: &Geom2dAdaptorCurve,
        [param1, param2, param3]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let (cu1, cu2) = (qualified1.curve(), qualified2.curve());
        let (r1, r2, ron) = (
            range_of(cu1, tol)?,
            range_of(cu2, tol)?,
            range_of(on_curv, tol)?,
        );
        let mut search = Search {
            umin: vec![r1.0, r2.0, ron.0, 0.0],
            umax: vec![r1.1, r2.1, ron.1, real_last()],
            ufirst: vec![param1, param2, param3, 0.0],
            tol: vec![r1.2, r2.2, ron.2, tol / 10.0],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, CurveTool::value(on_curv, param3)?);
        let mut func = FunctionTanCuCuOnCu::curve_curve_on_curve(
            cu1,
            cu2,
            on_curv,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let (point3, _) = CurveTool::d1(on_curv, u[2])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: point3,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::Chord)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedCirc Qualified1, Qualified2, OnCurv,
    /// Param1, Param2, ParamOn, Tolerance)`;
    /// `Circ2d2TanOnIter::circ_curve_on_curve(qc, &q2, &on, [0.0; 3], 1e-9)?`.
    pub fn circ_curve_on_curve(
        qualified1: QualifiedCirc,
        qualified2: &QCurve,
        on_curv: &Geom2dAdaptorCurve,
        [param1, param2, param_on]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let (c1, cu2) = (qualified1.qualified(), qualified2.curve());
        let (r2, ron) = (range_of(cu2, tol)?, range_of(on_curv, tol)?);
        let mut search = Search {
            umin: vec![real_first(), r2.0, ron.0, 0.0],
            umax: vec![real_last(), r2.1, ron.1, real_last()],
            ufirst: vec![param1, param2, param_on, 0.0],
            tol: vec![2.0e-15 * PI, r2.2, ron.2, tol / 10.0],
        };
        let point1: Point2 = elclib::value(param1, &c1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, CurveTool::value(on_curv, param_on)?);
        let mut func = FunctionTanCuCuOnCu::circ_curve_on_curve(
            c1,
            cu2,
            on_curv,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let (point3, _) = CurveTool::d1(on_curv, u[2])?;
        let (point1, _) = elclib::d1(u[0], &c1);
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Circle(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: point3,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(GccEnt_QualifiedLin Qualified1, Qualified2, OnCurve,
    /// Param1, Param2, ParamOn, Tolerance)`;
    /// `Circ2d2TanOnIter::lin_curve_on_curve(ql, &q2, &on, [0.0; 3], 1e-9)?`.
    pub fn lin_curve_on_curve(
        qualified1: QualifiedLin,
        qualified2: &QCurve,
        on_curv: &Geom2dAdaptorCurve,
        [param1, param2, param_on]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_line_position(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let (l1, cu2) = (qualified1.qualified(), qualified2.curve());
        let (r2, ron) = (range_of(cu2, tol)?, range_of(on_curv, tol)?);
        let mut search = Search {
            umin: vec![real_first(), r2.0, ron.0, 0.0],
            umax: vec![real_last(), r2.1, ron.1, real_last()],
            ufirst: vec![param1, param2, param_on, 0.0],
            tol: vec![1.0e-15, r2.2, ron.2, tol / 10.0],
        };
        let point1: Point2 = elclib::value(param1, &l1);
        let point2: Point2 = CurveTool::value(cu2, param2)?;
        search.ufirst[3] = start_radius(point1, point2, CurveTool::value(on_curv, param_on)?);
        let mut func = FunctionTanCuCuOnCu::lin_curve_on_curve(
            l1,
            cu2,
            on_curv,
            std_max(search.ufirst[3], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, _) = elclib::d1(u[0], &l1);
        let (point2, tan2) = CurveTool::d1(cu2, u[1])?;
        let (point3, _) = CurveTool::d1(on_curv, u[2])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Line(qualified1), Arg::Curve(qualified2, tan2)],
            pararg: [u[0], u[1]],
            centre: point3,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOnIter(Qualified1, Point2, OnCurve, Param1, ParamOn,
    /// Tolerance)`: Standard_DimensionError from math_FunctionSetRoot, whose start point
    /// is a vector of 3 for 4 unknowns, as natively;
    /// `Circ2d2TanOnIter::curve_pnt_on_curve(&q1, point, &on, [0.0; 2], 1e-9)`.
    pub fn curve_pnt_on_curve(
        qualified1: &QCurve,
        point2: Point2,
        on_curve: &Geom2dAdaptorCurve,
        [param1, param_on]: [f64; 2],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        let tol: f64 = tolerance.abs();
        let cu1 = qualified1.curve();
        let (r1, ron) = (range_of(cu1, tol)?, range_of(on_curve, tol)?);
        let mut search = Search {
            umin: vec![r1.0, real_first(), ron.0],
            umax: vec![r1.1, real_last(), ron.1],
            ufirst: vec![param1, param_on, 0.0],
            tol: vec![r1.2, ron.2, tol / 10.0],
        };
        let point1: Point2 = CurveTool::value(cu1, param1)?;
        search.ufirst[2] = start_radius(point1, point2, CurveTool::value(on_curve, param_on)?);
        let mut func = FunctionTanCuCuOnCu::curve_pnt_on_curve(
            cu1,
            point2,
            on_curve,
            std_max(search.ufirst[2], tol),
        );
        let Some(u) = solve(&mut func, &search)? else {
            return Ok(circ);
        };
        let (point1, tan1) = CurveTool::d1(cu1, u[0])?;
        let (point3, _) = CurveTool::d1(on_curve, u[2])?;
        let cand = Candidate {
            points: [point1, point2],
            args: [Arg::Curve(qualified1, tan1), Arg::Point],
            pararg: [u[0], 0.0],
            centre: point3,
            parcen3: u[2],
        };
        circ.finish(cand, tol, ParSol::OnCircle)?;
        Ok(circ)
    }
}
