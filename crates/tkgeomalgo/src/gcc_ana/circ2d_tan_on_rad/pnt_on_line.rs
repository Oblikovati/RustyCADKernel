// Rust translation of GccAna_Circ2dTanOnRad_2.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius through a point with their centre on a line.
//!
//! Native quirks kept: a point farther than Radius + Tol sets WellDone and still
//! runs the general case; a point on the line (within Tol) gives two circles but
//! leaves WellDone false and the qualifier slots unset.
use super::Circ2dTanOnRad;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2d, Circle2, Line2, Point2};
use rustycad_tkmath::math::DirectPolynomialRoots;

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Point1, OnLine, Radius, Tolerance): circles of radius
    /// `radius` through `point1`, centred on `on_line`;
    /// `Circ2dTanOnRad::pnt_on_line(point, line, radius, tol)?`.
    pub fn pnt_on_line(
        point1: Point2,
        on_line: Line2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let tol: f64 = tolerance.abs();
        let dp1lin: f64 = on_line.distance(point1);
        Self::check_radius(radius)?;
        if dp1lin > radius + tol {
            circ.well_done = true;
        }
        if (dp1lin - radius).abs() < tol {
            circ.touching_line(point1, on_line, radius, dp1lin)?;
        } else if dp1lin < tol {
            circ.point_on_line(point1, on_line, radius)?;
        } else {
            circ.general(point1, on_line, radius)?;
        }
        Ok(circ)
    }

    // The point at distance Radius: one circle, its centre the foot of the point.
    fn touching_line(
        &mut self,
        point1: Point2,
        on_line: Line2,
        radius: f64,
        dp1lin: f64,
    ) -> Result<(), Failure> {
        let (x1, y1) = (point1.x(), point1.y());
        let (xdir, ydir) = (on_line.direction().x(), on_line.direction().y());
        let (lxloc, lyloc) = (on_line.location().x(), on_line.location().y());
        self.well_done = true;
        self.nbr_sol = 1;
        let center = if -ydir * (x1 - lxloc) + xdir * (y1 - lyloc) < 0.0 {
            Point2::new(x1 - ydir * dp1lin, y1 + xdir * dp1lin)
        } else {
            Point2::new(x1 + ydir * dp1lin, y1 - xdir * dp1lin)
        };
        self.through_point(1, circle_at(center, radius)?, point1, on_line)?;
        self.qualifier1.set(1, Position::NoQualifier)
    }

    // The point on the line: the two circles centred on the line at +-Radius.
    fn point_on_line(
        &mut self,
        point1: Point2,
        on_line: Line2,
        radius: f64,
    ) -> Result<(), Failure> {
        let (xdir, ydir) = (on_line.direction().x(), on_line.direction().y());
        let c1 = Point2::new(point1.x() + radius * xdir, point1.y() + radius * ydir);
        let c2 = Point2::new(point1.x() - radius * xdir, point1.y() - radius * ydir);
        let c1: Point2 = elclib::value(elclib::parameter(&on_line, c1)?, &on_line);
        let c2: Point2 = elclib::value(elclib::parameter(&on_line, c2)?, &on_line);
        let circle1 = Circle2::from_x_axis(Axis2d::new(c1, on_line.direction()), radius, true)?;
        let circle2 = Circle2::from_x_axis(Axis2d::new(c2, on_line.direction()), radius, true)?;
        self.through_point(1, circle1, point1, on_line)?;
        self.through_point(2, circle2, point1, on_line)?;
        self.nbr_sol = 2;
        Ok(())
    }

    // The general case: centres (xc, yc) on the line at distance Radius from the
    // point, from a quadratic in y (in x for a line of equation coefficient A = 0,
    // whose coordinates are swapped).
    fn general(&mut self, point1: Point2, on_line: Line2, radius: f64) -> Result<(), Failure> {
        let (mut x1, mut y1) = (point1.x(), point1.y());
        let (mut a, mut b, c) = on_line.coefficients();
        let d: f64 = a;
        if a == 0.0 {
            (a, b) = (b, d);
            (x1, y1) = (y1, x1);
        }
        if a == 0.0 {
            return Ok(());
        }
        let sol = DirectPolynomialRoots::quadratic(
            (b * b + a * a) / (a * a),
            2.0 * (b * c / (a * a) + (b / a) * x1 - y1),
            x1 * x1 + y1 * y1 + c * c / (a * a) - radius * radius + 2.0 * c * x1 / a,
        );
        if !sol.is_done() {
            return Ok(());
        }
        for i in 1..=sol.solutions()? {
            let root: f64 = sol.value(i)?;
            let center = if d != 0.0 {
                Point2::new(-(b / a) * root - c / a, root)
            } else {
                Point2::new(root, -(b / a) * root - c / a)
            };
            self.nbr_sol += 1;
            let index: i32 = self.nbr_sol;
            self.through_point(index, circle_at(center, radius)?, point1, on_line)?;
            self.qualifier1.set(index, Position::NoQualifier)?;
        }
        self.well_done = true;
        Ok(())
    }

    // Store a circle through the point: tangency point Point1 (argument parameter 0)
    // and its centre on the locus.
    fn through_point(
        &mut self,
        index: i32,
        solution: Circle2,
        point1: Point2,
        locus: Line2,
    ) -> Result<(), Failure> {
        self.cirsol.set(index, solution)?;
        let centre: Point2 = solution.location();
        let par_sol: f64 = elclib::parameter(&solution, point1)?;
        self.tangency1.record(index, point1, par_sol, 0.0)?;
        self.centre_on(index, centre, elclib::parameter(&locus, centre)?)
    }
}
