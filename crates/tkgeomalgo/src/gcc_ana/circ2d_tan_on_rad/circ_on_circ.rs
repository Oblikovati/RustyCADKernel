// Rust translation of GccAna_Circ2dTanOnRad_3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a circle with their centre on another
//! circle: the intersections of the locus with the circles concentric to C1 at
//! |R1 +- Radius|, or the single centre on the line of centres.
//!
//! Native quirk kept: the single-centre case writes the qualifier of slot NbrSol,
//! which is 0 at that point, so it raises Standard_OutOfRange.
use super::{Circ2dTanOnRad, NAME};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::{Slots, circle_at, qualify};
use crate::gcc_ent::QualifiedCirc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2, StandardDirection2};

// The native analysis: centre on the line of centres (`sign`), concentric
// circles to intersect (`disparal`), or the argument itself.
#[derive(Default)]
struct Plan {
    sign: f64,
    disparal: Vec<f64>,
    dir1on: Option<Direction2>,
}

// The per-solution arrays of the constructor (Center, dir1on, signe).
struct Work {
    center: Slots<Point2>,
    dir1on: Slots<Direction2>,
    signe: [i32; 5],
}

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Qualified1, OnCirc, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified circle, centred on `on_circ`;
    /// `Circ2dTanOnRad::circ_on_circ(q1, circle, radius, tol)?`.
    pub fn circ_on_circ(
        qualified1: QualifiedCirc,
        on_circ: Circle2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        let tol: f64 = tolerance.abs();
        check_qualifier(qualified1.qualifier(), NAME)?;
        Self::check_radius(radius)?;
        let plan: Plan = circ.plan_3(qualified1, on_circ, radius, tol)?;
        let mut work = Work {
            center: Slots::new(4),
            dir1on: Slots::new(4),
            signe: [0; 5],
        };
        if let Some(direction) = plan.dir1on {
            work.dir1on.set(1, direction)?;
        }
        let c1: Circle2 = qualified1.qualified();
        for &distance in &plan.disparal {
            circ.concentric_3(qualified1, on_circ, radius, tol, distance, &mut work)?;
        }
        if plan.sign != 0.0 {
            let center = Point2::from(
                on_circ.location().xy() + plan.sign * on_circ.radius() * work.dir1on.get(1)?.xy(),
            );
            work.center.set(1, center)?;
            circ.cirsol.set(1, circle_at(center, radius)?)?;
            let distcc1: f64 = center.distance(c1.location());
            let position = qualify(
                qualified1.qualifier(),
                distcc1,
                radius,
                c1.radius(),
                tol,
                false,
            );
            circ.qualifier1.set(circ.nbr_sol, position)?;
            circ.nbr_sol = 1;
            work.signe[0] = 1;
            circ.well_done = true;
        }
        circ.tangency_points_3(c1, on_circ, radius, &work)?;
        Ok(circ)
    }

    fn plan_3(
        &mut self,
        qualified1: QualifiedCirc,
        on_circ: Circle2,
        radius: f64,
        tol: f64,
    ) -> Result<Plan, Failure> {
        let c1: Circle2 = qualified1.qualified();
        let (r1, r2) = (c1.radius(), on_circ.radius());
        let center1: Point2 = c1.location();
        let dist: f64 = on_circ.location().distance(center1);
        let on_to_c1 = || {
            Direction2::new(
                center1.x() - on_circ.location().x(),
                center1.y() - on_circ.location().y(),
            )
        };
        let mut plan = Plan::default();
        if qualified1.is_enclosed() {
            if (radius - r1 > tol) || (r1 - dist - r2 - radius > tol) {
                self.well_done = true;
            } else if (radius - r1).abs() < tol {
                self.same_circle(qualified1, on_circ, tol, false)?;
            } else if (r1 - dist - r2 - radius).abs() <= tol {
                plan.dir1on = Some(on_to_c1()?);
                plan.sign = -1.0;
            } else {
                plan.disparal = vec![(r1 - radius).abs()];
            }
        } else if qualified1.is_enclosing() {
            if (tol < r1 - radius)
                || (tol < r1 + dist - r2 - radius)
                || (radius - r1 - dist - r2 > tol)
            {
                self.well_done = true;
            } else if (radius - r1).abs() < tol {
                self.same_circle(qualified1, on_circ, tol, true)?;
            } else if ((r1 + dist - r2 - radius).abs() < tol)
                || ((radius - r1 - dist - r2).abs() < tol)
            {
                plan.dir1on = Some(on_to_c1()?);
                plan.sign = if (r1 + dist - r2 - radius).abs() < tol {
                    1.0
                } else {
                    -1.0
                };
            } else {
                plan.disparal = vec![(r1 - radius).abs()];
            }
        } else if qualified1.is_outside() {
            if (dist - r2 - r1 - radius > tol) || (radius - dist - r2 + r1 > tol) {
                self.well_done = true;
            } else {
                plan.dir1on = Some(on_to_c1()?);
                if (r1 - dist - r2 + radius).abs() < tol {
                    plan.sign = -1.0;
                } else if (dist - r1 - r2 - radius).abs() < tol {
                    plan.sign = 1.0;
                } else {
                    plan.disparal = vec![(r1 + radius).abs()];
                }
            }
        } else if (dist - r2 - r1 - radius > tol)
            || (radius - dist - r1 - r2 > tol)
            || ((r1 - dist - r2 > tol) && (tol < r1 - r2 - dist - radius))
        {
            self.well_done = true;
        } else {
            plan.dir1on = Some(on_to_c1()?);
            if (dist - r1 - r2 - radius).abs() < tol {
                plan.sign = 1.0;
            } else if (dist + r1 + r2 - radius < 0.0)
                || ((r1 - dist + r2 > 0.0) && (r1 - r2 - dist - radius > 0.0))
            {
                plan.sign = -1.0;
            } else {
                plan.disparal = vec![(r1 + radius).abs(), (r1 - radius).abs()];
            }
        }
        Ok(plan)
    }

    // Radius = R1: the argument itself when its centre lies on the locus (an
    // enclosed circle off the locus leaves WellDone false, an enclosing one true).
    fn same_circle(
        &mut self,
        qualified1: QualifiedCirc,
        on_circ: Circle2,
        tol: f64,
        enclosing: bool,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let on_locus: bool = on_circ.distance(c1.location()) < tol;
        if !on_locus {
            self.well_done = enclosing;
            return Ok(());
        }
        self.nbr_sol = 1;
        self.cirsol.set(1, c1)?;
        self.qualifier1.set(1, qualified1.qualifier())?;
        self.tangency1.same.set(1, 1)?;
        self.centre_on(
            1,
            c1.location(),
            elclib::parameter(&on_circ, c1.location())?,
        )?;
        self.well_done = true;
        Ok(())
    }

    // The centres on the locus at `distance` from the centre of C1.
    fn concentric_3(
        &mut self,
        qualified1: QualifiedCirc,
        on_circ: Circle2,
        radius: f64,
        tol: f64,
        distance: f64,
        work: &mut Work,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let (r1, center1) = (c1.radius(), c1.location());
        let dirx = Direction2::from_standard(StandardDirection2::X);
        let ring = Circle2::from_x_axis(
            rustycad_tkmath::gp::Axis2d::new(center1, dirx),
            distance,
            true,
        )?;
        let intp = AnaIntersection::circles(on_circ, ring)?;
        if !intp.is_done() {
            return Ok(());
        }
        if !intp.is_empty()? {
            for i in 1..=intp.nb_points()? {
                self.nbr_sol += 1;
                let index: i32 = self.nbr_sol;
                let center: Point2 = intp.point(i)?.value();
                work.center.set(index, center)?;
                self.cirsol.set(index, circle_at(center, radius)?)?;
                let distcc1: f64 = center.distance(center1);
                self.qualifier1.set(
                    index,
                    qualify(qualified1.qualifier(), distcc1, radius, r1, tol, false),
                )?;
                work.dir1on.set(
                    index,
                    Direction2::new(center1.x() - center.x(), center1.y() - center.y())?,
                )?;
                let inside: bool = (r1 > radius) && (center.distance(center1) <= r1);
                // index <= 4 here: Center(1, 4) has already accepted it.
                work.signe[(index - 1) as usize] = if inside { -1 } else { 1 };
            }
        }
        self.well_done = true;
        Ok(())
    }

    // The tangency points of the solutions flagged in `signe` (a 0 ends the list).
    fn tangency_points_3(
        &mut self,
        c1: Circle2,
        on_circ: Circle2,
        radius: f64,
        work: &Work,
    ) -> Result<(), Failure> {
        let mut ii: usize = 0;
        while work.signe[ii] != 0 {
            let index: i32 = ii as i32 + 1;
            let signe: f64 = f64::from(work.signe[ii]);
            let point = Point2::from(
                work.center.get(index)?.xy() + signe * radius * work.dir1on.get(index)?.xy(),
            );
            let solution: Circle2 = self.cirsol.get(index)?;
            let centre: Point2 = solution.location();
            let par_arg: f64 = elclib::parameter(&c1, point)?;
            let par_sol: f64 = elclib::parameter(&solution, point)?;
            let same: i32 = self.tangency1.same.get(index)?;
            self.tangency1.record(index, point, par_sol, par_arg)?;
            self.tangency1.same.set(index, same)?;
            self.centre_on(index, centre, elclib::parameter(&on_circ, centre)?)?;
            ii += 1;
        }
        Ok(())
    }
}
