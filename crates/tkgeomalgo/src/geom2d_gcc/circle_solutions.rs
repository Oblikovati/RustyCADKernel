// Rust translation of the solution arrays and consultation checks shared by the
// Geom2dGcc circle solvers at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solution storage of the Geom2dGcc circle solvers: the `cirsol`,
//! `qualifier<i>`, `TheSame<i>`, `pnttg<i>sol`, `par<i>sol` and `pararg<i>` arrays
//! of one native class, and the checks its consultation members share.
use crate::gcc_ana::solutions::{Slots, Tangencies, check_solution};
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Circle2, Point2};

/// The arrays of a solver with `args` tangency arguments (argument 1 at 0).
#[derive(Clone, Debug)]
pub(crate) struct CircleSolutions {
    pub(crate) well_done: bool,
    pub(crate) nbr_sol: i32,
    pub(crate) cirsol: Slots<Circle2>,
    pub(crate) qualifier: Vec<Slots<Position>>,
    pub(crate) tangency: Vec<Tangencies>,
}

impl CircleSolutions {
    /// `NCollection_Array1(1, size)` fields for `args` arguments, not done.
    pub(crate) fn new(size: usize, args: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier: vec![Slots::new(size); args],
            tangency: vec![Tangencies::new(size); args],
        }
    }

    /// StdFail_NotDone, then Standard_OutOfRange outside `1..=NbSolutions()`.
    pub(crate) fn check(&self, index: i32, member: &str) -> Result<(), Failure> {
        check_solution(self.well_done, self.nbr_sol, index, member)
    }

    /// `ThisSolution(Index)` after the done and index checks.
    pub(crate) fn solution(&self, index: i32, member: &str) -> Result<Circle2, Failure> {
        self.check(index, member)?;
        self.cirsol.get(index)
    }

    /// The qualifier of argument `arg` for solution `index` (no check).
    pub(crate) fn qualifier_of(&self, arg: usize, index: i32) -> Result<Position, Failure> {
        self.qualifier[arg].get(index)
    }

    /// `Tangency<arg>(Index, ...)` after the checks; StdFail_NotDone when the
    /// solution is the argument itself.
    pub(crate) fn tangency_of(
        &self,
        arg: usize,
        index: i32,
        member: &str,
    ) -> Result<(f64, f64, Point2), Failure> {
        self.check(index, member)?;
        self.tangency[arg].checked(index, member)
    }

    /// `IsTheSame<arg>(Index)` after the checks.
    pub(crate) fn same_of(&self, arg: usize, index: i32, member: &str) -> Result<bool, Failure> {
        self.check(index, member)?;
        self.tangency[arg].is_the_same(index)
    }

    /// Stores one tangency (point, ParSol, ParArg) with its TheSame flag.
    pub(crate) fn store_tangency(
        &mut self,
        arg: usize,
        index: i32,
        same: bool,
        (par_sol, par_arg, point): (f64, f64, Point2),
    ) -> Result<(), Failure> {
        self.tangency[arg].record(index, point, par_sol, par_arg)?;
        self.tangency[arg].same.set(index, i32::from(same))
    }
}

/// The consultation members of a two-argument circle solver (GccAna or Geom2dGcc)
/// that the `Results` members of Geom2dGcc copy.
pub(crate) trait TwoTangencySolver {
    /// `ThisSolution(Index)`.
    fn solution(&self, index: i32) -> Result<Circle2, Failure>;
    /// `WhichQualifier(Index, Qualif1, Qualif2)`.
    fn qualifiers(&self, index: i32) -> Result<(Position, Position), Failure>;
    /// `IsTheSame1(Index)` (arg 0) or `IsTheSame2(Index)` (arg 1).
    fn same(&self, arg: usize, index: i32) -> Result<bool, Failure>;
    /// `Tangency1(Index, ...)` (arg 0) or `Tangency2(Index, ...)` (arg 1).
    fn tangency(&self, arg: usize, index: i32) -> Result<(f64, f64, Point2), Failure>;
}

// One TwoTangencySolver implementation per solver type, forwarding to its members.
macro_rules! two_tangency_solver {
    ($solver:ty) => {
        impl crate::geom2d_gcc::circle_solutions::TwoTangencySolver for $solver {
            fn solution(&self, index: i32) -> Result<Circle2, Failure> {
                self.this_solution(index)
            }
            fn qualifiers(&self, index: i32) -> Result<(Position, Position), Failure> {
                self.which_qualifier(index)
            }
            fn same(&self, arg: usize, index: i32) -> Result<bool, Failure> {
                if arg == 0 {
                    self.is_the_same1(index)
                } else {
                    self.is_the_same2(index)
                }
            }
            fn tangency(&self, arg: usize, index: i32) -> Result<(f64, f64, Point2), Failure> {
                if arg == 0 {
                    self.tangency1(index)
                } else {
                    self.tangency2(index)
                }
            }
        }
    };
}
two_tangency_solver!(crate::gcc_ana::Circ2d2TanRad);
two_tangency_solver!(crate::gcc_ana::Circ2d2TanOn);
two_tangency_solver!(super::Circ2d2TanRadGeo);
two_tangency_solver!(super::Circ2d2TanOnGeo);

impl CircleSolutions {
    /// `WhichQualifier` of solutions 1 to NbrSol, as each constructor copies them.
    pub(crate) fn store_qualifiers(
        &mut self,
        source: &impl TwoTangencySolver,
    ) -> Result<(), Failure> {
        for i in 1..=self.nbr_sol {
            let (q1, q2) = source.qualifiers(i)?;
            self.qualifier[0].set(i, q1)?;
            self.qualifier[1].set(i, q2)?;
        }
        Ok(())
    }

    /// `Results(Circ)` of Circ2d2TanRad: for solutions 1 to NbrSol the circle, both
    /// TheSame flags, then both tangencies (a tangency on an argument identical to
    /// the solution raises StdFail_NotDone, as natively).
    pub(crate) fn store_results(&mut self, source: &impl TwoTangencySolver) -> Result<(), Failure> {
        for j in 1..=self.nbr_sol {
            self.cirsol.set(j, source.solution(j)?)?;
            let same = [source.same(0, j)?, source.same(1, j)?];
            for (arg, same) in same.into_iter().enumerate() {
                self.tangency[arg].same.set(j, i32::from(same))?;
            }
            for (arg, same) in same.into_iter().enumerate() {
                self.store_tangency(arg, j, same, source.tangency(arg, j)?)?;
            }
        }
        Ok(())
    }
}

// The side test of a curve from the angle between the centre direction and the
// tangent.
pub(crate) fn curve_side(q: &super::QCurve, angle: f64) -> bool {
    q.is_unqualified()
        || (q.is_enclosing() && angle <= 0.0)
        || (q.is_outside() && angle >= 0.0)
        || (q.is_enclosed() && angle <= 0.0)
}

// The side test of a circle of radius `r1` from the solution radius and the
// distance between the centres.
pub(crate) fn circle_side(q: Position, rsol: f64, r1: f64, dist: f64) -> bool {
    q == Position::Unqualified
        || (q == Position::Enclosing && rsol >= r1 && dist <= rsol)
        || (q == Position::Outside && dist >= rsol)
        || (q == Position::Enclosed && rsol <= r1 && dist <= rsol)
}

// The side test of a line from the scalar product with its normal.
pub(crate) fn line_side(q: Position, pscal: f64) -> bool {
    q == Position::Unqualified
        || (q == Position::Outside && pscal <= 0.0)
        || (q == Position::Enclosed && pscal >= 0.0)
}
