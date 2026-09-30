// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SweepApproximation.hxx/.lxx/.cxx (fields, accessors, Dump)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_SweepApproximation`: the B-spline surface (and 2D curves) of a sweep,
//! approximating each pole of its B-spline section along the path.
mod accessors;
mod cache;
mod perform;

pub use accessors::{SweepCurve2d, SweepSurface, SweepSurfaceShape};

use super::sweep_function::SweepFunction;
use crate::native_failure::failure;
use cache::SweepCache;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3};

/// The arguments of `Perform` after the range, with the native defaults
/// `Continuity = GeomAbs_C0`, `Degmax = 11`, `Segmax = 50`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweepOptions {
    /// `Tol3d`.
    pub tol_3d: f64,
    /// `BoundTol`.
    pub bound_tol: f64,
    /// `Tol2d`.
    pub tol_2d: f64,
    /// `TolAngular`.
    pub tol_angular: f64,
    /// `Continuity`.
    pub continuity: Shape,
    /// `Degmax`.
    pub degmax: i32,
    /// `Segmax`.
    pub segmax: i32,
}

impl SweepOptions {
    /// The tolerances with the default continuity, degree and segments;
    /// `SweepOptions::new(1e-4, 1e-4, 1e-5, 1e-2)`.
    pub fn new(tol_3d: f64, bound_tol: f64, tol_2d: f64, tol_angular: f64) -> Self {
        Self {
            tol_3d,
            bound_tol,
            tol_2d,
            tol_angular,
            continuity: Shape::C0,
            degmax: 11,
            segmax: 50,
        }
    }
}

/// `Approx_SweepApproximation`: after a Perform with a result, the surface poles,
/// weights, knots and multiplicities, the 2D curves and the errors. The surface
/// accessors raise StdFail_NotDone without a result, and the 2D ones
/// Standard_DomainError without 2D curves.
///
/// ```ignore
/// let mut approx = SweepApproximation::new(&function);
/// approx.perform(0.0, 1.0, SweepOptions::new(1e-4, 1e-4, 1e-5, 1e-2))?;
/// let poles = approx.surf_poles()?;
/// ```
pub struct SweepApproximation {
    cache: SweepCache,
    done: bool,
    udeg: i32,
    vdeg: i32,
    deg2d: i32,
    tab_poles: Option<Array2<'static, Point3>>,
    tab_weights: Option<Array2<'static, f64>>,
    tab_u_knots: Option<Array1<'static, f64>>,
    tab_v_knots: Option<Array1<'static, f64>>,
    tab_2d_knots: Option<Array1<'static, f64>>,
    tab_u_mults: Option<Array1<'static, i32>>,
    tab_v_mults: Option<Array1<'static, i32>>,
    tab_2d_mults: Option<Array1<'static, i32>>,
    seq_poles_2d: Vec<Array1<'static, Point2>>,
    m_error_1d: Option<Array1<'static, f64>>,
    tab_2d_error: Option<Array1<'static, f64>>,
    m_error_3d: Option<Array1<'static, f64>>,
    a_error_1d: Option<Array1<'static, f64>>,
    ave_2d_error: Option<Array1<'static, f64>>,
    a_error_3d: Option<Array1<'static, f64>>,
    con_surf_err: Option<Array1<'static, f64>>,
}

/// The Standard_NullObject of a table the native code reads through a null handle.
pub(crate) fn null_table(what: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("Approx_SweepApproximation: null {what}"),
    )
}

fn func_ref(func: &Handle<dyn SweepFunction>) -> Result<&dyn SweepFunction, Failure> {
    func.get().ok_or_else(|| null_table("sweep function"))
}

impl SweepApproximation {
    /// `Approx_SweepApproximation(Func)`: no evaluation cached yet;
    /// `SweepApproximation::new(&function)`.
    pub fn new(func: &Handle<dyn SweepFunction>) -> Self {
        Self {
            cache: SweepCache::new(func.clone()),
            done: false,
            udeg: 0,
            vdeg: 0,
            deg2d: 0,
            tab_poles: None,
            tab_weights: None,
            tab_u_knots: None,
            tab_v_knots: None,
            tab_2d_knots: None,
            tab_u_mults: None,
            tab_v_mults: None,
            tab_2d_mults: None,
            seq_poles_2d: Vec::new(),
            m_error_1d: None,
            tab_2d_error: None,
            m_error_3d: None,
            a_error_1d: None,
            ave_2d_error: None,
            a_error_3d: None,
            con_surf_err: None,
        }
    }

    /// `Eval(Parameter, DerivativeRequest, First, Last, Result)`: the values AdvApprox
    /// reads (the weights, the transformed 2D poles, then the weighted 3D poles, or
    /// their derivatives) into `result`; returns 0, 1 when the function fails, 2 for
    /// an order beyond 2; `approx.eval(0.5, 0, 0.0, 1.0, &mut values)?`.
    pub fn eval(
        &mut self,
        parameter: f64,
        derivative_request: i32,
        first: f64,
        last: f64,
        result: &mut [f64],
    ) -> Result<i32, Failure> {
        self.cache
            .eval(parameter, derivative_request, [first, last], result)
    }

    /// `IsDone()`; `approx.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    fn check_done(&self) -> Result<(), Failure> {
        if !self.done {
            return Err(failure("StdFail_NotDone", "Approx_SweepApproximation"));
        }
        Ok(())
    }

    fn check_curves_2d(&self) -> Result<(), Failure> {
        self.check_done()?;
        if self.seq_poles_2d.is_empty() {
            return Err(failure("Standard_DomainError", "Approx_SweepApproximation"));
        }
        Ok(())
    }
}
