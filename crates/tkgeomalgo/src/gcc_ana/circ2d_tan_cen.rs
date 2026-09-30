// Rust translation of GccAna_Circ2dTanCen.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2dTanCen: circles of given centre tangent to a circle, a line or a
//! point.
use super::check_qualifier;
use super::solutions::{Slots, Tangencies, check_index, check_solution, circle_at, qualify};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Circle2, Direction2, Line2, Point2, Xy};

/// GccAna_Circ2dTanCen: the circles centred at `Pcenter` and tangent to a
/// qualified circle (up to two), to a line or through a point (one each).
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2dTanCen;
/// use rustycad_tkmath::gp::{Line2, Point2};
/// let circ = Circ2dTanCen::tangent_to_line(Line2::default(), Point2::new(0.0, 2.0))?;
/// assert_eq!(circ.this_solution(1)?.radius(), 2.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2dTanCen {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
}

impl Circ2dTanCen {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier1: Slots::new(size),
            tangency1: Tangencies::new(size),
        }
    }

    /// GccAna_Circ2dTanCen(Qualified1, Pcenter, Tolerance): the circles centred at
    /// `pcenter` tangent to a qualified circle; `Circ2dTanCen::tangent_to_circle(q, c, tol)?`.
    pub fn tangent_to_circle(
        qualified1: QualifiedCirc,
        pcenter: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        check_qualifier(qualified1.qualifier(), "GccAna_Circ2dTanCen")?;
        let tol: f64 = tolerance.abs();
        let c1: Circle2 = qualified1.qualified();
        let dist: f64 = pcenter.distance(c1.location());
        if !qualified1.is_unqualified() {
            circ.qualified_solution(qualified1, pcenter, dist, tol)?;
        } else if dist >= gp::resolution() {
            circ.unqualified_solutions(c1, pcenter, dist, tol)?;
        } else {
            // Concentric: the solution is the circle itself.
            circ.nbr_sol = 1;
            circ.cirsol.set(1, c1)?;
            circ.qualifier1.set(1, qualified1.qualifier())?;
            circ.tangency1.same.set(1, 1)?;
            circ.well_done = true;
        }
        Ok(circ)
    }

    // One solution of an enclosed, enclosing or outside circle, if compatible.
    fn qualified_solution(
        &mut self,
        qualified1: QualifiedCirc,
        pcenter: Point2,
        dist: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let (mut radius, mut signe): (f64, f64) = (0.0, 0.0);
        if qualified1.is_enclosed() {
            if dist - r1 <= tol {
                (radius, signe) = ((r1 - dist).abs(), 1.0);
            } else {
                self.well_done = true;
            }
        } else if qualified1.is_enclosing() {
            (radius, signe) = (r1 + dist, -1.0);
        } else if qualified1.is_outside() {
            if dist < r1 - tol {
                self.well_done = true;
            } else {
                (radius, signe) = ((r1 - dist).abs(), -1.0);
            }
        }
        if signe == 0.0 {
            return Ok(());
        }
        self.nbr_sol += 1;
        let index: i32 = self.nbr_sol;
        self.cirsol.set(index, circle_at(pcenter, radius)?)?;
        self.qualifier1.set(index, qualified1.qualifier())?;
        if dist <= gp::resolution() {
            self.tangency1.same.set(index, 1)?;
        } else {
            self.touch(index, c1, pcenter, signe * radius)?;
        }
        self.well_done = true;
        Ok(())
    }

    // The two circles of an unqualified circle: radii |R1 - dist| and R1 + dist.
    fn unqualified_solutions(
        &mut self,
        c1: Circle2,
        pcenter: Point2,
        dist: f64,
        tol: f64,
    ) -> Result<(), Failure> {
        let r1: f64 = c1.radius();
        let mut signe: f64 = 1.0;
        for _ in 1..=2 {
            signe = -signe;
            let signe1: f64 = if r1 - dist <= 0.0 { -1.0 } else { -signe };
            let radius: f64 = (r1 + signe * dist).abs();
            self.nbr_sol += 1;
            let index: i32 = self.nbr_sol;
            self.cirsol.set(index, circle_at(pcenter, radius)?)?;
            let distcc1: f64 = pcenter.distance(c1.location());
            let position = qualify(Position::Unqualified, distcc1, radius, r1, tol, false);
            self.qualifier1.set(index, position)?;
            self.well_done = true;
            self.touch(index, c1, pcenter, signe1 * radius)?;
        }
        Ok(())
    }

    // The tangency point Pcenter + reach * d (d from the centre of C1 to Pcenter)
    // and its parameters, TheSame = 0.
    fn touch(
        &mut self,
        index: i32,
        c1: Circle2,
        pcenter: Point2,
        reach: f64,
    ) -> Result<(), Failure> {
        let centre: Point2 = c1.location();
        let d = Direction2::new(pcenter.x() - centre.x(), pcenter.y() - centre.y())?;
        let point = Point2::from(pcenter.xy() + reach * d.xy());
        let par_sol: f64 = elclib::parameter(&self.cirsol.get(index)?, point)?;
        let par_arg: f64 = elclib::parameter(&c1, point)?;
        self.tangency1.record(index, point, par_sol, par_arg)
    }

    /// GccAna_Circ2dTanCen(Linetan, Pcenter): the circle centred at `pcenter`
    /// tangent to a line; `Circ2dTanCen::tangent_to_line(line, center)?`.
    pub fn tangent_to_line(linetan: Line2, pcenter: Point2) -> Result<Self, Failure> {
        let mut circ = Self::empty(1);
        let rayon: f64 = linetan.distance(pcenter);
        let cirsol: Circle2 = circle_at(pcenter, rayon)?;
        circ.cirsol.set(1, cirsol)?;
        circ.qualifier1.set(1, Position::NoQualifier)?;
        let (xloc, yloc) = (linetan.location().x(), linetan.location().y());
        let (xdir, ydir) = (linetan.direction().x(), linetan.direction().y());
        let towards = Direction2::new(xloc - pcenter.x(), yloc - pcenter.y())?;
        let side = if towards.dot(Direction2::new(-ydir, xdir)?) > 0.0 {
            Xy::new(-ydir, xdir)
        } else {
            Xy::new(ydir, -xdir)
        };
        let point = Point2::from(pcenter.xy() + rayon * side);
        let par_sol: f64 = elclib::parameter(&cirsol, point)?;
        let par_arg: f64 = elclib::parameter(&linetan, point)?;
        circ.tangency1.record(1, point, par_sol, par_arg)?;
        circ.nbr_sol = 1;
        circ.well_done = true;
        Ok(circ)
    }

    /// GccAna_Circ2dTanCen(Point1, Pcenter): the circle centred at `pcenter`
    /// through `point1`; `Circ2dTanCen::through_point(point, center)?`.
    pub fn through_point(point1: Point2, pcenter: Point2) -> Result<Self, Failure> {
        let mut circ = Self::empty(1);
        let cirsol: Circle2 = circle_at(pcenter, point1.distance(pcenter))?;
        circ.cirsol.set(1, cirsol)?;
        circ.qualifier1.set(1, Position::NoQualifier)?;
        let par_sol: f64 = elclib::parameter(&cirsol, point1)?;
        circ.tangency1.record(1, point1, par_sol, 0.0)?;
        circ.nbr_sol = 1;
        circ.well_done = true;
        Ok(circ)
    }

    /// IsDone: true when the construction ran to its end; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions (no WellDone test natively); `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index): Standard_OutOfRange outside [1, NbSolutions] (no
    /// WellDone test natively); `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        check_index(self.nbr_sol, index, "GccAna_Circ2dTanCen::ThisSolution")?;
        self.cirsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1); `circ.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "GccAna_Circ2dTanCen::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol); StdFail_NotDone when the
    /// solution is the argument itself; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2dTanCen::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.checked(index, member)
    }

    /// IsTheSame1(Index): whether solution `index` is the argument circle;
    /// `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2dTanCen::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }
}
