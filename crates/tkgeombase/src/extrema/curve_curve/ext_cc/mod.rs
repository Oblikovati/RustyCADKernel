// Rust translation of Extrema_ExtCC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtCC`: the extrema between two 3D curves: Extrema_ExtElC for a line
//! with a line or conic and for two circles, Extrema_ECC otherwise; parallel
//! results are reduced to the trimmed curves.
mod parallel;
mod results;

use super::ECC;
use crate::extrema::ExtElC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::curve_tools::CurveTool;
use crate::extrema::failures::{
    extremum_index, not_done, null_object, out_of_range, sequence_value,
};
use crate::extrema::pon::POnCurv;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

/// `Extrema_ExtCC`. Its Extrema_ECC member is kept across Performs, so the general
/// case appends to the ECC solutions of earlier calls, as natively;
/// `ExtCC::with_curves(&c1, &c2, 1.0e-10, 1.0e-10)?.nb_ext()?`.
pub struct ExtCC<'a> {
    is_find_single_solution: bool,
    ecc: ECC<'a>,
    done: bool,
    is_parallel: bool,
    points: Vec<POnCurv>,
    sq_dist: Vec<f64>,
    curves: [Option<&'a dyn Adaptor3dCurve>; 2],
    inf: [f64; 2],
    sup: [f64; 2],
    tol: [f64; 2],
    p1f: Point3,
    p1l: Point3,
    p2f: Point3,
    p2l: Point3,
    dist: [f64; 4],
}

impl<'a> ExtCC<'a> {
    /// `Extrema_ExtCC(TolC1, TolC2)` (the native defaults are 1e-10): no curves,
    /// infinite ranges; `ExtCC::new(1.0e-10, 1.0e-10)`.
    pub fn new(tol_c1: f64, tol_c2: f64) -> Self {
        let infinite: f64 = precision::infinite();
        Self {
            is_find_single_solution: false,
            ecc: ECC::new(),
            done: false,
            is_parallel: false,
            points: Vec::new(),
            sq_dist: Vec::new(),
            curves: [None, None],
            inf: [-infinite, -infinite],
            sup: [infinite, infinite],
            tol: [tol_c1, tol_c2],
            p1f: Point3::default(),
            p1l: Point3::default(),
            p2f: Point3::default(),
            p2l: Point3::default(),
            dist: [f64::MIN; 4],
        }
    }
    /// `Extrema_ExtCC(C1, C2, TolC1, TolC2)`: on the curve ranges;
    /// `ExtCC::with_curves(&c1, &c2, 1.0e-10, 1.0e-10)?`.
    pub fn with_curves(
        c1: &'a dyn Adaptor3dCurve,
        c2: &'a dyn Adaptor3dCurve,
        tol_c1: f64,
        tol_c2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new(tol_c1, tol_c2);
        extrema.ecc = ECC::with_curves(c1, c2)?;
        extrema.initialize(c1, c2, tol_c1, tol_c2)?;
        extrema.perform()?;
        Ok(extrema)
    }
    /// `Extrema_ExtCC(C1, C2, U1, U2, V1, V2, TolC1, TolC2)`;
    /// `ExtCC::with_ranges(&c1, &c2, (0.0, 1.0), (0.0, 1.0), 1.0e-10, 1.0e-10)?`.
    pub fn with_ranges(
        c1: &'a dyn Adaptor3dCurve,
        c2: &'a dyn Adaptor3dCurve,
        u: (f64, f64),
        v: (f64, f64),
        tol_c1: f64,
        tol_c2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new(tol_c1, tol_c2);
        extrema.ecc = ECC::with_bounds(c1, c2, u, v);
        extrema.initialize_ranges(c1, c2, u, v, (tol_c1, tol_c2))?;
        extrema.perform()?;
        Ok(extrema)
    }
    /// `Initialize(C1, C2, TolC1, TolC2)`: not done, on the curve ranges;
    /// `extrema.initialize(&c1, &c2, 1.0e-10, 1.0e-10)?`.
    pub fn initialize(
        &mut self,
        c1: &'a dyn Adaptor3dCurve,
        c2: &'a dyn Adaptor3dCurve,
        tol_c1: f64,
        tol_c2: f64,
    ) -> Result<(), Failure> {
        let u: (f64, f64) = (c1.first_parameter()?, c1.last_parameter()?);
        let v: (f64, f64) = (c2.first_parameter()?, c2.last_parameter()?);
        self.initialize_ranges(c1, c2, u, v, (tol_c1, tol_c2))
    }
    /// `Initialize(C1, C2, U1, U2, V1, V2, TolC1, TolC2)`;
    /// `extrema.initialize_ranges(&c1, &c2, (0.0, 1.0), (0.0, 1.0), (1e-10, 1e-10))?`.
    pub fn initialize_ranges(
        &mut self,
        c1: &'a dyn Adaptor3dCurve,
        c2: &'a dyn Adaptor3dCurve,
        (u1, u2): (f64, f64),
        (v1, v2): (f64, f64),
        (tol_c1, tol_c2): (f64, f64),
    ) -> Result<(), Failure> {
        // myECC will be re-initialized by Perform()
        self.done = false;
        self.set_curve_range(1, c1, u1, u2)?;
        self.set_curve_range(2, c2, v1, v2)?;
        self.set_tolerance(1, tol_c1)?;
        self.set_tolerance(2, tol_c2)?;
        self.dist = [f64::MIN; 4];
        Ok(())
    }
    // The 0-based index of `rank`, Standard_OutOfRange outside 1 and 2.
    fn rank_index(member: &str, rank: i32) -> Result<usize, Failure> {
        if !(1..=2).contains(&rank) {
            return Err(out_of_range(member, rank, 2));
        }
        Ok((rank - 1) as usize)
    }
    /// `SetCurve(theRank, C)`; `extrema.set_curve(1, &c1)?`.
    pub fn set_curve(&mut self, rank: i32, curve: &'a dyn Adaptor3dCurve) -> Result<(), Failure> {
        self.curves[Self::rank_index("Extrema_ExtCC::SetCurve()", rank)?] = Some(curve);
        Ok(())
    }
    /// `SetCurve(theRank, C, Uinf, Usup)`; `extrema.set_curve_range(1, &c1, 0.0, 1.0)?`.
    pub fn set_curve_range(
        &mut self,
        rank: i32,
        curve: &'a dyn Adaptor3dCurve,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.set_curve(rank, curve)?;
        self.set_range(rank, uinf, usup)
    }
    /// `SetRange(theRank, Uinf, Usup)`; `extrema.set_range(1, 0.0, 1.0)?`.
    pub fn set_range(&mut self, rank: i32, uinf: f64, usup: f64) -> Result<(), Failure> {
        let index: usize = Self::rank_index("Extrema_ExtCC::SetRange()", rank)?;
        (self.inf[index], self.sup[index]) = (uinf, usup);
        Ok(())
    }
    /// `SetTolerance(theRank, Tol)`; `extrema.set_tolerance(1, 1e-10)?`.
    pub fn set_tolerance(&mut self, rank: i32, tol: f64) -> Result<(), Failure> {
        self.tol[Self::rank_index("Extrema_ExtCC::SetTolerance()", rank)?] = tol;
        Ok(())
    }
    /// `SetSingleSolutionFlag(theFlag)`; `extrema.set_single_solution_flag(true)`.
    pub fn set_single_solution_flag(&mut self, flag: bool) {
        self.is_find_single_solution = flag;
    }
    /// `GetSingleSolutionFlag()`; `extrema.get_single_solution_flag()`.
    pub fn get_single_solution_flag(&self) -> bool {
        self.is_find_single_solution
    }
    fn curve_pair(&self) -> Result<[&'a dyn Adaptor3dCurve; 2], Failure> {
        let missing = || null_object("Extrema_ExtCC::Perform()");
        Ok([
            self.curves[0].ok_or_else(missing)?,
            self.curves[1].ok_or_else(missing)?,
        ])
    }
}

impl ExtCC<'_> {
    /// `Perform()`; Standard_NullObject without both curves; `extrema.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let [c1, c2] = self.curve_pair()?;
        self.ecc.set_params(
            c1,
            c2,
            (self.inf[0], self.sup[0]),
            (self.inf[1], self.sup[1]),
        );
        self.ecc.set_tolerance(std_min(self.tol[0], self.tol[1]));
        self.ecc
            .set_single_solution_flag(self.get_single_solution_flag());
        self.done = false;
        self.points.clear();
        self.sq_dist.clear();
        self.is_parallel = false;
        let (type1, type2) = (c1.get_type()?, c2.get_type()?);
        let tol: f64 = std_min(self.tol[0], self.tol[1]);
        let ranges: [f64; 4] = [self.inf[0], self.sup[0], self.inf[1], self.sup[1]];
        self.end_distances(c1, c2, ranges)?;
        let is_elementary = |t: CurveType| (t as i32) <= (CurveType::Parabola as i32);
        if (type1 == CurveType::Line && is_elementary(type2))
            || (type2 == CurveType::Line && is_elementary(type1))
        {
            return self.perform_line_conic([c1, c2], (type1, type2), tol, ranges);
        }
        if type1 == CurveType::Circle && type2 == CurveType::Circle {
            let xtrem = ExtElC::circles(c1.circle()?, c2.circle()?)?;
            if xtrem.is_done() {
                return self.prepare_results_el(&xtrem, false, ranges);
            }
        }
        self.ecc.perform()?;
        self.prepare_results_ecc(ranges)
    }

    // The analytic case: one curve is always a line (the first argument of
    // Extrema_ExtElC, which inverts the order when the line is the second curve).
    fn perform_line_conic(
        &mut self,
        curves: [&dyn Adaptor3dCurve; 2],
        (type1, type2): (CurveType, CurveType),
        tol: f64,
        ranges: [f64; 4],
    ) -> Result<(), Failure> {
        let is_inverse: bool = (type1 as i32) > (type2 as i32);
        let (line, other, other_type) = if is_inverse {
            (curves[1], curves[0], type1)
        } else {
            (curves[0], curves[1], type2)
        };
        let xtrem = match other_type {
            CurveType::Line => ExtElC::lines(line.line()?, other.line()?, tol),
            CurveType::Circle => ExtElC::line_circle(line.line()?, other.circle()?, tol)?,
            CurveType::Ellipse => ExtElC::line_ellipse(line.line()?, other.ellipse()?)?,
            CurveType::Hyperbola => ExtElC::line_hyperbola(line.line()?, other.hyperbola()?)?,
            _ => ExtElC::line_parabola(line.line()?, other.parabola()?)?,
        };
        self.prepare_results_el(&xtrem, is_inverse, ranges)
    }

    // The end points of the ranges and their square distances (RealLast() with an
    // infinite end).
    fn end_distances(
        &mut self,
        c1: &dyn Adaptor3dCurve,
        c2: &dyn Adaptor3dCurve,
        [u11, u12, u21, u22]: [f64; 4],
    ) -> Result<(), Failure> {
        let finite = |u: f64| !precision::is_infinite(u);
        if finite(u11) {
            self.p1f = CurveTool::value(c1, u11)?;
        }
        if finite(u12) {
            self.p1l = CurveTool::value(c1, u12)?;
        }
        if finite(u21) {
            self.p2f = CurveTool::value(c2, u21)?;
        }
        if finite(u22) {
            self.p2l = CurveTool::value(c2, u22)?;
        }
        let distance = |a: f64, b: f64, pa: Point3, pb: Point3| {
            if finite(a) && finite(b) {
                pa.square_distance(&pb)
            } else {
                f64::MAX
            }
        };
        self.dist = [
            distance(u11, u21, self.p1f, self.p2f),
            distance(u11, u22, self.p1f, self.p2l),
            distance(u12, u21, self.p1l, self.p2f),
            distance(u12, u22, self.p1l, self.p2l),
        ];
        Ok(())
    }

    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCC::IsParallel()"));
        }
        Ok(self.is_parallel)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCC::NbExt()"));
        }
        Ok(self.sq_dist.len() as i32)
    }
    /// `SquareDistance(N)` (the native default is 1); `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let member: &str = "Extrema_ExtCC::SquareDistance()";
        extremum_index(member, n, || self.nb_ext())?;
        sequence_value(member, &self.sq_dist, n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; a parallel result may hold a
    /// distance without points (Standard_OutOfRange then); `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnCurv, POnCurv), Failure> {
        let member: &str = "Extrema_ExtCC::Points()";
        extremum_index(member, n, || self.nb_ext())?;
        let first: POnCurv = sequence_value(member, &self.points, 2 * n - 1)?;
        Ok((first, sequence_value(member, &self.points, 2 * n)?))
    }
    /// `TrimmedSquareDistances(dist11, dist12, dist21, dist22, P11, P12, P21, P22)`,
    /// returned as `([dist11, dist12, dist21, dist22], [P11, P12, P21, P22])`;
    /// `extrema.trimmed_square_distances()`.
    pub fn trimmed_square_distances(&self) -> ([f64; 4], [Point3; 4]) {
        (self.dist, [self.p1f, self.p1l, self.p2f, self.p2l])
    }
    // `ClearSolutions()` (protected): the extrema, not the flags.
    fn clear_solutions(&mut self) {
        self.sq_dist.clear();
        self.points.clear();
    }
}
