// Rust translation of BRepBlend_AppFuncRoot.hxx/.cxx, BRepBlend_AppFunc.hxx/.cxx,
// BRepBlend_AppFuncRst.hxx/.cxx and BRepBlend_AppFuncRstRst.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sweep functions of the blend approximations: the section of a blend function at
//! any parameter of a blend line, found from the computed points by a Newton solve.
mod search;
mod sweep;

pub use sweep::{BRepBlendAppFunc, BRepBlendAppFuncRst, BRepBlendAppFuncRstRst};

use super::BRepBlendLineHandle;
use crate::blend::{BlendAppFunction, BlendPoint};
use crate::blend_func::linalg::{vector, vget, vset};
use crate::chfi_ds::{Shared, lock};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkgeombase::approx::SweepFunction;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, Matrix, Vector,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// A blend function shared between its caller and the sweep function that evaluates it
/// (natively a reference kept by `BRepBlend_AppFuncRoot`).
pub type SharedAppFunction = Shared<dyn BlendAppFunction>;

/// The kind of the points of the line: on two surfaces (`BRepBlend_AppFunc`), on a
/// surface and a restriction (`BRepBlend_AppFuncRst`), on two restrictions
/// (`BRepBlend_AppFuncRstRst`). It selects the `Point` and `Vec` members.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LineKind {
    Surfaces,
    SurfaceRestriction,
    Restrictions,
}

/// A blend function (or an inverse one) behind a reference, as a sized math function set
/// for the solver: `DynFunctionSet(&mut *function)`.
pub(crate) struct DynFunctionSet<'a, T: ?Sized>(pub(crate) &'a mut T);

impl<T: FunctionSetWithDerivatives + ?Sized> FunctionSet for DynFunctionSet<'_, T> {
    fn variables(&self) -> i32 {
        self.0.variables()
    }
    fn equations(&self) -> i32 {
        self.0.equations()
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        self.0.value(x, f)
    }
}

impl<T: FunctionSetWithDerivatives + ?Sized> FunctionSetWithDerivatives for DynFunctionSet<'_, T> {
    fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> bool {
        self.0.derivatives(x, d)
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut Vector<'_>, d: &mut Matrix<'_>) -> bool {
        self.0.values(x, f, d)
    }
}

/// The evaluation state of `BRepBlend_AppFuncRoot` (`myTolerance`, `myPnt`, `X1`, `X2`,
/// `XInit`, `Sol`, `mySolver`).
pub(crate) struct AppFuncState {
    tolerance: Vector<'static>,
    pnt: BlendPoint,
    x1: Vector<'static>,
    x2: Vector<'static>,
    xinit: Vector<'static>,
    sol: Vector<'static>,
    solver: FunctionSetRoot,
}

/// `BRepBlend_AppFuncRoot`: the line, the blend function, the barycentre of the line
/// points (rational sections) and the Newton solver of 30 iterations.
pub struct BRepBlendAppFuncRoot {
    parent: StandardTransient,
    line: BRepBlendLineHandle,
    func: SharedAppFunction,
    kind: LineKind,
    bary: Point3,
    state: Mutex<AppFuncState>,
}
impl_transient!(
    BRepBlendAppFuncRoot,
    "BRepBlend_AppFuncRoot",
    dyn SweepFunction,
    parent,
    interfaces: [dyn SweepFunction]
);

/// `myTolerance`: the function tolerances at `tol3d`, at most `tol2d`.
fn clamped_tolerances(
    func: &dyn BlendAppFunction,
    tol3d: f64,
    tol2d: f64,
) -> Result<Vector<'static>, Failure> {
    let mut tolerance = vector(func.variables())?;
    func.get_tolerance(&mut tolerance, tol3d)?;
    for ii in 1..=func.variables() {
        if vget(&tolerance, ii)? > tol2d {
            vset(&mut tolerance, ii, tol2d)?;
        }
    }
    Ok(tolerance)
}

/// The centre of the bounding box of the points of the line on both supports (rational
/// sections), the origin otherwise.
fn barycentre(func: &dyn BlendAppFunction, line: &BRepBlendLineHandle) -> Result<Point3, Failure> {
    if !func.is_rational()? {
        return Ok(Point3::default());
    }
    let (mut max, mut min) = ([-1.0e100; 3], [1.0e100; 3]);
    let mut bary = Point3::default();
    let line = lock(line);
    for ii in 1..=line.nb_points() {
        let p = line.point(ii)?;
        let (p1, p2) = (p.point_on_s1(), p.point_on_s2());
        for (k, (a, b)) in [(p1.x(), p2.x()), (p1.y(), p2.y()), (p1.z(), p2.z())]
            .into_iter()
            .enumerate()
        {
            max[k] = std_max(std_max(a, b), max[k]);
            min[k] = std_min(std_min(a, b), min[k]);
        }
        bary = Point3::new(
            (max[0] + min[0]) / 2.,
            (max[1] + min[1]) / 2.,
            (max[2] + min[2]) / 2.,
        );
    }
    Ok(bary)
}

impl BRepBlendAppFuncRoot {
    /// `BRepBlend_AppFuncRoot(Line, Func, Tol3d, Tol2d)`: the tolerances, the section shape
    /// (which fixes the conversion of the function), the barycentre and the solver.
    pub(crate) fn new(
        line: &BRepBlendLineHandle,
        func: &SharedAppFunction,
        (tol3d, tol2d): (f64, f64),
        kind: LineKind,
    ) -> Result<Self, Failure> {
        let mut guard = lock(func);
        let tolerance = clamped_tolerances(&*guard, tol3d, tol2d)?;
        guard.get_shape()?;
        let bary = barycentre(&*guard, line)?;
        let dim = guard.variables();
        let solver = FunctionSetRoot::new(&DynFunctionSet(&mut *guard), &tolerance, 30)?;
        drop(guard);
        let state = AppFuncState {
            tolerance,
            pnt: BlendPoint::default(),
            x1: vector(dim)?,
            x2: vector(dim)?,
            xinit: vector(dim)?,
            sol: vector(dim)?,
            solver,
        };
        Ok(Self {
            parent: StandardTransient::default(),
            line: line.clone(),
            func: func.clone(),
            kind,
            bary,
            state: Mutex::new(state),
        })
    }

    fn state(&self) -> MutexGuard<'_, AppFuncState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The line of the function.
    pub fn line(&self) -> &BRepBlendLineHandle {
        &self.line
    }

    /// `Point(Func, Param, Sol, Pnt)`: the line point of the solution `sol` at `param`,
    /// for the line kind of the class; `root.point(&func, t, &sol, &mut pnt)?`.
    pub fn point(
        &self,
        func: &dyn BlendAppFunction,
        param: f64,
        sol: &Vector<'_>,
        pnt: &mut BlendPoint,
    ) -> Result<(), Failure> {
        self.kind.point(func, param, sol, pnt)
    }

    /// `Vec(Sol, Pnt)`: the variables of the function at the line point `pnt`;
    /// `root.vec(&mut sol, &pnt)?`.
    pub fn vec(&self, sol: &mut Vector<'_>, pnt: &BlendPoint) -> Result<(), Failure> {
        self.kind.vec(sol, pnt)
    }
}
