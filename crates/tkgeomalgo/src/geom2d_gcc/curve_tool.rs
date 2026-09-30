// Rust translation of Geom2dGcc_CurveTool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_CurveTool`: the static curve access of the Geom2dGcc solvers.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dD3};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `Geom2dGcc_CurveTool`: static members forwarding to the adaptor, with the fixed
/// sample count 20 of the Geom2dGcc solvers. The adaptor's exceptions are returned
/// as `Err`; `CurveTool::nb_samples(&adaptor)`.
pub struct CurveTool;

impl CurveTool {
    /// `FirstParameter(C)`. `CurveTool::first_parameter(&adaptor)?`.
    pub fn first_parameter(c: &Geom2dAdaptorCurve) -> Result<f64, Failure> {
        c.first_parameter()
    }
    /// `LastParameter(C)`. `CurveTool::last_parameter(&adaptor)?`.
    pub fn last_parameter(c: &Geom2dAdaptorCurve) -> Result<f64, Failure> {
        c.last_parameter()
    }
    /// `EpsX(C, Tol)`: the parametric resolution of `tol`.
    /// `CurveTool::eps_x(&adaptor, 1e-7)?`.
    pub fn eps_x(c: &Geom2dAdaptorCurve, tol: f64) -> Result<f64, Failure> {
        c.resolution(tol)
    }
    /// `NbSamples(C)`: 20 whatever the curve (the native "template a respecter").
    /// `CurveTool::nb_samples(&adaptor)`.
    pub fn nb_samples(_c: &Geom2dAdaptorCurve) -> i32 {
        20
    }
    /// `Value(C, X)`. `CurveTool::value(&adaptor, 0.5)?`.
    pub fn value(c: &Geom2dAdaptorCurve, x: f64) -> Result<Point2, Failure> {
        c.value(x)
    }
    /// `D1(C, U, P, T)`. `let (p, t) = CurveTool::d1(&adaptor, 0.5)?;`.
    pub fn d1(c: &Geom2dAdaptorCurve, u: f64) -> Result<(Point2, Vector2), Failure> {
        c.d1(u)
    }
    /// `D2(C, U, P, T, N)`. `let (p, t, n) = CurveTool::d2(&adaptor, 0.5)?;`.
    pub fn d2(c: &Geom2dAdaptorCurve, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        c.d2(u)
    }
    /// `D3(C, U, P, T, N, dN)`. `let (p, t, n, dn) = CurveTool::d3(&adaptor, 0.5)?;`.
    pub fn d3(c: &Geom2dAdaptorCurve, u: f64) -> Result<Adaptor2dD3, Failure> {
        c.d3(u)
    }
}
