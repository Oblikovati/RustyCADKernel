// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_PrjResolve.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_PrjResolve`: the orthogonal projection of a curve point on a surface
//! near a start point, by a Newton search on [`PrjFunc`] with a fallback to the
//! general `math_FunctionSetRoot`.
use super::prj_func::PrjFunc;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::{FunctionSetRoot, NewtonFunctionSetRoot, Vector};

// The native default iteration limit of both solvers.
const NB_ITERATIONS: i32 = 100;

/// The 2D bounds and tolerances of [`PrjResolve::perform`]: `tol` gives the
/// tolerance of each free variable, `inf` and `sup` their bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PrjResolveBox {
    /// `Tol`: the tolerance of each free variable.
    pub tol: Point2,
    /// `Inf`: the lower bounds of the free variables.
    pub inf: Point2,
    /// `Sup`: the upper bounds of the free variables.
    pub sup: Point2,
}

/// `ProjLib_PrjResolve`: `fix` 1 solves (u, v) at fixed t, 2 solves (t, v) at fixed
/// u and 3 solves (t, u) at fixed v.
///
/// ```ignore
/// let mut resolve = PrjResolve::new(&curve, &surface, 1)?;
/// resolve.perform(t, u0, v0, bounds, 1e-10, false)?;
/// if resolve.is_done() { let uv = resolve.solution()?; }
/// ```
pub struct PrjResolve<'a> {
    curve: &'a dyn Adaptor3dCurve,
    surface: &'a dyn Adaptor3dSurface,
    done: bool,
    solution: Point2,
    fix: i32,
}

impl<'a> PrjResolve<'a> {
    /// `ProjLib_PrjResolve(C, S, Fix)`: Standard_ConstructionError unless `fix` is
    /// 1, 2 or 3; `PrjResolve::new(&curve, &surface, 1)?`.
    pub fn new(
        curve: &'a dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
        fix: i32,
    ) -> Result<Self, Failure> {
        if !(1..=3).contains(&fix) {
            return Err(failure(
                "Standard_ConstructionError",
                &format!("ProjLib_PrjResolve: fixed variable index {fix}; expected 1, 2 or 3"),
            ));
        }
        Ok(Self {
            curve,
            surface,
            done: false,
            solution: Point2::new(0., 0.),
            fix,
        })
    }
    /// `IsDone()`; `resolve.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `Solution()`: StdFail_NotDone unless done; `resolve.solution()?`.
    pub fn solution(&self) -> Result<Point2, Failure> {
        if !self.done {
            return Err(failure("StdFail_NotDone", "ProjLib_PrjResolve::Solution"));
        }
        Ok(self.solution)
    }

    /// `Perform(t, U, V, Tol, Inf, Sup, FTol, StrictInside)`: searches the root near
    /// the start values (the fixed one is taken from t, U or V) in the bounds grown
    /// by ten tolerances. `strict_inside` is unused, as natively; a negative
    /// `func_tol` skips the residual check of the fallback solver;
    /// `resolve.perform(0.5, 0.1, 0.2, bounds, 1e-10, false)?`.
    pub fn perform(
        &mut self,
        t: f64,
        u: f64,
        v: f64,
        bounds: PrjResolveBox,
        func_tol: f64,
        _strict_inside: bool,
    ) -> Result<(), Failure> {
        self.done = false;
        let (start, fix_val): ([f64; 2], f64) = match self.fix {
            1 => ([u, v], t),
            2 => ([t, v], u),
            _ => ([t, u], v),
        };
        let mut function: PrjFunc<'_> = PrjFunc::new(self.curve, fix_val, self.surface, self.fix)?;
        let Some(newton_done) = solve(&mut function, start, &bounds, func_tol)? else {
            return Ok(());
        };
        self.solution = function.solution();
        self.done = true;
        self.snap_to_bounds(&bounds);
        if self.outside_extra(&bounds) {
            self.done = false;
        } else if func_tol > 0. {
            self.check_residual(&mut function, newton_done, func_tol)?;
        }
        Ok(())
    }

    // A solution slightly outside a bound (by less than its tolerance) is put on it;
    // each test reads the solution as the previous one left it.
    fn snap_to_bounds(&mut self, b: &PrjResolveBox) {
        let x: f64 = self.solution.x();
        if x > b.inf.x() - b.tol.x() && x < b.inf.x() {
            self.solution.set_x(b.inf.x());
        }
        let x: f64 = self.solution.x();
        if x > b.sup.x() && x < b.sup.x() + b.tol.x() {
            self.solution.set_x(b.sup.x());
        }
        let y: f64 = self.solution.y();
        if y > b.inf.y() - b.tol.y() && y < b.inf.y() {
            self.solution.set_y(b.inf.y());
        }
        let y: f64 = self.solution.y();
        if y > b.sup.y() && y < b.sup.y() + b.tol.y() {
            self.solution.set_y(b.sup.y());
        }
    }

    // Whether the solution lies farther than two tolerances outside the bounds.
    fn outside_extra(&self, b: &PrjResolveBox) -> bool {
        let (extra_u, extra_v) = (2. * b.tol.x(), 2. * b.tol.y());
        let (x, y) = (self.solution.x(), self.solution.y());
        x < b.inf.x() - extra_u
            || x > b.sup.x() + extra_u
            || y < b.inf.y() - extra_v
            || y > b.sup.y() + extra_v
    }

    // A fallback solution is rejected when its squared residual exceeds FTol; the
    // evaluation at the solution is made even after a Newton success, as natively.
    fn check_residual(
        &mut self,
        function: &mut PrjFunc<'_>,
        newton_done: bool,
        func_tol: f64,
    ) -> Result<(), Failure> {
        let mut x: Vector<'static> = Vector::filled(1, 2, 0.)?;
        let mut f_val: Vector<'static> = Vector::filled(1, 2, 0.)?;
        x[1] = self.solution.x();
        x[2] = self.solution.y();
        function.value(&x, &mut f_val)?;
        if !newton_done && (f_val[1] * f_val[1] + f_val[2] * f_val[2]) > func_tol {
            self.done = false;
        }
        Ok(())
    }
}

// The Newton search, then the general one when it fails: `Some(newton_done)` when
// one of them is done, `None` when both fail.
fn solve(
    function: &mut PrjFunc<'_>,
    start: [f64; 2],
    b: &PrjResolveBox,
    func_tol: f64,
) -> Result<Option<bool>, Failure> {
    let (ext_u, ext_v) = (10. * b.tol.x(), 10. * b.tol.y());
    let tol: Vector<'static> = pair(b.tol.x(), b.tol.y())?;
    let start: Vector<'static> = pair(start[0], start[1])?;
    let inf: Vector<'static> = pair(b.inf.x() - ext_u, b.inf.y() - ext_v)?;
    let sup: Vector<'static> = pair(b.sup.x() + ext_u, b.sup.y() + ext_v)?;
    let mut newton = NewtonFunctionSetRoot::new(function, &tol, func_tol, NB_ITERATIONS)?;
    newton.perform_bounded(function, &start, &inf, &sup)?;
    function.take_failure()?;
    if newton.is_done() {
        return Ok(Some(true));
    }
    let mut general = FunctionSetRoot::new(function, &tol, NB_ITERATIONS)?;
    general.perform_bounded(function, &start, &inf, &sup, false)?;
    function.take_failure()?;
    Ok(general.is_done().then_some(false))
}

fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    vector[1] = first;
    vector[2] = second;
    Ok(vector)
}
