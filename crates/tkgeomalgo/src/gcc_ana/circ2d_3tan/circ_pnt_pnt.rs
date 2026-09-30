// Rust translation of GccAna_Circ2d3Tan_7.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a circle and through two points: the intersections of
//! the bisector of the points with the bisectors of (C1, Point2), at most two
//! solutions.
use super::candidates::RadiusCandidates;
use super::{Argument, Circ2d3Tan};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{CircPnt2dBisec, Pnt2dBisec, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

struct Args {
    q1: QualifiedCirc,
    point2: Point2,
    point3: Point2,
    tolerance: f64,
}

// Intp.Perform(Sol1, <circle point bisector>) for its four possible types.
fn intersect(sol1: Line2, sol2: &Handle<dyn Bisec>) -> Result<AnaIntersection, Failure> {
    Ok(match sol2.arc_type() {
        IType::Cir => AnaIntersection::line_circle(sol1, sol2.circle()?)?,
        IType::Lin => AnaIntersection::lines(sol1, sol2.line()?),
        IType::Hpr => AnaIntersection::line_conic(sol1, &Conic::from_hyperbola(sol2.hyperbola()?))?,
        IType::Ell => AnaIntersection::line_conic(sol1, &Conic::from_ellipse(sol2.ellipse()?))?,
        _ => AnaIntersection::new(),
    })
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Point2, Point3, Tolerance): the circles
    /// tangent to a circle and through two points (not done for coincident points);
    /// `Circ2d3Tan::circ_pnt_pnt(q1, p2, p3, tol)?`.
    pub fn circ_pnt_pnt(
        qualified1: QualifiedCirc,
        point2: Point2,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        check_qualifier(qualified1.qualifier(), super::NAME)?;
        if point2.is_equal(point3, precision::confusion()) {
            return Ok(circ);
        }
        let args = Args {
            q1: qualified1,
            point2,
            point3,
            tolerance,
        };
        let bis1 = Pnt2dBisec::new(point2, point3)?;
        let bis2 = CircPnt2dBisec::new(qualified1.qualified(), point2);
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        let mut radius = RadiusCandidates::new();
        for i in 1..=bis2.nb_solutions() {
            let sol2: Handle<dyn Bisec> = bis2.this_solution(i)?;
            let intp: AnaIntersection = intersect(bis1.this_solution()?, &sol2)?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    if !circ.centre_7(&args, intp.point(j)?.value(), &mut radius)? {
                        break;
                    }
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    // One candidate centre; false stops the scan (two solutions already).
    fn centre_7(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut RadiusCandidates,
    ) -> Result<bool, Failure> {
        let c1 = a.q1.qualified();
        let tol: f64 = a.tolerance.abs();
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = center.distance(a.point2);
        let dist3: f64 = center.distance(a.point3);
        let nbsol1: i32 = radius.first_circle(a.q1, dist1, a.tolerance, true);
        let mut nbsol3: i32 = 0;
        if radius.ok {
            radius.ok = false;
            for ii in 1..=nbsol1 {
                if (dist2 - radius.at(ii)).abs() <= tol && (dist3 - radius.at(ii)).abs() <= tol {
                    nbsol3 = ii;
                    radius.ok = true;
                }
            }
        }
        if !radius.ok {
            return Ok(true);
        }
        if self.nbr_sol >= 2 {
            return Ok(false);
        }
        self.solution_7(a, center, radius.at(nbsol3))?;
        Ok(true)
    }

    fn solution_7(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let c1 = a.q1.qualified();
        let tol: f64 = a.tolerance.abs();
        let index: i32 = self.push_circle(center, r)?;
        let q1: Position = qualify(
            a.q1.qualifier(),
            center.distance(c1.location()),
            r,
            c1.radius(),
            tol,
            false,
        );
        self.set_qualifier(1, index, q1)?;
        self.set_qualifier(2, index, Position::NoQualifier)?;
        self.set_qualifier(3, index, Position::NoQualifier)?;
        if center.distance(c1.location()) <= a.tolerance && (r - c1.radius()).abs() <= a.tolerance {
            self.mark_same(1, index)?;
        } else {
            // A solution inside circle 1 touches it on the far side.
            let mut dc = Direction2::from_xy(c1.location().xy() - center.xy())?;
            if q1 == Position::Enclosed {
                dc.reverse();
            }
            self.touch(
                1,
                index,
                Point2::from(center.xy() + r * dc.xy()),
                Argument::Circle(c1),
            )?;
        }
        self.touch(2, index, a.point2, Argument::Point)?;
        self.touch(3, index, a.point3, Argument::Point)
    }
}
