// Rust translation of GccAna_Circ2d2TanRad_4.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to two lines: the intersections of their
//! parallels at the radius, on the sides the qualifiers ask for.
//!
//! Native quirks kept: the tolerance is not used (parallel lines are detected with
//! Precision::Angular), and the "normals" of the unqualified side tests are the
//! line directions themselves.
use super::{Circ2d2TanRad, NAME};
use crate::gcc_ent::{Position, QualifiedLin, bad_qualifier};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

fn check_line(qualified: QualifiedLin) -> Result<(), Failure> {
    if qualified.is_enclosed() || qualified.is_outside() || qualified.is_unqualified() {
        return Ok(());
    }
    Err(bad_qualifier(&format!(
        "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
        qualified.qualifier()
    )))
}

// The sides (cote) of a qualified line: +1 enclosed, -1 outside, both unqualified.
fn sides(qualified: QualifiedLin) -> &'static [f64] {
    if qualified.is_enclosed() {
        &[1.0]
    } else if qualified.is_outside() {
        &[-1.0]
    } else {
        &[1.0, -1.0]
    }
}

// The parallel to `line` at the signed distance cote * radius.
fn parallel(line: Line2, cote: f64, radius: f64) -> Line2 {
    let (xdir, ydir) = (line.direction().x(), line.direction().y());
    let (lxloc, lyloc) = (line.location().x(), line.location().y());
    Line2::new(
        Point2::new(lxloc - cote * ydir * radius, lyloc + cote * xdir * radius),
        line.direction(),
    )
}

// The qualifier of an unqualified line from the side of its origin relative to
// the centre, tested against the line direction as natively.
fn line_side(qualified: QualifiedLin, center: Point2) -> Result<Position, Failure> {
    let line: Line2 = qualified.qualified();
    let dc = Direction2::from_xy(line.location().xy() - center.xy())?;
    if !qualified.is_unqualified() {
        return Ok(qualified.qualifier());
    }
    // normL = gp_Dir2d(xdir, ydir): the direction renormalized.
    let norm = Direction2::new(line.direction().x(), line.direction().y())?;
    if dc.dot(norm) > 0.0 {
        return Ok(Position::Outside);
    }
    Ok(Position::Enclosed)
}

impl Circ2d2TanRad {
    /// GccAna_Circ2d2TanRad(Qualified1, Qualified2, Radius, Tolerance): circles of
    /// radius `radius` tangent to two qualified lines (none for parallel lines);
    /// `Circ2d2TanRad::lin_lin(l1, l2, radius, tol)?`.
    pub fn lin_lin(
        qualified1: QualifiedLin,
        qualified2: QualifiedLin,
        radius: f64,
        _tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(4);
        check_line(qualified1)?;
        check_line(qualified2)?;
        let (l1, l2) = (qualified1.qualified(), qualified2.qualified());
        Self::check_radius(radius)?;
        if l1
            .direction()
            .is_parallel(l2.direction(), precision::angular())
        {
            circ.well_done = true;
        } else {
            for &cote1 in sides(qualified1) {
                for &cote2 in sides(qualified2) {
                    let intp = AnaIntersection::lines(
                        parallel(l1, cote1, radius),
                        parallel(l2, cote2, radius),
                    );
                    if !intp.is_done() {
                        continue;
                    }
                    if !intp.is_empty()? {
                        for i in 1..=intp.nb_points()? {
                            circ.corner_solution(
                                (qualified1, qualified2),
                                radius,
                                (cote1, cote2),
                                intp.point(i)?.value(),
                            )?;
                        }
                    }
                    circ.well_done = true;
                }
            }
        }
        for i in 1..=circ.nbr_sol {
            let solution: Circle2 = circ.cirsol.get(i)?;
            let (p1, p2) = (circ.tangency1.point.get(i)?, circ.tangency2.point.get(i)?);
            circ.tangency1
                .par_sol
                .set(i, elclib::parameter(&solution, p1)?)?;
            circ.tangency1.par_arg.set(i, elclib::parameter(&l1, p1)?)?;
            circ.tangency2
                .par_sol
                .set(i, elclib::parameter(&solution, p2)?)?;
            circ.tangency2.par_arg.set(i, elclib::parameter(&l2, p2)?)?;
        }
        Ok(circ)
    }

    fn corner_solution(
        &mut self,
        (q1, q2): (QualifiedLin, QualifiedLin),
        radius: f64,
        (cote1, cote2): (f64, f64),
        center: Point2,
    ) -> Result<(), Failure> {
        let (l1, l2) = (q1.qualified(), q2.qualified());
        let index: i32 = self.push_circle(center, radius)?;
        let position1: Position = line_side(q1, center)?;
        let position2: Position = line_side(q2, center)?;
        self.set_qualifiers(index, position1, position2)?;
        let (x1, y1) = (l1.direction().x(), l1.direction().y());
        let (x2, y2) = (l2.direction().x(), l2.direction().y());
        self.tangency1.point.set(
            index,
            Point2::from(center.xy() + cote1 * radius * Xy::new(y1, -x1)),
        )?;
        self.tangency2.point.set(
            index,
            Point2::from(center.xy() + cote2 * radius * Xy::new(y2, -x2)),
        )
    }
}
