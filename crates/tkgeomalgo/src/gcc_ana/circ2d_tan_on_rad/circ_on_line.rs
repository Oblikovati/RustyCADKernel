// Rust translation of the circle-on-line constructor of GccAna_Circ2dTanOnRad.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a circle with their centre on a line: the
//! centres are at distance |Radius -+ R1| from the circle centre, found as the
//! roots of a quadratic along the line.
use super::{Circ2dTanOnRad, NAME};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{circle_at, qualify};
use crate::gcc_ent::QualifiedCirc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};
use rustycad_tkmath::math::DirectPolynomialRoots;

// The outcome of the qualifier analysis: a single tangent centre (with the side
// of its tangency point), the squared centre distances to solve for, or nothing.
enum Plan {
    None,
    Single(Point2, f64),
    Coefficients(Vec<f64>),
}

// The centre of the circle tangent to the line on the side of C1: the foot of the
// perpendicular from C1 moved by `dist` across the line.
fn foot_centre(c1: Point2, on_line: Line2, dist: f64) -> Point2 {
    let (xdir, ydir) = (on_line.direction().x(), on_line.direction().y());
    let (lxloc, lyloc) = (on_line.location().x(), on_line.location().y());
    let (x1, y1) = (c1.x(), c1.y());
    if -ydir * (x1 - lxloc) + xdir * (y1 - lyloc) < 0.0 {
        return Point2::new(x1 - ydir * dist, y1 + xdir * dist);
    }
    Point2::new(x1 + ydir * dist, y1 - xdir * dist)
}

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Qualified1, OnLine, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified circle, centred on `on_line`;
    /// Standard_NegativeValue for a negative radius;
    /// `Circ2dTanOnRad::circ_on_line(q1, line, radius, tol)?`.
    pub fn circ_on_line(
        qualified1: QualifiedCirc,
        on_line: Line2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), NAME)?;
        Self::check_radius(radius)?;
        match plan(qualified1, on_line, radius, tol, &mut circ) {
            Plan::None => {}
            Plan::Single(center, signe) => {
                circ.single(qualified1, on_line, radius, tol, (center, signe))?
            }
            Plan::Coefficients(coef) => {
                for c in coef {
                    circ.roots_on_line(qualified1, on_line, radius, tol, c)?;
                }
            }
        }
        Ok(circ)
    }

    fn single(
        &mut self,
        qualified1: QualifiedCirc,
        on_line: Line2,
        radius: f64,
        tol: f64,
        (center, signe): (Point2, f64),
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(1, solution)?;
        let distcc1: f64 = center.distance(c1.location());
        self.qualifier1.set(
            1,
            qualify(qualified1.qualifier(), distcc1, radius, r1, tol, false),
        )?;
        if (radius - r1).abs() <= tol {
            self.tangency1.same.set(1, 1)?;
        } else {
            let dir1cen = Direction2::new(
                center.x() - c1.location().x(),
                center.y() - c1.location().y(),
            )?;
            let point = Point2::from(center.xy() + signe * radius * dir1cen.xy());
            let par_sol: f64 = elclib::parameter(&solution, point)?;
            let par_arg: f64 = elclib::parameter(&c1, point)?;
            self.tangency1.record(1, point, par_sol, par_arg)?;
        }
        let centre: Point2 = self.cirsol.get(self.nbr_sol)?.location();
        self.centre_on(1, centre, elclib::parameter(&on_line, centre)?)
    }

    // The centres on the line at squared distance `coef` from the circle centre.
    fn roots_on_line(
        &mut self,
        qualified1: QualifiedCirc,
        on_line: Line2,
        radius: f64,
        tol: f64,
        coef: f64,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let (x1, y1) = (c1.location().x(), c1.location().y());
        let (mut a, mut b, c) = on_line.coefficients();
        let d: f64 = a;
        let (x0, y0) = if d.abs() <= tol {
            (a, b) = (b, d);
            (y1, x1)
        } else {
            (x1, y1)
        };
        let sol = DirectPolynomialRoots::quadratic(
            (b * b + a * a) / (a * a),
            2.0 * (b * c / (a * a) + (b / a) * x0 - y0),
            x0 * x0 + y0 * y0 + c * c / (a * a) - coef + 2.0 * c * x0 / a,
        );
        if !sol.is_done() {
            return Ok(());
        }
        for i in 1..=sol.solutions()? {
            let root: f64 = sol.value(i)?;
            let center = if d.abs() > tol {
                Point2::new(-(b / a) * root - c / a, root)
            } else {
                Point2::new(root, -(b / a) * root - c / a)
            };
            if on_line.distance(center) > tol {
                continue;
            }
            self.root_solution(qualified1, on_line, radius, tol, center)?;
        }
        self.well_done = true;
        Ok(())
    }

    fn root_solution(
        &mut self,
        qualified1: QualifiedCirc,
        on_line: Line2,
        radius: f64,
        tol: f64,
        center: Point2,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let distcc1: f64 = center.distance(c1.location());
        self.qualifier1.set(
            index,
            qualify(qualified1.qualifier(), distcc1, radius, r1, tol, false),
        )?;
        let dir1cen = Direction2::new(
            center.x() - c1.location().x(),
            center.y() - c1.location().y(),
        )?;
        let point = if (radius > r1) || (center.distance(c1.location()) > r1) {
            Point2::from(center.xy() + radius * dir1cen.xy())
        } else {
            Point2::from(center.xy() - radius * dir1cen.xy())
        };
        let centre: Point2 = solution.location();
        let par_sol: f64 = elclib::parameter(&solution, point)?;
        let par_arg: f64 = elclib::parameter(&c1, point)?;
        self.tangency1.record(index, point, par_sol, par_arg)?;
        self.centre_on(index, centre, elclib::parameter(&on_line, centre)?)
    }
}

// The native analysis of the qualifier: WellDone and NbrSol for the cases it
// settles, the plan for the others.
fn plan(
    qualified1: QualifiedCirc,
    on_line: Line2,
    radius: f64,
    tol: f64,
    circ: &mut Circ2dTanOnRad,
) -> Plan {
    let c1: Circle2 = qualified1.qualified();
    let r1: f64 = c1.radius();
    let dist: f64 = on_line.distance(c1.location());
    let foot = |circ: &mut Circ2dTanOnRad, signe: f64| {
        circ.well_done = true;
        circ.nbr_sol = 1;
        Plan::Single(foot_centre(c1.location(), on_line, dist), signe)
    };
    if qualified1.is_enclosed() {
        if tol < radius - r1 + dist {
            circ.well_done = true;
            return Plan::None;
        }
        if (radius - r1 + dist).abs() < tol {
            return foot(circ, 1.0);
        }
        return Plan::Coefficients(vec![(r1 - radius) * (r1 - radius)]);
    }
    if qualified1.is_enclosing() {
        if r1 + dist - radius > tol {
            circ.well_done = true;
            return Plan::None;
        }
        if r1 + dist - radius > 0.0 {
            return foot(circ, -1.0);
        }
        return Plan::Coefficients(vec![(radius - r1) * (radius - r1)]);
    }
    if dist - r1 - radius > tol {
        circ.well_done = false;
        return Plan::None;
    }
    if (dist - r1 - radius).abs() < tol {
        return foot(circ, -1.0);
    }
    if qualified1.is_outside() {
        return Plan::Coefficients(vec![(radius + r1) * (radius + r1)]);
    }
    Plan::Coefficients(vec![
        (radius - r1) * (radius - r1),
        (radius + r1) * (radius + r1),
    ])
}
