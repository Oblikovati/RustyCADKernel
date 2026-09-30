// Rust translation of GccAna_Circ2d3Tan_5.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to a circle and a line and through a point: the
//! intersections of the bisectors of (C1, L2) and of (L2, Point3), at most 4
//! solutions with radii in [1e-6, 1e10].
use super::candidates::RadiusCandidates;
use super::{Argument, Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of, side_of};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{CircLin2dBisec, LinPnt2dBisec, check_qualifier};
use crate::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::gp::{Direction2, Point2};

// pop: no solution with a radius above MaxRad or below MinRad.
const MAX_RAD: f64 = 1e10;
const MIN_RAD: f64 = 1e-6;

struct Args {
    q1: QualifiedCirc,
    q2: QualifiedLin,
    point3: Point2,
    tolerance: f64,
}

// Intp.Perform for the line and parabola bisectors of this file.
fn intersect(
    sol1: &Handle<dyn Bisec>,
    sol2: &Handle<dyn Bisec>,
) -> Result<AnaIntersection, Failure> {
    Ok(match (sol1.arc_type(), sol2.arc_type()) {
        (IType::Lin, IType::Lin) => AnaIntersection::lines(sol1.line()?, sol2.line()?),
        (IType::Lin, IType::Par) => {
            AnaIntersection::line_conic(sol1.line()?, &Conic::from_parabola(sol2.parabola()?))?
        }
        (IType::Par, IType::Lin) => {
            AnaIntersection::line_conic(sol2.line()?, &Conic::from_parabola(sol1.parabola()?))?
        }
        (IType::Par, IType::Par) => AnaIntersection::parabola_conic(
            sol1.parabola()?,
            &Conic::from_parabola(sol2.parabola()?),
        )?,
        _ => AnaIntersection::new(),
    })
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Point3, Tolerance) with a circle,
    /// a line and a point, up to 4 solutions;
    /// `Circ2d3Tan::circ_lin_pnt(q1, l2, point, tol)?`.
    pub fn circ_lin_pnt(
        qualified1: QualifiedCirc,
        qualified2: QualifiedLin,
        point3: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_qualifier(qualified1.qualifier(), super::NAME)?;
        check_line_qualifier(qualified2)?;
        let args = Args {
            q1: qualified1,
            q2: qualified2,
            point3,
            tolerance,
        };
        let l2 = qualified2.qualified();
        let bis1 = CircLin2dBisec::new(qualified1.qualified(), l2);
        let bis2 = LinPnt2dBisec::new(l2, point3)?;
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        let mut radius = RadiusCandidates::new();
        for i in 1..=bis1.nb_solutions() {
            let intp: AnaIntersection = intersect(&bis1.this_solution(i)?, &bis2.this_solution()?)?;
            if intp.is_done() {
                if !intp.is_empty()? {
                    for j in 1..=intp.nb_points()? {
                        circ.centre_5(&args, intp.point(j)?.value(), &mut radius)?;
                    }
                }
                circ.well_done = true;
            }
            if circ.nbr_sol == 4 {
                break;
            }
        }
        Ok(circ)
    }

    fn centre_5(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut RadiusCandidates,
    ) -> Result<(), Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = l2.distance(center);
        let dist3: f64 = center.distance(a.point3);
        let nbsol1: i32 = radius.first_circle(a.q1, dist1, a.tolerance, false);
        radius.line_radius(a.q2, side_of(l2, center), dist2, nbsol1, tol);
        let nbsol3: i32 = i32::from((dist3 - radius.at(1)).abs() <= tol && radius.ok);
        if !radius.ok {
            return Ok(());
        }
        for k in 1..=nbsol3 {
            let r: f64 = radius.at(k);
            if self.nbr_sol == 4 || r > MAX_RAD || r.abs() < MIN_RAD {
                break;
            }
            self.solution_5(a, center, r)?;
        }
        Ok(())
    }

    fn solution_5(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, l2) = (a.q1.qualified(), a.q2.qualified());
        let tol: f64 = a.tolerance.abs();
        let index: i32 = self.push_circle(center, r)?;
        let distcc1: f64 = center.distance(c1.location());
        self.set_qualifier(
            1,
            index,
            qualify(a.q1.qualifier(), distcc1, r, c1.radius(), tol, false),
        )?;
        self.set_qualifier(2, index, line_qualifier(a.q2, center, normal_of(l2)?)?)?;
        self.set_qualifier(3, index, Position::NoQualifier)?;
        if center.distance(c1.location()) <= a.tolerance && (r - c1.radius()).abs() <= a.tolerance {
            self.mark_same(1, index)?;
        } else {
            // NIZHNY-EAP 1999: the direction goes from C1 to the centre.
            let dc = Direction2::from_xy(center.xy() - c1.location().xy())?;
            self.touch(
                1,
                index,
                Point2::from(center.xy() + r * dc.xy()),
                Argument::Circle(c1),
            )?;
        }
        self.touch_line(2, index, l2)?;
        self.touch(3, index, a.point3, Argument::Point)
    }
}
