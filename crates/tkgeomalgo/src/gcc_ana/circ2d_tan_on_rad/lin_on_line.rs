// Rust translation of GccAna_Circ2dTanOnRad_1.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Circles of given radius tangent to a line with their centre on another line:
//! the intersections of the locus with the parallels to the line at the radius.
use super::{Circ2dTanOnRad, NAME};
use crate::gcc_ana::solutions::circle_at;
use crate::gcc_ent::{Position, QualifiedLin, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Vector2, Xy};

impl Circ2dTanOnRad {
    /// GccAna_Circ2dTanOnRad(Qualified1, OnLine, Radius, Tolerance): circles of
    /// radius `radius` tangent to a qualified line, centred on `on_line` (none when
    /// the lines are parallel); `Circ2dTanOnRad::lin_on_line(l1, line, radius, tol)?`.
    pub fn lin_on_line(
        qualified1: QualifiedLin,
        on_line: Line2,
        radius: f64,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let tol: f64 = tolerance.abs();
        let mut circ = Self::empty(2);
        if !(qualified1.is_enclosed() || qualified1.is_outside() || qualified1.is_unqualified()) {
            return Err(bad_qualifier(&format!(
                "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
                qualified1.qualifier()
            )));
        }
        let l1: Line2 = qualified1.qualified();
        let dir1: Direction2 = l1.direction();
        Self::check_radius(radius)?;
        if on_line.direction().is_parallel(dir1, tol) {
            circ.well_done = true;
            return Ok(circ);
        }
        let eps: &[f64] = if qualified1.is_enclosed() {
            &[-1.0]
        } else if qualified1.is_outside() {
            &[1.0]
        } else {
            &[1.0, -1.0]
        };
        let (dx1, dy1) = (dir1.x(), dir1.y());
        let (lx1, ly1) = (l1.location().x(), l1.location().y());
        for &e in eps {
            let l1para = Line2::new(
                Point2::new(lx1 + e * radius * dy1, ly1 - e * radius * dx1),
                dir1,
            );
            let intp = AnaIntersection::lines(on_line, l1para);
            if !intp.is_done() {
                continue;
            }
            if !intp.is_empty()? {
                for i in 1..=intp.nb_points()? {
                    circ.parallel_solution(qualified1, on_line, radius, intp.point(i)?.value())?;
                }
            }
            circ.well_done = true;
        }
        Ok(circ)
    }

    // A solution centred at `center`. Native stores the centre of the first
    // solution as pntcen3 of every solution; kept.
    fn parallel_solution(
        &mut self,
        qualified1: QualifiedLin,
        on_line: Line2,
        radius: f64,
        center: Point2,
    ) -> Result<(), Failure> {
        let l1: Line2 = qualified1.qualified();
        let dir1: Direction2 = l1.direction();
        let (dx1, dy1) = (dir1.x(), dir1.y());
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        let solution: Circle2 = circle_at(center, radius)?;
        self.cirsol.set(index, solution)?;
        let normal = Direction2::new(-dir1.y(), dir1.x())?;
        let dc1 = Direction2::from_xy(l1.location().xy() - center.xy())?;
        let position = if !qualified1.is_unqualified() {
            qualified1.qualifier()
        } else if dc1.dot(normal) > 0.0 {
            Position::Outside
        } else {
            Position::Enclosed
        };
        self.qualifier1.set(index, position)?;
        let towards_line: f64 =
            Vector2::between(center, l1.location()).dot(Vector2::from(Direction2::new(-dy1, dx1)?));
        let point = if towards_line > 0.0 {
            Point2::from(center.xy() + radius * Xy::new(-dy1, dx1))
        } else {
            Point2::from(center.xy() - radius * Xy::new(-dy1, dx1))
        };
        let pntcen3: Point2 = self.cirsol.get(1)?.location();
        let par_sol: f64 = elclib::parameter(&solution, point)?;
        let par_arg: f64 = elclib::parameter(&l1, point)?;
        self.tangency1.record(index, point, par_sol, par_arg)?;
        self.centre_on(index, pntcen3, elclib::parameter(&on_line, pntcen3)?)
    }
}
