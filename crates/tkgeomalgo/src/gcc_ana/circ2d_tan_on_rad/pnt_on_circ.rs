// Rust translation of GccAna_Circ2dTanOnRad_5.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius through a point with their centre on a circle.
//!
//! Native quirk kept: in the general case every solution writes the qualifier,
//! tangency and centre data of slot 1, so only the circles of the other slots
//! are set (their data stays unset: 0 here, uninitialized natively).
use super::Circ2dTanOnRad;
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Point2};

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Point1, OnCirc, Radius, Tolerance): circles of radius
    /// `radius` through `point1`, centred on `on_circ`;
    /// `Circ2dTanOnRad::pnt_on_circ(point, circle, radius, tol)?`.
    pub fn pnt_on_circ(
        point1: Point2,
        on_circ: Circle2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let tol: f64 = tolerance.abs();
        let roncirc: f64 = on_circ.radius();
        let dist1: f64 = point1.distance(on_circ.location()) - roncirc;
        let dist2: f64 = point1.distance(on_circ.location()) + roncirc;
        Self::check_radius(radius)?;
        if (dist1 - radius > tol) || (tol < radius - dist2) {
            circ.well_done = true;
            return Ok(circ);
        }
        let signe: f64 = if (dist1 - radius).abs() < tol {
            1.0
        } else if (dist2 - radius).abs() < tol {
            -1.0
        } else {
            0.0
        };
        if signe != 0.0 {
            let (ox, oy) = (on_circ.location().x(), on_circ.location().y());
            let towards = Direction2::new(ox - signe * point1.x(), oy - signe * point1.y())?;
            let center = Point2::from(on_circ.location().xy() - roncirc * towards.xy());
            circ.cirsol.set(1, circle_at(center, radius)?)?;
            circ.first_slot(point1, on_circ)?;
            circ.well_done = true;
            circ.nbr_sol = 1;
            return Ok(circ);
        }
        let dirx = Direction2::from_standard(rustycad_tkmath::gp::StandardDirection2::X);
        let around =
            Circle2::from_x_axis(rustycad_tkmath::gp::Axis2d::new(point1, dirx), radius, true)?;
        let intp = AnaIntersection::circles(on_circ, around)?;
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            for i in 1..=intp.nb_points()? {
                circ.nbr_sol += 1;
                let center: Point2 = intp.point(i)?.value();
                circ.cirsol.set(circ.nbr_sol, circle_at(center, radius)?)?;
                circ.first_slot(point1, on_circ)?;
            }
        }
        circ.well_done = true;
        Ok(circ)
    }

    // The qualifier, tangency and centre data of slot 1 (cirsol(1) already set).
    fn first_slot(&mut self, point1: Point2, on_circ: Circle2) -> Result<(), Failure> {
        let solution: Circle2 = self.cirsol.get(1)?;
        self.qualifier1.set(1, Position::NoQualifier)?;
        let centre: Point2 = solution.location();
        let par_sol: f64 = elclib::parameter(&solution, point1)?;
        self.tangency1.record(1, point1, par_sol, 0.0)?;
        self.centre_on(1, centre, elclib::parameter(&on_circ, centre)?)
    }
}
