// Rust translation of the curve and point constructors of Geom2dGcc_Circ2d3TanIter.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to three curves, two curves and a point, or a curve and
//! two points.
use super::{Circ2d3TanIter, Search, all_normal, circle_through, cosine, solve, unit_tangent};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use crate::geom2d_gcc::circle_solutions::curve_side;
use crate::geom2d_gcc::{CurveTool, FunctionTanCuCuCu, QCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use std::f64::consts::PI;

const MEMBER: &str = "Geom2dGcc_Circ2d3TanIter";

// The first and last parameters and the resolution of `tol` of a curve argument.
fn curve_range(q: &QCurve, tol: f64) -> Result<(f64, f64, f64), Failure> {
    let c = q.curve();
    Ok((
        CurveTool::first_parameter(c)?,
        CurveTool::last_parameter(c)?,
        CurveTool::eps_x(c, tol.abs())?,
    ))
}

impl Circ2d3TanIter {
    /// `Geom2dGcc_Circ2d3TanIter(Qualified1, Qualified2, Qualified3, Param1, Param2,
    /// Param3, Tolerance)` on three curves; GccEnt_BadQualifier for
    /// GccEnt_noqualifier; `Circ2d3TanIter::new(&q1, &q2, &q3, 0.0, 0.0, 0.0, 1e-9)?`.
    pub fn new(
        qualified1: &QCurve,
        qualified2: &QCurve,
        qualified3: &QCurve,
        [param1, param2, param3]: [f64; 3],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        for q in [qualified1, qualified2, qualified3] {
            check_qualifier(q.qualifier(), MEMBER)?;
        }
        let mut func =
            FunctionTanCuCuCu::new(qualified1.curve(), qualified2.curve(), qualified3.curve());
        let (r1, r2, r3) = (
            curve_range(qualified1, tolerance)?,
            curve_range(qualified2, tolerance)?,
            curve_range(qualified3, tolerance)?,
        );
        let search = Search {
            umin: [r1.0, r2.0, r3.0],
            umax: [r1.1, r2.1, r3.1],
            ufirst: [param1, param2, param3],
            tol: [r1.2, r2.2, r3.2],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.three_curves([qualified1, qualified2, qualified3], u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the three-curve search at the parameters `u`.
    fn three_curves(&mut self, qs: [&QCurve; 3], u: [f64; 3], tol: f64) -> Result<(), Failure> {
        let mut points = [Point2::default(); 3];
        let mut tans = [Vector2::default(); 3];
        for i in 0..3 {
            (points[i], tans[i]) = CurveTool::d1(qs[i].curve(), u[i])?;
        }
        let Some(cirsol) = circle_through(points, tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let vecs: [Vector2; 3] = points.map(|p| Vector2::between(p, centre));
        if !all_normal([0, 1, 2].map(|i| cosine(vecs[i], tans[i]))) {
            return Ok(());
        }
        for i in 0..3 {
            if !curve_side(qs[i], vecs[i].angle(tans[i])?) {
                return Ok(());
            }
        }
        let (d12, d13) = (points[1].distance(points[0]), points[2].distance(points[0]));
        self.accept(qs.map(QCurve::qualifier), points, u, [0.0, d12, d13]);
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(Qualified1, Qualified2, Point3, Param1, Param2,
    /// Tolerance)`: the point is a circle of radius 0 (first function argument, from
    /// PI). Natively the second curve is tested with the first qualifier;
    /// `Circ2d3TanIter::curve_curve_pnt(&q1, &q2, point, 0.0, 0.0, 1e-9)?`.
    pub fn curve_curve_pnt(
        qualified1: &QCurve,
        qualified2: &QCurve,
        point3: Point2,
        [param1, param2]: [f64; 2],
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        check_qualifier(qualified2.qualifier(), MEMBER)?;
        let c1 = circle_at(point3, 0.0)?;
        let mut func =
            FunctionTanCuCuCu::circ_curve_curve(c1, qualified1.curve(), qualified2.curve());
        let (r1, r2) = (
            curve_range(qualified1, tolerance)?,
            curve_range(qualified2, tolerance)?,
        );
        let search = Search {
            umin: [0.0, r1.0, r2.0],
            umax: [2.0 * PI, r1.1, r2.1],
            ufirst: [PI, param1, param2],
            tol: [2.0e-15 * PI, r1.2, r2.2],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.two_curves_point([qualified1, qualified2], point3, u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the curve-curve-point search at `u` (point circle first).
    fn two_curves_point(
        &mut self,
        qs: [&QCurve; 2],
        point3: Point2,
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let (point1, tan1) = CurveTool::d1(qs[0].curve(), u[1])?;
        let (point2, tan2) = CurveTool::d1(qs[1].curve(), u[2])?;
        let Some(cirsol) = circle_through([point3, point1, point2], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let tan3: Vector2 = unit_tangent(u[0]);
        let vec1 = Vector2::between(point1, centre);
        let vec2 = Vector2::between(point2, centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)]) {
            return Ok(());
        }
        if !curve_side(qs[0], vec1.angle(tan1)?) || !curve_side(qs[0], vec2.angle(tan2)?) {
            return Ok(());
        }
        let qualifiers = [qs[0].qualifier(), qs[1].qualifier(), Position::NoQualifier];
        self.accept(
            qualifiers,
            [point1, point2, point3],
            [u[1], u[2], 0.0],
            [0.0; 3],
        );
        Ok(())
    }

    /// `Geom2dGcc_Circ2d3TanIter(Qualified1, Point1, Point2, Param1, Tolerance)`: the
    /// points are circles of radius 0 (from PI). Natively the qualifier is tested on
    /// the tangent of the first point circle, not of the curve;
    /// `Circ2d3TanIter::curve_pnt_pnt(&q, p1, p2, 0.0, 1e-9)?`.
    pub fn curve_pnt_pnt(
        qualified1: &QCurve,
        point1: Point2,
        point2: Point2,
        param1: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::default();
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        let (c1, c2) = (circle_at(point1, 0.0)?, circle_at(point2, 0.0)?);
        let mut func = FunctionTanCuCuCu::circ_circ_curve(c1, c2, qualified1.curve());
        let r1 = curve_range(qualified1, tolerance)?;
        let search = Search {
            umin: [0.0, 0.0, r1.0],
            umax: [2.0 * PI, 2.0 * PI, r1.1],
            ufirst: [PI, PI, param1],
            tol: [2.0e-15 * PI, 2.0e-15 * PI, r1.2],
        };
        if let Some(u) = solve(&mut func, &search, false)? {
            circ.curve_two_points(qualified1, [point1, point2], u, tolerance.abs())?;
        }
        Ok(circ)
    }

    // The solution of the curve-point-point search at `u` (point circles first).
    fn curve_two_points(
        &mut self,
        q1: &QCurve,
        points: [Point2; 2],
        u: [f64; 3],
        tol: f64,
    ) -> Result<(), Failure> {
        let (point3, tan3) = CurveTool::d1(q1.curve(), u[2])?;
        let Some(cirsol) = circle_through([points[0], points[1], point3], tol)? else {
            return Ok(());
        };
        self.cirsol = cirsol;
        let centre: Point2 = cirsol.location();
        let (tan2, tan1) = (unit_tangent(u[1]), unit_tangent(u[0]));
        let vec1 = Vector2::between(points[0], centre);
        let vec2 = Vector2::between(points[1], centre);
        let vec3 = Vector2::between(point3, centre);
        if !all_normal([cosine(vec1, tan1), cosine(vec2, tan2), cosine(vec3, tan3)]) {
            return Ok(());
        }
        if !curve_side(q1, vec1.angle(tan1)?) {
            return Ok(());
        }
        let qualifiers = [q1.qualifier(), Position::NoQualifier, Position::NoQualifier];
        self.accept(
            qualifiers,
            [point3, points[0], points[1]],
            [u[2], 0.0, 0.0],
            [0.0; 3],
        );
        Ok(())
    }
}
