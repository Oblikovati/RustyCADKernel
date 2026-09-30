// Rust translation of GccAna_Circ2d2TanRad.hxx and the consultation members of
// GccAna_Circ2d2TanRad.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2d2TanRad: circles of given radius tangent to two elements
//! (qualified circles and lines, and points). Each native constructor file is one
//! submodule; `circ_circ` and its `circ_circ_cases*` hold the main file.
mod circ_circ;
mod circ_circ_cases;
mod circ_circ_cases2;
mod circ_lin;
mod circ_pnt;
mod lin_lin;
mod lin_pnt;
mod pnt_pnt;

use super::solutions::{Slots, Tangencies, check_solution, circle_at, negative_value};
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Point2};

/// GccAna_Circ2d2TanRad: the circles of radius `Radius` tangent to two elements
/// among qualified circles, qualified lines and points, in the native order.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2d2TanRad;
/// use rustycad_tkmath::gp::Point2;
/// let circ = Circ2d2TanRad::pnt_pnt(Point2::new(-1.0, 0.0), Point2::new(1.0, 0.0), 1.0, 1e-7)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanRad {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    qualifier2: Slots<Position>,
    tangency1: Tangencies,
    tangency2: Tangencies,
}

const NAME: &str = "GccAna_Circ2d2TanRad";

impl Circ2d2TanRad {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier1: Slots::new(size),
            qualifier2: Slots::new(size),
            tangency1: Tangencies::new(size),
            tangency2: Tangencies::new(size),
        }
    }

    fn check_radius(radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(negative_value(NAME, radius));
        }
        Ok(())
    }

    // NbrSol++; cirsol(NbrSol) = the circle of `radius` at `center`.
    fn push_circle(&mut self, center: Point2, radius: f64) -> Result<i32, Failure> {
        self.nbr_sol += 1;
        self.cirsol.set(self.nbr_sol, circle_at(center, radius)?)?;
        Ok(self.nbr_sol)
    }

    fn set_qualifiers(&mut self, index: i32, q1: Position, q2: Position) -> Result<(), Failure> {
        self.qualifier1.set(index, q1)?;
        self.qualifier2.set(index, q2)
    }

    /// IsDone: true when the construction ran to its end; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions (no WellDone test natively); `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index); `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        check_solution(
            self.well_done,
            self.nbr_sol,
            index,
            "GccAna_Circ2d2TanRad::ThisSolution",
        )?;
        self.cirsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1, Qualif2); `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let member = "GccAna_Circ2d2TanRad::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.qualifier1.get(index)?, self.qualifier2.get(index)?))
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol); StdFail_NotDone when the solution
    /// is the first argument; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d2TanRad::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.checked(index, member)
    }

    /// Tangency2(Index, ParSol, ParArg, PntSol); `circ.tangency2(1)?`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d2TanRad::Tangency2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.checked(index, member)
    }

    /// IsTheSame1(Index); `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d2TanRad::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }

    /// IsTheSame2(Index); `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d2TanRad::IsTheSame2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.is_the_same(index)
    }
}
