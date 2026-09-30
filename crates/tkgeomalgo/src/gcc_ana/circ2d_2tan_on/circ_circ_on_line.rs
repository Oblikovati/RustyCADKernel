// Rust translation of the circle-circle-on-line constructor of GccAna_Circ2d2TanOn.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to two circles with their centre on a line: the boundary cases
//! (radii from the distances of the centres to the line, centre on the line of
//! centres), else the intersections of the line with the bisectors of the circles.
//!
//! Native quirks kept: the general case reads the `Radius` array at the bisector
//! index (not the solution index) for its unqualified-circle tests, so a third
//! bisector raises Standard_OutOfRange there; an unqualified first circle writes
//! Rbid(1) twice and leaves Rbid(2) unset (0 here, uninitialized natively); and
//! the argument parameter on C1 is computed at the second tangency point, which is
//! not set yet (the origin).
use super::{Circ2d2TanOn, NAME};
use crate::gcc_ana::solutions::{Slots, circle_at, qualify};
use crate::gcc_ana::{Circ2dBisec, check_qualifier};
use crate::gcc_ent::QualifiedCirc;
use crate::gcc_int::{Bisec, IType};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

// The arguments and the arrays the native constructor shares between its steps.
struct Work {
    q1: QualifiedCirc,
    q2: QualifiedCirc,
    on_line: Line2,
    tol: f64,
    radius: Slots<f64>,
    rbid1: Slots<f64>,
    rbid2: Slots<f64>,
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnLine, Tolerance): circles
    /// tangent to two qualified circles, centred on `on_line`, up to 4;
    /// `Circ2d2TanOn::circ_circ_on_line(q1, q2, line, tol)?`.
    pub fn circ_circ_on_line(
        qualified1: QualifiedCirc,
        qualified2: QualifiedCirc,
        on_line: Line2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_qualifier(qualified1.qualifier(), NAME)?;
        check_qualifier(qualified2.qualifier(), NAME)?;
        let mut w = Work {
            q1: qualified1,
            q2: qualified2,
            on_line,
            tol: tolerance.abs(),
            radius: Slots::new(2),
            rbid1: Slots::new(2),
            rbid2: Slots::new(2),
        };
        let nbsol: i32 = circ.boundary_radii(&mut w)?;
        let (center1, center2) = (
            qualified1.qualified().location(),
            qualified2.qualified().location(),
        );
        let l = Line2::new(center1, Direction2::from_xy(center2.xy() - center1.xy())?);
        let mut intp = AnaIntersection::lines(on_line, l);
        if intp.is_done() && !intp.is_empty()? {
            for j in 1..=intp.nb_points()? {
                let center: Point2 = intp.point(j)?.value();
                for i in 1..=nbsol {
                    let radius: f64 = w.radius.get(i)?;
                    circ.boundary_solution(&w, center, radius)?;
                }
            }
        }
        if !circ.well_done {
            circ.bisector_case(&mut w, &mut intp)?;
        }
        Ok(circ)
    }

    // Processing of boundary cases: equal distances of the circles to the line.
    fn boundary_radii(&mut self, w: &mut Work) -> Result<i32, Failure> {
        let (c1, c2) = (w.q1.qualified(), w.q2.qualified());
        let (dist1, dist2) = (
            w.on_line.distance(c1.location()),
            w.on_line.distance(c2.location()),
        );
        let (d1, d2, d3, d4) = (
            dist1 + c1.radius(),
            dist2 + c2.radius(),
            dist1 - c1.radius(),
            dist2 - c2.radius(),
        );
        let inner = |q: QualifiedCirc| q.is_enclosed() || q.is_outside() || q.is_unqualified();
        let outer = |q: QualifiedCirc| q.is_enclosing() || q.is_unqualified();
        let cases: [(f64, bool, f64); 4] = [
            (d3 - d4, inner(w.q1) && inner(w.q2), d3),
            (d1 - d2, outer(w.q1) && outer(w.q2), d1),
            (d1 - d4, outer(w.q1) && inner(w.q2), d1),
            (d3 - d2, outer(w.q2) && inner(w.q1), d3),
        ];
        let mut nbsol: i32 = 0;
        for (difference, qualified, value) in cases {
            if difference.abs() < w.tol && qualified {
                nbsol += 1;
                w.radius.set(nbsol, value.abs())?;
                self.well_done = true;
            }
        }
        Ok(nbsol)
    }

    fn boundary_solution(&mut self, w: &Work, center: Point2, radius: f64) -> Result<(), Failure> {
        let (c1, c2) = (w.q1.qualified(), w.q2.qualified());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        self.well_done = true;
        let (distcc1, distcc2) = (
            center.distance(c1.location()),
            center.distance(c2.location()),
        );
        let q1 = qualify(w.q1.qualifier(), distcc1, radius, c1.radius(), w.tol, false);
        let q2 = qualify(w.q2.qualifier(), distcc2, radius, c2.radius(), w.tol, false);
        self.set_qualifiers(index, q1, q2)?;
        let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
        let dc2 = Direction2::from_xy(c2.location().xy() - center.xy())?;
        let p1 = Point2::from(center.xy() + radius * dc1.xy());
        let p2 = Point2::from(center.xy() + radius * dc2.xy());
        self.tangency1.record(
            index,
            p1,
            elclib::parameter(&solution, p1)?,
            elclib::parameter(&c1, p1)?,
        )?;
        self.tangency2.record(
            index,
            p2,
            elclib::parameter(&solution, p2)?,
            elclib::parameter(&c2, p2)?,
        )?;
        let centre: Point2 = solution.location();
        self.centre_on(index, centre, elclib::parameter(&w.on_line, centre)?)
    }

    // General case: the centres are where the line meets the bisectors of C1, C2.
    fn bisector_case(&mut self, w: &mut Work, intp: &mut AnaIntersection) -> Result<(), Failure> {
        let bis = Circ2dBisec::new(w.q1.qualified(), w.q2.qualified());
        if !bis.is_done() {
            return Ok(());
        }
        for i in 1..=bis.nb_solutions()? {
            let sol: Handle<dyn Bisec> = bis.this_solution(i)?;
            match sol.arc_type() {
                IType::Cir => *intp = AnaIntersection::line_circle(w.on_line, sol.circle()?)?,
                IType::Lin => *intp = AnaIntersection::lines(w.on_line, sol.line()?),
                IType::Hpr => {
                    *intp = AnaIntersection::line_conic(
                        w.on_line,
                        &Conic::from_hyperbola(sol.hyperbola()?),
                    )?
                }
                IType::Ell => {
                    *intp = AnaIntersection::line_conic(
                        w.on_line,
                        &Conic::from_ellipse(sol.ellipse()?),
                    )?
                }
                _ => {}
            }
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for j in 1..=intp.nb_points()? {
                    self.bisector_centre(w, i, intp.point(j)?.value())?;
                }
            }
            self.well_done = true;
        }
        Ok(())
    }

    // The radii each circle allows at `center` (Rbid for C1, RBid for C2); the
    // agreeing pairs give the solutions.
    fn bisector_centre(&mut self, w: &mut Work, i: i32, center: Point2) -> Result<(), Failure> {
        let (c1, c2) = (w.q1.qualified(), w.q2.qualified());
        let (dist1, dist2) = (
            center.distance(c1.location()),
            center.distance(c2.location()),
        );
        let nnsol: i32 = agreeing_radii(
            (w.q1, w.q2),
            (dist1, dist2),
            w.tol,
            (&mut w.rbid1, &mut w.rbid2),
            &mut w.radius,
        )?;
        for k in 1..=nnsol {
            self.bisector_solution(w, i, k, center)?;
        }
        Ok(())
    }

    fn bisector_solution(
        &mut self,
        w: &Work,
        i: i32,
        k: i32,
        center: Point2,
    ) -> Result<(), Failure> {
        let (c1, c2) = (w.q1.qualified(), w.q2.qualified());
        let radius: f64 = w.radius.get(k)?;
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let (distcc1, distcc2) = (
            center.distance(c1.location()),
            center.distance(c2.location()),
        );
        // Native tests the unqualified positions with Radius(i), i the bisector index.
        let q1 = if w.q1.is_unqualified() {
            qualify(
                w.q1.qualifier(),
                distcc1,
                w.radius.get(i)?,
                c1.radius(),
                w.tol,
                false,
            )
        } else {
            w.q1.qualifier()
        };
        let q2 = if w.q2.is_unqualified() {
            qualify(
                w.q2.qualifier(),
                distcc2,
                w.radius.get(i)?,
                c2.radius(),
                w.tol,
                false,
            )
        } else {
            w.q2.qualifier()
        };
        self.set_qualifiers(index, q1, q2)?;
        if center.distance(c1.location()) <= w.tol && (radius - c1.radius()).abs() <= w.tol {
            self.tangency1.same.set(index, 1)?;
        } else {
            let dc1 = Direction2::from_xy(c1.location().xy() - center.xy())?;
            let p1 = Point2::from(center.xy() + radius * dc1.xy());
            let par1: f64 = elclib::parameter(&solution, p1)?;
            let pararg1: f64 = elclib::parameter(&c1, self.tangency2.point.get(index)?)?;
            self.tangency1.record(index, p1, par1, pararg1)?;
        }
        if center.distance(c2.location()) <= w.tol && (radius - c2.radius()).abs() <= w.tol {
            self.tangency2.same.set(index, 1)?;
        } else {
            let dc2 = Direction2::from_xy(c2.location().xy() - center.xy())?;
            let p2 = Point2::from(center.xy() + radius * dc2.xy());
            self.tangency2.record(
                index,
                p2,
                elclib::parameter(&solution, p2)?,
                elclib::parameter(&c2, p2)?,
            )?;
        }
        self.centre_on(index, center, elclib::parameter(&w.on_line, center)?)
    }
}

// The radii both circles allow for a centre at (dist1, dist2), the agreeing
// pairs averaged into `radius`; returns their count (nnsol).
pub(super) fn agreeing_radii(
    (q1, q2): (QualifiedCirc, QualifiedCirc),
    (dist1, dist2): (f64, f64),
    tol: f64,
    (rbid1, rbid2): (&mut Slots<f64>, &mut Slots<f64>),
    radius: &mut Slots<f64>,
) -> Result<i32, Failure> {
    let nbsol: i32 = allowed_radii(q1, dist1, tol, rbid1, false)?;
    let nsol: i32 = if nbsol != 0 {
        allowed_radii(q2, dist2, tol, rbid2, true)?
    } else {
        0
    };
    let mut nnsol: i32 = 0;
    for isol in 1..=nbsol {
        for jsol in 1..=nsol {
            let (r_1, r_2) = (rbid1.get(isol)?, rbid2.get(jsol)?);
            if (r_1 - r_2).abs() <= tol {
                nnsol += 1;
                radius.set(nnsol, (r_2 + r_1) / 2.0)?;
            }
        }
    }
    Ok(nnsol)
}

// The radii a qualified circle allows for a centre at `dist`, into `rbid`; the
// count (nbsol / nsol). An unqualified first circle overwrites Rbid(1) (native).
fn allowed_radii(
    q: QualifiedCirc,
    dist: f64,
    tol: f64,
    rbid: &mut Slots<f64>,
    second: bool,
) -> Result<i32, Failure> {
    let r: f64 = q.qualified().radius();
    if q.is_enclosed() {
        if dist - r < tol {
            rbid.set(1, (r - dist).abs())?;
            return Ok(1);
        }
        return Ok(0);
    }
    if q.is_outside() {
        if r - dist < tol {
            let value: f64 = if second {
                (r - dist).abs()
            } else {
                (dist - r).abs()
            };
            rbid.set(1, value)?;
            return Ok(1);
        }
        return Ok(0);
    }
    if q.is_enclosing() {
        rbid.set(1, dist + r)?;
        return Ok(1);
    }
    if second {
        rbid.set(1, dist + r)?;
        rbid.set(2, (r - dist).abs())?;
    } else {
        rbid.set(1, dist + r)?;
        rbid.set(1, (dist - r).abs())?;
    }
    Ok(2)
}
