// Rust translation of GccAna_Circ2dTanOnRad.hxx and the consultation members of
// GccAna_Circ2dTanOnRad.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2dTanOnRad: circles of given radius tangent to a circle, a line or a
//! point, with their centre on a line or a circle. Each native constructor file
//! is one submodule; `circ_on_line` holds the one of the main file.
mod circ_on_circ;
mod circ_on_line;
mod lin_on_circ;
mod lin_on_line;
mod pnt_on_circ;
mod pnt_on_line;

use super::solutions::{Slots, Tangencies, check_index, check_solution, negative_value};
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Point2};

/// GccAna_Circ2dTanOnRad: the circles of radius `Radius` tangent to a qualified
/// circle, a qualified line or a point, centred on a line or a circle. The
/// constructors name the tangency argument then the centre locus
/// (`circ_on_line`...).
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2dTanOnRad;
/// use rustycad_tkmath::gp::{Line2, Point2};
/// let circ = Circ2dTanOnRad::pnt_on_line(Point2::new(0.0, 1.0), Line2::default(), 1.0, 1e-7)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2dTanOnRad {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
    pntcen3: Slots<Point2>,
    parcen3: Slots<f64>,
}

const NAME: &str = "GccAna_Circ2dTanOnRad";

impl Circ2dTanOnRad {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier1: Slots::new(size),
            tangency1: Tangencies::new(size),
            pntcen3: Slots::new(size),
            parcen3: Slots::new(size),
        }
    }

    // Standard_NegativeValue for a negative radius, as every constructor raises.
    fn check_radius(radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(negative_value(NAME, radius));
        }
        Ok(())
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
        check_index(self.nbr_sol, index, "GccAna_Circ2dTanOnRad::ThisSolution")?;
        self.cirsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1); `circ.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "GccAna_Circ2dTanOnRad::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol). Native does not test TheSame1
    /// here: for a solution equal to the argument the values are unset (0 here,
    /// uninitialized natively); `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2dTanOnRad::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }

    /// CenterOn3(Index, ParArg, PntSol): the parameter of the centre on the locus
    /// and, as natively, the tangency point (pnttg1sol), not the centre;
    /// `let (par_arg, point) = circ.center_on3(1)?;`.
    pub fn center_on3(&self, index: i32) -> Result<(f64, Point2), Failure> {
        let member = "GccAna_Circ2dTanOnRad::CenterOn3";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.parcen3.get(index)?, self.tangency1.point.get(index)?))
    }

    /// IsTheSame1(Index): whether solution `index` is the argument circle;
    /// `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2dTanOnRad::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }

    // pntcen3(index) = the centre of solution `index` and its parameter on the locus.
    fn centre_on(&mut self, index: i32, center: Point2, parameter: f64) -> Result<(), Failure> {
        self.pntcen3.set(index, center)?;
        self.parcen3.set(index, parameter)
    }
}
