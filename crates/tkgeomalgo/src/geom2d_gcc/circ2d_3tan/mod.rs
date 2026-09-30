// Rust translation of Geom2dGcc_Circ2d3Tan.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d3Tan`: circles tangent to three curves or points.
mod analytic;

use super::circle_solutions::CircleSolutions;
use super::{Circ2d3TanIter, QualifiedCurve, point_of};
use crate::gcc_ana;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkmath::gp::{Circle2, Point2};

/// `Geom2dGcc_Circ2d3Tan`: the circles tangent to three qualified curves, to two
/// curves and through a point, to a curve and through two points, or through
/// three points: GccAna_Circ2d3Tan when every curve is a line or a circle, else one
/// Geom2dGcc_Circ2d3TanIter from the given parameters (whose single solution
/// counts even when the search failed).
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d3Tan, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let circle = |x: f64, y: f64| -> Result<_, rustycad_tkernel::standard::Failure> {
///     let axis = Axis2d::new(Point2::new(x, y), Direction2::new(1.0, 0.0)?);
///     let c: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 2.0, true)?).cast();
///     unqualified(&Geom2dAdaptorCurve::with_curve(&c)?)
/// };
/// let (q1, q2, q3) = (circle(0.0, 0.0)?, circle(10.0, 0.0)?, circle(5.0, 8.0)?);
/// let circ = Circ2d3Tan::new(&q1, &q2, &q3, 1e-9, [0.0; 3])?;
/// assert!(circ.nb_solutions() >= 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d3Tan {
    solutions: CircleSolutions,
}

impl Circ2d3Tan {
    fn empty(size: usize) -> Self {
        Self {
            solutions: CircleSolutions::new(size, 3),
        }
    }

    /// `Geom2dGcc_Circ2d3Tan(Qualified1, Qualified2, Qualified3, Tolerance, Param1,
    /// Param2, Param3)`; `Circ2d3Tan::new(&q1, &q2, &q3, 1e-9, [0.0; 3])?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        qualified3: &QualifiedCurve,
        tolerance: f64,
        params: [f64; 3],
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(16);
        let qs = [qualified1, qualified2, qualified3];
        if !circ.three_analytic(qs, tolerance)? {
            let iter = Circ2d3TanIter::new(qualified1, qualified2, qualified3, params, tolerance)?;
            circ.store_iter(&iter)?;
        }
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d3Tan(Qualified1, Qualified2, Point, Tolerance, Param1,
    /// Param2)`; `Circ2d3Tan::with_point(&q1, &q2, &point, 1e-9, [0.0; 2])?`.
    pub fn with_point(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        point: &Handle<dyn Geom2dPoint>,
        tolerance: f64,
        [param1, param2]: [f64; 2],
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(20);
        if !circ.two_analytic_point([qualified1, qualified2], point, tolerance)? {
            let pnt: Point2 = point_of(point)?;
            let params = [param1, param2];
            let iter =
                Circ2d3TanIter::curve_curve_pnt(qualified1, qualified2, pnt, params, tolerance)?;
            circ.store_iter(&iter)?;
        }
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d3Tan(Qualified1, Point1, Point2, Tolerance, Param1)`;
    /// `Circ2d3Tan::with_points(&q, &p1, &p2, 1e-9, 0.0)?`.
    pub fn with_points(
        qualified1: &QualifiedCurve,
        point1: &Handle<dyn Geom2dPoint>,
        point2: &Handle<dyn Geom2dPoint>,
        tolerance: f64,
        param1: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(16);
        if !circ.one_analytic_points(qualified1, [point1, point2], tolerance)? {
            let (p1, p2) = (point_of(point1)?, point_of(point2)?);
            let iter = Circ2d3TanIter::curve_pnt_pnt(qualified1, p1, p2, param1, tolerance)?;
            circ.store_iter(&iter)?;
        }
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d3Tan(Point1, Point2, Point3, Tolerance)`: GccAna;
    /// `Circ2d3Tan::with_three_points(&p1, &p2, &p3, 1e-9)?`.
    pub fn with_three_points(
        point1: &Handle<dyn Geom2dPoint>,
        point2: &Handle<dyn Geom2dPoint>,
        point3: &Handle<dyn Geom2dPoint>,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty(2);
        let (p1, p2, p3) = (point_of(point1)?, point_of(point2)?, point_of(point3)?);
        let ana = gcc_ana::Circ2d3Tan::pnt_pnt_pnt(p1, p2, p3, tolerance)?;
        circ.store_ana(&ana, [0, 1, 2], [1, 2, 3])?;
        Ok(circ)
    }

    // WellDone, NbrSol, the qualifiers (GccAna argument i into slot `into[i]`) and
    // the Results of a GccAna solver.
    fn store_ana(
        &mut self,
        ana: &gcc_ana::Circ2d3Tan,
        into: [usize; 3],
        ranks: [i32; 3],
    ) -> Result<(), Failure> {
        self.solutions.well_done = ana.is_done();
        self.solutions.nbr_sol = ana.nb_solutions();
        for i in 1..=self.solutions.nbr_sol {
            let (a, b, c) = ana.which_qualifier(i)?;
            for (slot, q) in into.into_iter().zip([a, b, c]) {
                self.solutions.qualifier[slot].set(i, q)?;
            }
        }
        self.results(ana, ranks[0], ranks[1], ranks[2])
    }

    // The single solution of Geom2dGcc_Circ2d3TanIter: NbrSol is 1 even when the
    // search failed, as natively.
    fn store_iter(&mut self, iter: &Circ2d3TanIter) -> Result<(), Failure> {
        let s = &mut self.solutions;
        s.well_done = iter.is_done();
        s.nbr_sol = 1;
        if !s.well_done {
            return Ok(());
        }
        s.cirsol.set(1, iter.this_solution())?;
        let same = [
            iter.is_the_same1()?,
            iter.is_the_same2()?,
            iter.is_the_same3()?,
        ];
        let tangencies = [iter.tangency1()?, iter.tangency2()?, iter.tangency3()?];
        for arg in 0..3 {
            s.store_tangency(arg, 1, same[arg], tangencies[arg])?;
        }
        let (q1, q2, q3) = iter.which_qualifier()?;
        for (arg, q) in [q1, q2, q3].into_iter().enumerate() {
            s.qualifier[arg].set(1, q)?;
        }
        Ok(())
    }

    /// `Results(Circ, Rank1, Rank2, Rank3)`: for solutions 1 to NbSolutions() the
    /// circle of `circ`, and for argument `k` the TheSame flag and (when it is 0)
    /// the tangency of the GccAna argument `Rank_k` (1 to 3; another rank leaves the
    /// argument unset); `solver.results(&ana, 1, 2, 3)?`.
    pub fn results(
        &mut self,
        circ: &gcc_ana::Circ2d3Tan,
        rank1: i32,
        rank2: i32,
        rank3: i32,
    ) -> Result<(), Failure> {
        for j in 1..=self.solutions.nbr_sol {
            self.solutions.cirsol.set(j, circ.this_solution(j)?)?;
            let same = [
                circ.is_the_same1(j)?,
                circ.is_the_same2(j)?,
                circ.is_the_same3(j)?,
            ];
            for (arg, rank) in [rank1, rank2, rank3].into_iter().enumerate() {
                self.result_of(circ, arg, rank, j, same)?;
            }
        }
        Ok(())
    }

    // The TheSame flag and tangency of argument `arg` from GccAna argument `rank`.
    fn result_of(
        &mut self,
        circ: &gcc_ana::Circ2d3Tan,
        arg: usize,
        rank: i32,
        j: i32,
        same: [bool; 3],
    ) -> Result<(), Failure> {
        if !(1..=3).contains(&rank) {
            return Ok(());
        }
        let the_same: bool = same[rank as usize - 1];
        self.solutions.tangency[arg]
            .same
            .set(j, i32::from(the_same))?;
        if the_same {
            return Ok(());
        }
        let tangency = match rank {
            1 => circ.tangency1(j)?,
            2 => circ.tangency2(j)?,
            _ => circ.tangency3(j)?,
        };
        self.solutions.store_tangency(arg, j, false, tangency)
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.solutions.well_done
    }

    /// `NbSolutions()`; `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.solutions.nbr_sol
    }

    /// `ThisSolution(Index)`; `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        self.solutions
            .solution(index, "Geom2dGcc_Circ2d3Tan::ThisSolution")
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2, Qualif3)`;
    /// `let (q1, q2, q3) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position, Position), Failure> {
        let s = &self.solutions;
        s.check(index, "Geom2dGcc_Circ2d3Tan::WhichQualifier")?;
        Ok((
            s.qualifier_of(0, index)?,
            s.qualifier_of(1, index)?,
            s.qualifier_of(2, index)?,
        ))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`: StdFail_NotDone for a solution
    /// identical to the argument; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(0, index, "Geom2dGcc_Circ2d3Tan::Tangency1")
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`; `circ.tangency2(1)?`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(1, index, "Geom2dGcc_Circ2d3Tan::Tangency2")
    }

    /// `Tangency3(Index, ParSol, ParArg, PntSol)`: always StdFail_NotDone after the
    /// iteration, whose IsTheSame3 is true; `circ.tangency3(1)?`.
    pub fn tangency3(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(2, index, "Geom2dGcc_Circ2d3Tan::Tangency3")
    }

    /// `IsTheSame1(Index)`; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(0, index, "Geom2dGcc_Circ2d3Tan::IsTheSame1")
    }

    /// `IsTheSame2(Index)`; `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(1, index, "Geom2dGcc_Circ2d3Tan::IsTheSame2")
    }

    /// `IsTheSame3(Index)`; `circ.is_the_same3(1)?`.
    pub fn is_the_same3(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(2, index, "Geom2dGcc_Circ2d3Tan::IsTheSame3")
    }
}
