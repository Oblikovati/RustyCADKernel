// Rust translation of GccAna_Circ2d2TanOn.hxx and the consultation members of
// GccAna_Circ2d2TanOn.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2d2TanOn: circles tangent to two elements (qualified circles and
//! lines, and points) with their centre on a line or a circle. Each native
//! constructor file is one submodule, named after its arguments and locus.
mod circ_circ_on_circ;
mod circ_circ_on_line;
mod circ_lin_on_circ;
mod circ_lin_on_line;
mod circ_pnt_on_circ;
mod circ_pnt_on_line;
mod lin_lin_on_circ;
mod lin_lin_on_line;
mod lin_pnt_on_circ;
mod lin_pnt_on_line;
mod pnt_pnt_on_circ;
mod pnt_pnt_on_line;

use super::solutions::{Slots, Tangencies, check_solution};
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Point2};

/// GccAna_Circ2d2TanOn: the circles tangent to two elements among qualified
/// circles, qualified lines and points, whose centre lies on a line or a circle,
/// in the native order. The constructors name the two arguments and the locus
/// (`circ_circ_on_line`...).
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2d2TanOn;
/// use rustycad_tkmath::gp::{Line2, Point2};
/// let circ = Circ2d2TanOn::pnt_pnt_on_line(
///     Point2::new(-1.0, 1.0), Point2::new(1.0, 1.0), Line2::default(), 1e-7)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanOn {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    qualifier2: Slots<Position>,
    tangency1: Tangencies,
    tangency2: Tangencies,
    pntcen: Slots<Point2>,
    parcen3: Slots<f64>,
}

const NAME: &str = "GccAna_Circ2d2TanOn";

impl Circ2d2TanOn {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier1: Slots::new(size),
            qualifier2: Slots::new(size),
            tangency1: Tangencies::new(size),
            tangency2: Tangencies::new(size),
            pntcen: Slots::new(size),
            parcen3: Slots::new(size),
        }
    }

    fn set_qualifiers(&mut self, index: i32, q1: Position, q2: Position) -> Result<(), Failure> {
        self.qualifier1.set(index, q1)?;
        self.qualifier2.set(index, q2)
    }

    // The tangency arrays of argument 1 or 2.
    fn tangency_of(&mut self, which: usize) -> &mut Tangencies {
        if which == 1 {
            return &mut self.tangency1;
        }
        &mut self.tangency2
    }

    // pntcen(index) = centre and its parameter on the locus.
    fn centre_on(&mut self, index: i32, centre: Point2, parameter: f64) -> Result<(), Failure> {
        self.pntcen.set(index, centre)?;
        self.parcen3.set(index, parameter)
    }

    /// IsDone: true when the construction ran to its end; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions (no WellDone test natively); `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index): native reads the solution array without testing
    /// WellDone or NbSolutions, so an index past the solutions but within the
    /// array gives the default circle; only the array bounds raise
    /// Standard_OutOfRange; `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        self.cirsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1, Qualif2); `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let member = "GccAna_Circ2d2TanOn::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.qualifier1.get(index)?, self.qualifier2.get(index)?))
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol); StdFail_NotDone when the solution
    /// is the first argument; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d2TanOn::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.checked(index, member)
    }

    /// Tangency2(Index, ParSol, ParArg, PntSol); `circ.tangency2(1)?`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d2TanOn::Tangency2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.checked(index, member)
    }

    /// CenterOn3(Index, ParArg, PntArg): the parameter of the centre on the locus
    /// and, as natively, the first tangency point (pnttg1sol), not the centre;
    /// `let (par_arg, point) = circ.center_on3(1)?;`.
    pub fn center_on3(&self, index: i32) -> Result<(f64, Point2), Failure> {
        let member = "GccAna_Circ2d2TanOn::CenterOn3";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.parcen3.get(index)?, self.tangency1.point.get(index)?))
    }

    /// IsTheSame1(Index); `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d2TanOn::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }

    /// IsTheSame2(Index); `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d2TanOn::IsTheSame2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.is_the_same(index)
    }
}
