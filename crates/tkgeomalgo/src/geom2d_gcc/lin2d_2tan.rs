// Rust translation of Geom2dGcc_Lin2d2Tan.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Lin2d2Tan`: lines tangent to two curves, or to a curve and through
//! a point.
use super::{CurveTool, Lin2d2TanIter, QualifiedCurve, circle_of};
use crate::gcc_ana;
use crate::gcc_ana::solutions::{Slots, Tangencies, check_index, check_solution};
use crate::gcc_ent::{Position, QualifiedCirc};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

/// `Geom2dGcc_Lin2d2Tan`: the lines tangent to two qualified curves (up to four) or
/// tangent to a qualified curve and through a point (up to two): GccAna_Lin2d2Tan
/// for circles, else Geom2dGcc_Lin2d2TanIter from 21 samples of each curve (or
/// from given parameters).
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Lin2d2Tan, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&circle)?)?;
/// let lin = Lin2d2Tan::through_point(&q, Point2::new(3.0, 0.0), 1e-9)?;
/// assert_eq!(lin.nb_solutions(), 2);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2d2Tan {
    well_done: bool,
    nbr_sol: i32,
    linsol: Slots<Line2>,
    qualifier1: Slots<Position>,
    qualifier2: Slots<Position>,
    tangency1: Tangencies,
    tangency2: Tangencies,
}

impl Lin2d2Tan {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            linsol: Slots::new(size),
            qualifier1: Slots::new(size),
            qualifier2: Slots::new(size),
            tangency1: Tangencies::new(size),
            tangency2: Tangencies::new(size),
        }
    }

    /// `Geom2dGcc_Lin2d2Tan(Qualified1, Qualified2, Tolang)`: GccAna for two circles,
    /// else the iteration from the 21 x 21 sample pairs, keeping the first four
    /// distinct solutions tangent within `Tolang`; `Lin2d2Tan::new(&q1, &q2, 1e-9)?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty(4);
        if lin.ana_circles(qualified1, qualified2, tolang)? {
            return Ok(lin);
        }
        let (c1, c2) = (qualified1.curve(), qualified2.curve());
        let (a1_f_par, a_step1) = sampling(c1)?;
        let (a2_f_par, a_step2) = sampling(c2)?;
        let mut param1: f64 = a1_f_par;
        let mut i: i32 = 0;
        while i <= CurveTool::nb_samples(c1) && lin.nbr_sol < 4 {
            let mut param2: f64 = a2_f_par;
            let mut j: i32 = 0;
            while j <= CurveTool::nb_samples(c2) && lin.nbr_sol < 4 {
                let iter = Lin2d2TanIter::new(qualified1, qualified2, param1, param2, tolang)?;
                if iter.is_done() && lin.add(lin.nbr_sol + 1, &iter, tolang, c1, Some(c2))? {
                    lin.nbr_sol += 1;
                }
                param2 += a_step2;
                j += 1;
            }
            param1 += a_step1;
            i += 1;
        }
        lin.well_done = lin.nbr_sol > 0;
        Ok(lin)
    }

    /// `Geom2dGcc_Lin2d2Tan(Qualified1, ThePoint, Tolang)`: GccAna for a circle, else
    /// the iteration from 21 parameters, keeping the first two distinct solutions
    /// tangent within `Tolang`; `Lin2d2Tan::through_point(&q, point, 1e-9)?`.
    pub fn through_point(
        qualified1: &QualifiedCurve,
        the_point: Point2,
        tolang: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty(2);
        if lin.ana_circle_point(qualified1, the_point, tolang)? {
            return Ok(lin);
        }
        let c1 = qualified1.curve();
        let (a_first_par, a_step) = sampling(c1)?;
        let mut param1: f64 = a_first_par;
        let mut i: i32 = 0;
        while i <= CurveTool::nb_samples(c1) && lin.nbr_sol < 2 {
            let iter = Lin2d2TanIter::through_point(qualified1, the_point, param1, tolang)?;
            // Natively the second curve is an empty Geom2dAdaptor_Curve: not tested.
            if iter.is_done() && lin.add(lin.nbr_sol + 1, &iter, tolang, c1, None)? {
                lin.nbr_sol += 1;
            }
            param1 += a_step;
            i += 1;
        }
        lin.well_done = lin.nbr_sol > 0;
        Ok(lin)
    }

    /// `Geom2dGcc_Lin2d2Tan(Qualified1, Qualified2, Tolang, Param1, Param2)`: GccAna
    /// for two circles, else one iteration from (`param1`, `param2`);
    /// `Lin2d2Tan::with_parameters(&q1, &q2, 1e-9, 0.0, 0.0)?`.
    pub fn with_parameters(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        tolang: f64,
        param1: f64,
        param2: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty(4);
        if lin.ana_circles(qualified1, qualified2, tolang)? {
            return Ok(lin);
        }
        let iter = Lin2d2TanIter::new(qualified1, qualified2, param1, param2, tolang)?;
        lin.store_single(&iter)?;
        Ok(lin)
    }

    /// `Geom2dGcc_Lin2d2Tan(Qualified1, ThePoint, Tolang, Param1)`: GccAna for a
    /// circle, else one iteration from `param1`;
    /// `Lin2d2Tan::through_point_with_parameter(&q, point, 1e-9, 0.0)?`.
    pub fn through_point_with_parameter(
        qualified1: &QualifiedCurve,
        the_point: Point2,
        tolang: f64,
        param1: f64,
    ) -> Result<Self, Failure> {
        let mut lin = Self::empty(2);
        if lin.ana_circle_point(qualified1, the_point, tolang)? {
            return Ok(lin);
        }
        let iter = Lin2d2TanIter::through_point(qualified1, the_point, param1, tolang)?;
        lin.store_single(&iter)?;
        Ok(lin)
    }

    // GccAna_Lin2d2Tan of two circle arguments; false for other curves.
    fn ana_circles(
        &mut self,
        q1: &QualifiedCurve,
        q2: &QualifiedCurve,
        tol: f64,
    ) -> Result<bool, Failure> {
        if q1.curve().get_type()? != CurveType::Circle
            || q2.curve().get_type()? != CurveType::Circle
        {
            return Ok(false);
        }
        let qc1 = QualifiedCirc::new(circle_of(q1.curve())?, q1.qualifier());
        let qc2 = QualifiedCirc::new(circle_of(q2.curve())?, q2.qualifier());
        self.store_ana(&gcc_ana::Lin2d2Tan::tangent_to_circles(qc1, qc2, tol)?)?;
        Ok(true)
    }

    // GccAna_Lin2d2Tan of a circle argument and a point; false for other curves.
    fn ana_circle_point(
        &mut self,
        q1: &QualifiedCurve,
        point: Point2,
        tol: f64,
    ) -> Result<bool, Failure> {
        if q1.curve().get_type()? != CurveType::Circle {
            return Ok(false);
        }
        let qc1 = QualifiedCirc::new(circle_of(q1.curve())?, q1.qualifier());
        self.store_ana(&gcc_ana::Lin2d2Tan::tangent_through_point(qc1, point, tol)?)?;
        Ok(true)
    }

    // The solutions of a done GccAna_Lin2d2Tan.
    fn store_ana(&mut self, ana: &gcc_ana::Lin2d2Tan) -> Result<(), Failure> {
        self.well_done = ana.is_done();
        if !self.well_done {
            return Ok(());
        }
        self.nbr_sol = ana.nb_solutions();
        for i in 1..=self.nbr_sol {
            self.linsol.set(i, ana.this_solution(i)?)?;
            let (par_sol, par_arg, point) = ana.tangency1(i)?;
            self.tangency1.record(i, point, par_sol, par_arg)?;
            let (par_sol, par_arg, point) = ana.tangency2(i)?;
            self.tangency2.record(i, point, par_sol, par_arg)?;
            let (q1, q2) = ana.which_qualifier(i)?;
            self.qualifier1.set(i, q1)?;
            self.qualifier2.set(i, q2)?;
        }
        Ok(())
    }

    // The single solution of a given-parameter iteration, when it is done.
    fn store_single(&mut self, iter: &Lin2d2TanIter) -> Result<(), Failure> {
        self.well_done = iter.is_done();
        if !self.well_done {
            return Ok(());
        }
        self.nbr_sol = 1;
        self.linsol.set(1, iter.this_solution()?)?;
        let (par_sol, par_arg, point) = iter.tangency1()?;
        self.tangency1.record(1, point, par_sol, par_arg)?;
        let (par_sol, par_arg, point) = iter.tangency2()?;
        self.tangency2.record(1, point, par_sol, par_arg)?;
        let (q1, q2) = iter.which_qualifier()?;
        self.qualifier1.set(1, q1)?;
        self.qualifier2.set(1, q2)
    }

    // `Add(theIndex, theLin, theTol, theC1, theC2)`: stores the iteration's solution
    // unless an earlier one has the same argument parameters within `the_tol`, or the
    // line is not tangent within `the_tol` to the first curve, then to the second
    // one when it is loaded (gp_Dir2d of a null tangent raises
    // Standard_ConstructionError).
    fn add(
        &mut self,
        the_index: i32,
        the_lin: &Lin2d2TanIter,
        the_tol: f64,
        the_c1: &Geom2dAdaptorCurve,
        the_c2: Option<&Geom2dAdaptorCurve>,
    ) -> Result<bool, Failure> {
        let a_lin: Line2 = the_lin.this_solution()?;
        let (a_par1sol, a_par1arg, a_pnt1sol) = the_lin.tangency1()?;
        let (a_par2sol, a_par2arg, a_pnt2sol) = the_lin.tangency2()?;
        for i in 1..the_index {
            if (a_par1arg - self.tangency1.par_arg.get(i)?).abs() <= the_tol
                && (a_par2arg - self.tangency2.par_arg.get(i)?).abs() <= the_tol
            {
                return Ok(false);
            }
        }
        let a_lin_dir: Direction2 = a_lin.direction();
        if leaves_tangency(a_lin_dir, the_c1, a_par1arg, the_tol)? {
            return Ok(false);
        }
        if let Some(c2) = the_c2.filter(|c2| !c2.curve().is_null())
            && leaves_tangency(a_lin_dir, c2, a_par2arg, the_tol)?
        {
            return Ok(false);
        }
        self.linsol.set(the_index, a_lin)?;
        self.tangency1
            .record(the_index, a_pnt1sol, a_par1sol, a_par1arg)?;
        self.tangency2
            .record(the_index, a_pnt2sol, a_par2sol, a_par2arg)?;
        let (q1, q2) = the_lin.which_qualifier()?;
        self.qualifier1.set(the_index, q1)?;
        self.qualifier2.set(the_index, q2)?;
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
        check_index(self.nbr_sol, index, "Geom2dGcc_Lin2d2Tan::ThisSolution")?;
        self.linsol.get(index)
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2)`; `let (q1, q2) = lin.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let member = "Geom2dGcc_Lin2d2Tan::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.qualifier1.get(index)?, self.qualifier2.get(index)?))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = lin.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Lin2d2Tan::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = lin.tangency2(1)?;`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "Geom2dGcc_Lin2d2Tan::Tangency2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.raw(index)
    }
}

// (first parameter, step) of the 20 sample intervals of a curve.
fn sampling(c: &Geom2dAdaptorCurve) -> Result<(f64, f64), Failure> {
    let first: f64 = CurveTool::first_parameter(c)?;
    let last: f64 = CurveTool::last_parameter(c)?;
    Ok((first, (last - first) / f64::from(CurveTool::nb_samples(c))))
}

// |Dir ^ gp_Dir2d(T(u))| > tol: the line is not tangent to the curve at `u` (a
// NaN angle is kept, as natively).
fn leaves_tangency(
    dir: Direction2,
    c: &Geom2dAdaptorCurve,
    u: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let (_, a_v_tan) = CurveTool::d1(c, u)?;
    Ok(dir.crossed(Direction2::from_vector(a_v_tan)?).abs() > tol)
}
