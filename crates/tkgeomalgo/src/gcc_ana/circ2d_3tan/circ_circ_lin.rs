// Rust translation of GccAna_Circ2d3Tan_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circles tangent to two circles and a line: the intersections of the
//! circle-line bisectors of (C1, L3) and (C2, L3).
use super::candidates::RadiusCandidates;
use super::{Circ2d3Tan, check_line_qualifier, line_qualifier, normal_of, side_of};
use crate::gcc_ana::solutions::qualify;
use crate::gcc_ana::{CircLin2dBisec, check_qualifier};
use crate::gcc_ent::{QualifiedCirc, QualifiedLin};
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

// Intp.Perform for the bisector types of this file (lines and parabolas); None
// when no branch applies, so that the previous result is kept as natively.
fn intersect(
    sol1: &Handle<dyn Bisec>,
    sol2: &Handle<dyn Bisec>,
) -> Result<Option<AnaIntersection>, Failure> {
    let result = match (sol1.arc_type(), sol2.arc_type()) {
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
        _ => return Ok(None),
    };
    Ok(Some(result))
}

// The arguments of the constructor.
struct Args {
    q1: QualifiedCirc,
    q2: QualifiedCirc,
    q3: QualifiedLin,
    tolerance: f64,
}

impl Circ2d3Tan {
    /// GccAna_Circ2d3Tan(Qualified1, Qualified2, Qualified3, Tolerance) with two
    /// circles and a line, up to 16 solutions. Centres beyond 1e10 stop the scan
    /// of their intersection, as natively; `Circ2d3Tan::circ_circ_lin(q1, q2, l3, tol)?`.
    pub fn circ_circ_lin(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        qualified3: QualifiedLin,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(16);
        check_qualifier(qualified1.qualifier(), super::NAME)?;
        check_qualifier(qualified2.qualifier(), super::NAME)?;
        check_line_qualifier(qualified3)?;
        let args = Args {
            q1: qualified1,
            q2: qualified2,
            q3: qualified3,
            tolerance,
        };
        let l3: Line2 = qualified3.qualified();
        let bis1 = CircLin2dBisec::new(qualified1.qualified(), l3);
        let bis2 = CircLin2dBisec::new(qualified2.qualified(), l3);
        let mut radius = RadiusCandidates::new();
        if !(bis1.is_done() && bis2.is_done()) {
            return Ok(circ);
        }
        for i in 1..=bis1.nb_solutions() {
            let sol1: Handle<dyn Bisec> = bis1.this_solution(i)?;
            let mut intp = AnaIntersection::new();
            for k in 1..=bis2.nb_solutions() {
                let sol2: Handle<dyn Bisec> = bis2.this_solution(k)?;
                if let Some(result) = intersect(&sol1, &sol2)? {
                    intp = result;
                }
                if !intp.is_done() {
                    continue;
                }
                if !intp.is_empty()? {
                    for j in 1..=intp.nb_points()? {
                        let center: Point2 = intp.point(j)?.value();
                        if !circ.centre_1(&args, center, &mut radius)? {
                            break;
                        }
                    }
                }
                circ.well_done = true;
            }
        }
        Ok(circ)
    }

    // One candidate centre; false stops the scan of the current intersection
    // (pop: coordinates or distance to the line beyond 1e10).
    fn centre_1(
        &mut self,
        a: &Args,
        center: Point2,
        radius: &mut RadiusCandidates,
    ) -> Result<bool, Failure> {
        let (c1, c2, l3) = (a.q1.qualified(), a.q2.qualified(), a.q3.qualified());
        if center.x() > 1e10 || center.y() > 1e10 {
            return Ok(false);
        }
        let dist1: f64 = center.distance(c1.location());
        let dist2: f64 = center.distance(c2.location());
        let dist3: f64 = l3.distance(center);
        if dist3 > 1e10 {
            return Ok(false);
        }
        let tol: f64 = a.tolerance.abs();
        let nbsol1: i32 = radius.first_circle(a.q1, dist1, a.tolerance, false);
        radius.second_circle(a.q2, dist2, nbsol1, (a.tolerance, tol));
        // Native tests Qualified2 instead of Qualified3 after the first branch.
        let side: f64 = side_of(l3, center);
        let nbsol3: i32 = if a.q3.is_enclosed() && radius.ok {
            i32::from(side <= 0.0)
        } else if a.q2.is_outside() && radius.ok {
            i32::from(side >= 0.0)
        } else {
            i32::from(a.q2.is_unqualified() && radius.ok)
        };
        if radius.ok {
            for ind3 in 1..=nbsol3 {
                self.solution_1(a, center, radius.at(ind3))?;
            }
        }
        Ok(true)
    }

    fn solution_1(&mut self, a: &Args, center: Point2, r: f64) -> Result<(), Failure> {
        let (c1, c2, l3) = (a.q1.qualified(), a.q2.qualified(), a.q3.qualified());
        let tol: f64 = a.tolerance.abs();
        let index: i32 = self.push_circle(center, r)?;
        // Native measures both distances from the centre of circle 1.
        let distcc: f64 = center.distance(c1.location());
        self.set_qualifier(
            1,
            index,
            qualify(a.q1.qualifier(), distcc, r, c1.radius(), tol, false),
        )?;
        self.set_qualifier(
            2,
            index,
            qualify(a.q2.qualifier(), distcc, r, c2.radius(), tol, false),
        )?;
        self.set_qualifier(3, index, line_qualifier(a.q3, center, normal_of(l3)?)?)?;
        self.protected_tangency_1(index, center, r, c1, a.tolerance)?;
        self.protected_tangency_2(index, center, r, c2, a.tolerance)?;
        self.touch_line(3, index, l3)
    }

    // The tangency on circle 1, guarded (POP) against a point at the solution or
    // argument centre, whose parameter is then 1.
    fn protected_tangency_1(
        &mut self,
        index: i32,
        center: Point2,
        r: f64,
        c1: Circle2,
        tolerance: f64,
    ) -> Result<(), Failure> {
        if center.distance(c1.location()) <= tolerance && (r - c1.radius()).abs() <= tolerance {
            return self.mark_same(1, index);
        }
        let point: Point2 = towards(center, c1, r)?;
        let solution: Circle2 = self.cirsol.get(index)?;
        self.tangency1.same.set(index, 0)?;
        self.tangency1.point.set(index, point)?;
        let par_sol: f64 = guarded_parameter(solution, point)?;
        self.tangency1.par_sol.set(index, par_sol)?;
        self.tangency1
            .par_arg
            .set(index, guarded_parameter(c1, point)?)
    }

    // The tangency on circle 2. Its first guard tests the tangency point on
    // circle 1 and then writes par1sol, leaving par2sol unset; kept as natively.
    fn protected_tangency_2(
        &mut self,
        index: i32,
        center: Point2,
        r: f64,
        c2: Circle2,
        tolerance: f64,
    ) -> Result<(), Failure> {
        if center.distance(c2.location()) <= tolerance && (r - c2.radius()).abs() <= tolerance {
            return self.mark_same(2, index);
        }
        let point: Point2 = towards(center, c2, r)?;
        let solution: Circle2 = self.cirsol.get(index)?;
        self.tangency2.same.set(index, 0)?;
        self.tangency2.point.set(index, point)?;
        let tangency1: Point2 = self.tangency1.point.get(index)?;
        if solution
            .location()
            .is_equal(tangency1, precision::confusion())
        {
            self.tangency1.par_sol.set(index, 1.0)?;
        } else {
            self.tangency2
                .par_sol
                .set(index, elclib::parameter(&solution, point)?)?;
        }
        self.tangency2
            .par_arg
            .set(index, guarded_parameter(c2, point)?)
    }
}

// Center + r * Dir(circle centre - Center).
fn towards(center: Point2, circle: Circle2, r: f64) -> Result<Point2, Failure> {
    let dc = Direction2::from_xy(circle.location().xy() - center.xy())?;
    Ok(Point2::from(center.xy() + r * dc.xy()))
}

// ElCLib::Parameter, or 1 when the point is the circle centre (POP protection).
fn guarded_parameter(circle: Circle2, point: Point2) -> Result<f64, Failure> {
    if circle.location().is_equal(point, precision::confusion()) {
        return Ok(1.0);
    }
    elclib::parameter(&circle, point)
}
