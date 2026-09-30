// Rust translation of GccAna_Circ2d2TanOn_11.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles through two points with their centre on a circle: the intersections of
//! the locus with the perpendicular bisector of the points. Native returns early
//! (done, no solution) for coincident points and for points whose distances to
//! the locus centre differ by its diameter, so its enlarged-locus branch for
//! those cases is never reached.
use super::Circ2d2TanOn;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Point1, Point2, OnCirc, Tolerance): circles through two
    /// points centred on `on_circ`, at most two;
    /// `Circ2d2TanOn::pnt_pnt_on_circ(p1, p2, circle, tol)?`.
    pub fn pnt_pnt_on_circ(
        point1: Point2,
        point2: Point2,
        on_circ: Circle2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let tol: f64 = tolerance.abs();
        let dist: f64 = point1.distance(point2);
        let dp1cen: f64 = point1.distance(on_circ.location());
        let dp2cen: f64 = point2.distance(on_circ.location());
        let r: f64 = on_circ.radius();
        if dist < tol
            || (dp1cen + 2.0 * r - dp2cen).abs() < tol
            || (dp2cen + 2.0 * r - dp1cen).abs() < tol
        {
            circ.well_done = true;
            return Ok(circ);
        }
        let middle = Point2::from((point1.xy() + point2.xy()) / 2.0);
        let l1 = Line2::new(
            middle,
            Direction2::new(point1.y() - point2.y(), point2.x() - point1.x())?,
        );
        let intp = AnaIntersection::line_circle(l1, on_circ)?;
        if !intp.is_done() {
            return Ok(circ);
        }
        if !intp.is_empty()? {
            let locus = move |centre: Point2| elclib::parameter(&on_circ, centre);
            for i in 1..=intp.nb_points()? {
                circ.through_both(intp.point(i)?.value(), (point1, point2), &locus)?;
            }
        }
        circ.well_done = true;
        Ok(circ)
    }
}
