// Rust translation of Geom2dGcc_Lin2dTanObl.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Lin2dTanObl`: lines tangent to a curve at a given angle to a line.
use super::{CurveTool, Lin2dTanOblIter, QualifiedCurve, circle_of};
use crate::gcc_ana;
use crate::gcc_ana::solutions::{Slots, Tangencies, check_index, check_solution};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

/// `Geom2dGcc_Lin2dTanObl`: the lines tangent to a qualified curve that make a
/// given angle with a line (at most two): GccAna_Lin2dTanObl for a circle, else
/// Geom2dGcc_Lin2dTanOblIter from 21 samples (or from one given parameter).
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Lin2dTanObl, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&circle)?)?;
/// let x_axis = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let lin = Lin2dTanObl::new(&q, x_axis, 1e-9, std::f64::consts::FRAC_PI_2)?;
/// assert_eq!(lin.nb_solutions(), 2);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2dTanObl {
    well_done: bool,
    nbr_sol: i32,
    linsol: Slots<Line2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
    intersection2: Tangencies,
}

impl Lin2dTanObl {
    fn empty() -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            linsol: Slots::new(2),
            qualifier1: Slots::new(2),
            tangency1: Tangencies::new(2),
            intersection2: Tangencies::new(2),
        }
    }

    /// `Geom2dGcc_Lin2dTanObl(Qualified1, TheLin, TolAng, Angle)`: GccAna for a
    /// circle, else the iteration from 21 parameters spread over the curve, keeping
    /// the first two distinct solutions tangent within `TolAng`. The iteration's
    /// Geom2dGcc_IsParallel (a parallel solution, `Angle` within `TolAng` of 0 or PI)
    /// leaves the constructor, as natively; `Lin2dTanObl::new(&q, line, 1e-9, 0.5)?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        the_line: Line2,
        tol_ang: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty();
        let c1: &Geom2dAdaptorCurve = qualified1.curve();
        if c1.get_type()? == CurveType::Circle {
            lin.ana_circle(qualified1, the_line, angle)?;
            return Ok(lin);
        }
        let a_first_par: f64 = CurveTool::first_parameter(c1)?;
        let a_last_par: f64 = CurveTool::last_parameter(c1)?;
        let a_nb_samples: i32 = CurveTool::nb_samples(c1);
        let a_step: f64 = (a_last_par - a_first_par) / f64::from(a_nb_samples);
        let mut param1: f64 = a_first_par;
        let mut i: i32 = 0;
        while i <= a_nb_samples && lin.nbr_sol < 2 {
            let iter = Lin2dTanOblIter::new(qualified1, the_line, param1, tol_ang, angle)?;
            if iter.is_done() && lin.add(lin.nbr_sol + 1, &iter, tol_ang, c1)? {
                lin.nbr_sol += 1;
            }
            param1 += a_step;
            i += 1;
        }
        lin.well_done = lin.nbr_sol > 0;
        Ok(lin)
    }

    /// `Geom2dGcc_Lin2dTanObl(Qualified1, TheLin, TolAng, Param1, Angle)`: GccAna for
    /// a circle, else one iteration. Natively the iteration receives `TolAng` as its
    /// start parameter and `Param1` as its tolerance, and the solution it finds is
    /// stored without counting it (NbSolutions stays 0); both are kept.
    /// `Lin2dTanObl::with_parameter(&q, line, 1e-9, 0.0, 0.5)?`.
    pub fn with_parameter(
        qualified1: &QualifiedCurve,
        the_line: Line2,
        tol_ang: f64,
        param1: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty();
        if qualified1.curve().get_type()? == CurveType::Circle {
            lin.ana_circle(qualified1, the_line, angle)?;
            return Ok(lin);
        }
        let iter = Lin2dTanOblIter::new(qualified1, the_line, tol_ang, param1, angle)?;
        lin.well_done = iter.is_done();
        if lin.well_done {
            lin.linsol.set(1, iter.this_solution()?)?;
            let (par_sol, par_arg, point) = iter.tangency1()?;
            lin.tangency1.record(1, point, par_sol, par_arg)?;
            let (par_sol, par_arg, point) = iter.intersection2()?;
            lin.intersection2.record(1, point, par_sol, par_arg)?;
            lin.qualifier1.set(1, iter.which_qualifier()?)?;
        }
        Ok(lin)
    }

    // The GccAna_Lin2dTanObl solutions of a circle argument.
    fn ana_circle(&mut self, q1: &QualifiedCurve, line: Line2, angle: f64) -> Result<(), Failure> {
        let qc1 = QualifiedCirc::new(circle_of(q1.curve())?, q1.qualifier());
        let ana = gcc_ana::Lin2dTanObl::tangent_to_circle(qc1, line, angle)?;
        self.well_done = ana.is_done();
        if !self.well_done {
            return Ok(());
        }
        self.nbr_sol = ana.nb_solutions()?;
        for i in 1..=self.nbr_sol {
            self.linsol.set(i, ana.this_solution(i)?)?;
            let (par_sol, par_arg, point) = ana.tangency1(i)?;
            self.tangency1.record(i, point, par_sol, par_arg)?;
            let (par_sol, par_arg, point) = ana.intersection2(i)?;
            self.intersection2.record(i, point, par_sol, par_arg)?;
            self.qualifier1.set(i, ana.which_qualifier(i)?)?;
        }
        Ok(())
    }

    // `Add(theIndex, theLin, theTol, theC1)`: stores the iteration's solution unless
    // an earlier one has the same argument parameters within `the_tol`, or the line
    // is not tangent to the curve within `the_tol` (gp_Dir2d of a null tangent
    // raises Standard_ConstructionError).
    fn add(
        &mut self,
        the_index: i32,
        the_lin: &Lin2dTanOblIter,
        the_tol: f64,
        the_c1: &Geom2dAdaptorCurve,
    ) -> Result<bool, Failure> {
        let a_lin: Line2 = the_lin.this_solution()?;
        let (a_par1sol, a_par1arg, a_pnt1sol) = the_lin.tangency1()?;
        let (a_par2sol, a_par2arg, a_pnt2sol) = the_lin.intersection2()?;
        for i in 1..the_index {
            if (a_par1arg - self.tangency1.par_arg.get(i)?).abs() <= the_tol
                && (a_par2arg - self.intersection2.par_arg.get(i)?).abs() <= the_tol
            {
                return Ok(false);
            }
        }
        let (_, a_v_tan) = CurveTool::d1(the_c1, a_par1arg)?;
        if a_lin
            .direction()
            .crossed(Direction2::from_vector(a_v_tan)?)
            .abs()
            > the_tol
        {
            return Ok(false);
        }
        self.linsol.set(the_index, a_lin)?;
        self.tangency1
            .record(the_index, a_pnt1sol, a_par1sol, a_par1arg)?;
        self.intersection2
            .record(the_index, a_pnt2sol, a_par2sol, a_par2arg)?;
        self.qualifier1.set(the_index, the_lin.which_qualifier()?)?;
        Ok(true)
    }

    /// `IsDone()`; `lin.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `NbSolutions()`; `lin.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// `ThisSolution(Index)`: Standard_OutOfRange outside `1..=NbSolutions()` (no
    /// done test, as natively); `lin.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Line2, Failure> {
        check_index(self.nbr_sol, index, "Geom2dGcc_Lin2dTanObl::ThisSolution")?;
        self.linsol.get(index)
    }

    /// `WhichQualifier(Index, Qualif1)`; `lin.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "Geom2dGcc_Lin2dTanObl::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`: the tangency on the curve;
    /// `let (par_sol, par_arg, point) = lin.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Lin2dTanObl::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }

    /// `Intersection2(Index, ParSol, ParArg, PntSol)`: the intersection with the
    /// line; `let (par_sol, par_arg, point) = lin.intersection2(1)?;`.
    pub fn intersection2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Lin2dTanObl::Intersection2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.intersection2.raw(index)
    }
}
