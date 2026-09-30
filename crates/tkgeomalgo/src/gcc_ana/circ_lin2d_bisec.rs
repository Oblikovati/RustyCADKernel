// Rust translation of GccAna_CircLin2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_CircLin2dBisec: the bisecting curves of a circle and a line.
use super::bisector;
use super::solutions::check_solution;
use crate::gcc_int::{BLine, BParab, Bisec};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis2d, Circle2, Direction2, Line2, Parabola2, Point2, Xy};

/// GccAna_CircLin2dBisec: the two loci equidistant from a circle and a line, both
/// parabolas of focus the circle centre; the first is the normal line through the
/// centre when the line is tangent to the circle (within gp::Resolution). The
/// solutions are computed when they are read, as natively.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::CircLin2dBisec;
/// use rustycad_tkmath::gp::{Axis2d, Circle2, Line2, Point2, Direction2};
/// let axis = Axis2d::new(Point2::new(0.0, 5.0), Direction2::new(1.0, 0.0)?);
/// let bisec = CircLin2dBisec::new(Circle2::from_x_axis(axis, 1.0, true)?, Line2::default());
/// assert_eq!(bisec.nb_solutions(), 2);
/// assert!(bisec.this_solution(2)?.parabola().is_ok());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct CircLin2dBisec {
    well_done: bool,
    nbr_sol: i32,
    circle: Circle2,
    line: Line2,
}

impl CircLin2dBisec {
    /// GccAna_CircLin2dBisec(Circle, Line): always done with two solutions;
    /// `CircLin2dBisec::new(circle, line)`.
    pub fn new(circle: Circle2, line: Line2) -> Self {
        Self {
            well_done: true,
            nbr_sol: 2,
            circle,
            line,
        }
    }

    /// IsDone: true; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions: 2; `bisec.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index): the bisecting curve `index` (1 or 2);
    /// `bisec.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Handle<dyn Bisec>, Failure> {
        let member = "GccAna_CircLin2dBisec::ThisSolution";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        let (xdir, ydir) = (self.line.direction().x(), self.line.direction().y());
        let centre: Point2 = self.circle.location();
        let r1: f64 = self.circle.radius();
        let dist: f64 = self.line.distance(centre);
        if (self.line.distance(centre) - r1).abs() <= gp::resolution() && index == 1 {
            let biscirlin1 = Line2::new(centre, Direction2::new(-ydir, xdir)?);
            return Ok(bisector(BLine::new(biscirlin1)));
        }
        let (xloc, yloc) = (self.line.location().x(), self.line.location().y());
        let side: f64 = -ydir * (centre.x() - xloc) + xdir * (centre.y() - yloc);
        let signe: f64 = if side > 0.0 { 1.0 } else { -1.0 };
        let biscirlin = parabola(centre, (xdir, ydir), signe, (dist, r1), index)?;
        Ok(bisector(BParab::new(biscirlin)))
    }
}

// The bisecting parabola `index` of a circle (centre, radius r1) at distance `dist`
// from a line of direction (xdir, ydir), the centre lying on side `signe`.
fn parabola(
    centre: Point2,
    (xdir, ydir): (f64, f64),
    signe: f64,
    (dist, r1): (f64, f64),
    index: i32,
) -> Result<Parabola2, Failure> {
    if dist == r1 {
        let apex = apex_at(centre, (xdir, ydir), signe, dist + r1);
        let axis = Axis2d::new(apex, Direction2::new(signe * (-ydir), signe * xdir)?);
        return Parabola2::from_mirror_axis(axis, r1, true);
    }
    if index == 1 {
        let apex = apex_at(centre, (xdir, ydir), signe, dist + r1);
        let axis = Axis2d::new(apex, Direction2::new(-signe * ydir, signe * xdir)?);
        return Parabola2::from_mirror_axis(axis, (dist + r1) / 2.0, true);
    }
    let apex = apex_at(centre, (xdir, ydir), signe, dist - r1);
    let direction = if dist < r1 {
        Direction2::new(signe * ydir, -signe * xdir)?
    } else {
        Direction2::new(-signe * ydir, signe * xdir)?
    };
    let axis = Axis2d::new(apex, direction);
    Parabola2::from_mirror_axis(axis, (dist - r1).abs() / 2.0, true)
}

// gp_XY(xcencir + signe * ydir * span / 2, ycencir - signe * xdir * span / 2.).
fn apex_at(centre: Point2, (xdir, ydir): (f64, f64), signe: f64, span: f64) -> Point2 {
    Point2::from(Xy::new(
        centre.x() + signe * ydir * span / 2.0,
        centre.y() - signe * xdir * span / 2.0,
    ))
}
