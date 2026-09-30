// Rust translation of GccAna_Circ2d2TanOn_9.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles tangent to two lines with their centre on a circle: the intersections
//! of the locus with the bisectors of the lines, filtered by the side of the
//! bisector apex each qualifier pair asks for.
//!
//! Native quirks kept: `D1.Angle(D2) <= gp::Resolution()` also holds for every
//! negative angle, which then takes the parallel-lines branch (whose second
//! qualifier test overwrites the first); the general branch builds each circle at
//! intersection point `i` (the first bisector index) instead of `l`, and reads
//! bisectors past the ones that exist, which raise Standard_OutOfRange.
use super::Circ2d2TanOn;
use super::circ_lin_on_line::check_line;
use crate::gcc_ana::Lin2dBisec;
use crate::gcc_ana::solutions::{circle_at, line_qualifier, line_tangency};
use crate::gcc_ent::{Position, QualifiedLin};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle2, Direction2, Line2, Point2, Vector2, Xy};

// The bisector selection of a qualifier pair: nbsol, first bisector i, count j,
// and the signs sgn, s of the apex tests.
struct Selection {
    nbsol: i32,
    i: i32,
    j: i32,
    sgn: f64,
    s: f64,
}

// The native table of the non-parallel qualifier pairs.
fn select(
    q1: QualifiedLin,
    q2: QualifiedLin,
    angle: f64,
    bis: &Lin2dBisec,
) -> Result<Selection, Failure> {
    use Position::{Enclosed as E, Outside as O, Unqualified as U};
    let mut sel = Selection {
        nbsol: 0,
        i: 0,
        j: 0,
        sgn: 1.0,
        s: 1.0,
    };
    let (k1, k2) = (q1.qualifier(), q2.qualifier());
    if (k1, k2) == (U, U) {
        (sel.nbsol, sel.i, sel.j) = (4, 1, 2);
        return Ok(sel);
    }
    if !bis.is_done() {
        return Ok(sel);
    }
    let count: i32 = bis.nb_solutions()?;
    match (k1, k2) {
        (E, E) if count == 2 => (sel.nbsol, sel.i, sel.j, sel.sgn) = (1, 2, 1, -1.0),
        (E, O) if count >= 1 => {
            (sel.nbsol, sel.i, sel.j) = (1, 1, 1);
            if angle >= 0.0 {
                sel.sgn = -1.0;
            }
        }
        (O, E) if count >= 1 => {
            (sel.nbsol, sel.i, sel.j) = (1, 1, 1);
            if angle <= 0.0 {
                sel.sgn = -1.0;
            }
        }
        (O, O) if count >= 1 => (sel.nbsol, sel.i, sel.j) = (1, 2, 1),
        (U, E) | (U, O) | (E, U) | (O, U) => {
            sel.nbsol = 2;
            if count >= 1 {
                (sel.i, sel.j) = (1, 2);
            }
            (sel.s, sel.sgn) = unqualified_signs(k1, k2, angle);
        }
        _ => {}
    }
    Ok(sel)
}

// (s, sgn) of the pairs with one unqualified line.
fn unqualified_signs(k1: Position, k2: Position, angle: f64) -> (f64, f64) {
    use Position::{Enclosed as E, Outside as O, Unqualified as U};
    match (k1, k2) {
        (U, E) if angle >= 0.0 => (-1.0, 1.0),
        (U, E) => (1.0, -1.0),
        (U, O) if angle >= 0.0 => (-1.0, -1.0),
        (E, U) if angle >= 0.0 => (1.0, -1.0),
        (E, U) => (-1.0, 1.0),
        (O, U) if angle <= 0.0 => (-1.0, -1.0),
        _ => (1.0, 1.0),
    }
}

impl Circ2d2TanOn {
    /// GccAna_Circ2d2TanOn(Qualified1, Qualified2, OnCirc, Tolerance): circles
    /// tangent to two qualified lines, centred on `on_circ` (the tolerance is not
    /// used natively); `Circ2d2TanOn::lin_lin_on_circ(l1, l2, circle, tol)?`.
    pub fn lin_lin_on_circ(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        on_circ: Circle2,
        _tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_line(qualified1)?;
        check_line(qualified2)?;
        let (l1, l2) = (qualified1.qualified(), qualified2.qualified());
        let bis = Lin2dBisec::new(l1, l2)?;
        let (d1, d2) = (l1.direction(), l2.direction());
        if d1.angle(d2) <= gp::resolution() {
            circ.parallel_lines((qualified1, qualified2), &bis, on_circ)?;
        } else {
            let sel: Selection = select(qualified1, qualified2, d1.angle(d2), &bis)?;
            if sel.nbsol >= 1 && bis.is_done() {
                circ.apex_filtered((qualified1, qualified2), &bis, on_circ, &sel)?;
            }
        }
        for i in 1..=circ.nbr_sol {
            let solution: Circle2 = circ.cirsol.get(i)?;
            let (pbid, radius) = (solution.location(), solution.radius());
            let p1: Point2 = line_tangency(l1, pbid, radius)?;
            let p2: Point2 = line_tangency(l2, pbid, radius)?;
            circ.tangency1.record(
                i,
                p1,
                elclib::parameter(&solution, p1)?,
                elclib::parameter(&l1, p1)?,
            )?;
            circ.tangency2.record(
                i,
                p2,
                elclib::parameter(&solution, p2)?,
                elclib::parameter(&l2, p2)?,
            )?;
            circ.centre_on(i, pbid, elclib::parameter(&on_circ, pbid)?)?;
        }
        Ok(circ)
    }

    fn parallel_lines(
        &mut self,
        (q1, q2): (QualifiedLin, QualifiedLin),
        bis: &Lin2dBisec,
        on_circ: Circle2,
    ) -> Result<(), Failure> {
        let (l1, l2) = (q1.qualified(), q2.qualified());
        let (d1, d2) = (l1.direction(), l2.direction());
        let dloc: Xy = l1.location().xy() - l2.location().xy();
        let (xynor1, xynor2) = (Xy::new(-d1.y(), d1.x()), Xy::new(-d2.y(), d2.x()));
        // The first qualifier test is overwritten by the second natively.
        let _first: bool = if q1.is_enclosed() {
            dloc.dot(xynor1) <= 0.0
        } else if q1.is_outside() {
            dloc.dot(xynor1) >= 0.0
        } else {
            true
        };
        let ok: bool = if q2.is_enclosed() {
            dloc.dot(xynor2) >= 0.0
        } else if q2.is_outside() {
            dloc.dot(xynor2) <= 0.0
        } else {
            true
        };
        if !ok {
            return Ok(());
        }
        let intp = AnaIntersection::line_circle(bis.this_solution(1)?, on_circ)?;
        if !intp.is_done() {
            return Ok(());
        }
        self.well_done = true;
        if intp.is_empty()? {
            return Ok(());
        }
        for l in 1..=intp.nb_points()? {
            let pt: Point2 = intp.point(l)?.value();
            self.bisector_circle((q1, q2), pt, l1.distance(pt))?;
        }
        Ok(())
    }

    fn apex_filtered(
        &mut self,
        (q1, q2): (QualifiedLin, QualifiedLin),
        bis: &Lin2dBisec,
        on_circ: Circle2,
        sel: &Selection,
    ) -> Result<(), Failure> {
        let l1: Line2 = q1.qualified();
        let mut kk: i32 = 0;
        for k in sel.i..=sel.i + sel.j - 1 {
            kk += 1;
            let bisector: Line2 = bis.this_solution(k)?;
            let intp = AnaIntersection::line_circle(bisector, on_circ)?;
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for l in 1..=intp.nb_points()? {
                    let v = Vector2::between(intp.point(l)?.value(), bisector.location());
                    let along: f64 = v.dot(Vector2::from(bisector.direction()));
                    let kept: bool = (kk == 1 && sel.sgn * along >= 0.0)
                        || (kk == 2 && sel.sgn * sel.s * along >= 0.0)
                        || sel.nbsol == 4;
                    if kept {
                        let pt: Point2 = intp.point(sel.i)?.value();
                        self.bisector_circle((q1, q2), pt, l1.distance(intp.point(l)?.value()))?;
                    }
                }
            }
            self.well_done = true;
        }
        Ok(())
    }

    // A solution centred at `pt` of radius `radius` and its line qualifiers.
    fn bisector_circle(
        &mut self,
        (q1, q2): (QualifiedLin, QualifiedLin),
        pt: Point2,
        radius: f64,
    ) -> Result<(), Failure> {
        let (l1, l2) = (q1.qualified(), q2.qualified());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        self.cirsol.set(index, circle_at(pt, radius)?)?;
        let dnor1 = Direction2::new(-l1.direction().y(), l1.direction().x())?;
        let dnor2 = Direction2::new(-l2.direction().y(), l2.direction().x())?;
        let position1: Position = line_qualifier(q1, pt, dnor1)?;
        let position2: Position = line_qualifier(q2, pt, dnor2)?;
        self.set_qualifiers(index, position1, position2)
    }
}
