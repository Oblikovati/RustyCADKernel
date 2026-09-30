// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_AHTBezierSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_AHTBezierSurface`: a tensor-product algebraic-hyperbolic-trigonometric
//! Bezier surface over `{1, t, ..., t^k, sinh(a t), cosh(a t), sin(b t), cos(b t)}`
//! in U and V, on `[0, 1] x [0, 1]`.
mod axis;
mod evaluation;
mod polynomial;
mod surface_traits;
use super::bezier_grid::{PoleGrid, check_weight_grid, owned_grid};
use crate::geom::errors;
use axis::AxisShape;
use evaluation::AhtSurfaceView;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkg2d::geom2d_eval::eval_basis::aht::basis_dimension;
use rustycad_tkmath::gp::Point3;

const CLASS: &str = "GeomEval_AHTBezierSurface";

/// A rational or non-rational AHT-Bezier surface; immutable like every eval surface
/// (reversal, iso extraction and `Transform` raise Standard_NotImplemented);
/// `Handle::new(GeomEvalAHTBezierSurface::new(&poles, [2, 2], [0.0, 0.0], [0.0, 0.0])?)`.
pub struct GeomEvalAHTBezierSurface {
    parent: StandardTransient,
    poles: Array2<'static, Point3>,
    weights: Array2<'static, f64>,
    alg_degrees: [i32; 2],
    alphas: [f64; 2],
    betas: [f64; 2],
    rational: bool,
}
impl_transient!(
    GeomEvalAHTBezierSurface,
    "GeomEval_AHTBezierSurface",
    dyn crate::geom::GeomBoundedSurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomBoundedSurface
    ]
);
impl Clone for GeomEvalAHTBezierSurface {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// The checks both constructors share, in native order: degrees, alphas, betas, then
// the pole grid against the basis dimensions.
fn check_basis(
    poles: &Array2<'_, Point3>,
    [degree_u, degree_v]: [i32; 2],
    [alpha_u, alpha_v]: [f64; 2],
    [beta_u, beta_v]: [f64; 2],
) -> Result<(), Failure> {
    let refuse = |what: String| Err(errors::construction(&format!("{CLASS}: {what}")));
    if degree_u < 0 || degree_v < 0 {
        return refuse(format!(
            "algebraic degrees ({degree_u}, {degree_v}); expected both >= 0"
        ));
    }
    if alpha_u < 0.0 || alpha_v < 0.0 {
        return refuse(format!("alphas ({alpha_u}, {alpha_v}); expected both >= 0"));
    }
    if beta_u < 0.0 || beta_v < 0.0 {
        return refuse(format!("betas ({beta_u}, {beta_v}); expected both >= 0"));
    }
    let dim_u: i32 = basis_dimension(degree_u, alpha_u, beta_u);
    let dim_v: i32 = basis_dimension(degree_v, alpha_v, beta_v);
    let (rows, cols) = (poles.nb_rows(), poles.nb_columns());
    if rows as i64 != i64::from(dim_u) || cols as i64 != i64::from(dim_v) {
        return refuse(format!(
            "poles are {rows}x{cols}; expected the basis dimensions {dim_u}x{dim_v}"
        ));
    }
    Ok(())
}

impl GeomEvalAHTBezierSurface {
    /// Non-rational surface (`GeomEval_AHTBezierSurface(Poles, AlgDegreeU, AlgDegreeV,
    /// AlphaU, AlphaV, BetaU, BetaV)`), the parameters given as `[U, V]` pairs;
    /// Standard_ConstructionError for a negative degree, alpha or beta, or a pole grid
    /// that is not `basisDimension(U) x basisDimension(V)`.
    ///
    /// The native constructor stores `myWeights(1, 1, 1, 1)`, one uninitialized double;
    /// the port stores 0.0 there; `GeomEvalAHTBezierSurface::new(&poles, [2, 2], [0.0; 2], [0.0; 2])?`.
    pub fn new(
        poles: &Array2<'_, Point3>,
        alg_degrees: [i32; 2],
        alphas: [f64; 2],
        betas: [f64; 2],
    ) -> Result<Self, Failure> {
        check_basis(poles, alg_degrees, alphas, betas)?;
        let weights = Array2::with_bounds(1, 1, 1, 1).map_err(errors::array_bounds)?;
        let basis = (alg_degrees, alphas, betas);
        Ok(Self::stored(poles, weights, basis, false))
    }
    /// Rational surface (`GeomEval_AHTBezierSurface(Poles, Weights, AlgDegreeU, ...)`);
    /// also refuses a weight grid of another shape and weights <= 0;
    /// `GeomEvalAHTBezierSurface::with_weights(&poles, &weights, [2, 2], [0.0; 2], [0.0; 2])?`.
    pub fn with_weights(
        poles: &Array2<'_, Point3>,
        weights: &Array2<'_, f64>,
        alg_degrees: [i32; 2],
        alphas: [f64; 2],
        betas: [f64; 2],
    ) -> Result<Self, Failure> {
        check_basis(poles, alg_degrees, alphas, betas)?;
        check_weight_grid(CLASS, weights, poles)?;
        let basis = (alg_degrees, alphas, betas);
        Ok(Self::stored(poles, owned_grid(weights), basis, true))
    }
    fn stored(
        poles: &Array2<'_, Point3>,
        weights: Array2<'static, f64>,
        (alg_degrees, alphas, betas): ([i32; 2], [f64; 2], [f64; 2]),
        rational: bool,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            poles: owned_grid(poles),
            weights,
            alg_degrees,
            alphas,
            betas,
            rational,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        let basis = (self.alg_degrees, self.alphas, self.betas);
        Self::stored(&self.poles, self.weights.clone(), basis, self.rational)
    }
    /// A copy of the poles grid with its bounds (`Poles`); `surface.poles().nb_rows()`.
    pub fn poles(&self) -> Array2<'static, Point3> {
        self.poles.clone()
    }
    /// A copy of the weights grid (`Weights`); one element for a non-rational surface,
    /// whose native value is uninitialized; `surface.weights().len()`.
    pub fn weights(&self) -> Array2<'static, f64> {
        self.weights.clone()
    }
    /// The algebraic degree in U (`AlgDegreeU`); `surface.alg_degree_u()`.
    pub fn alg_degree_u(&self) -> i32 {
        self.alg_degrees[0]
    }
    /// The algebraic degree in V (`AlgDegreeV`); `surface.alg_degree_v()`.
    pub fn alg_degree_v(&self) -> i32 {
        self.alg_degrees[1]
    }
    /// The hyperbolic frequency in U, 0 without the pair (`AlphaU`); `surface.alpha_u()`.
    pub fn alpha_u(&self) -> f64 {
        self.alphas[0]
    }
    /// The hyperbolic frequency in V (`AlphaV`); `surface.alpha_v()`.
    pub fn alpha_v(&self) -> f64 {
        self.alphas[1]
    }
    /// The trigonometric frequency in U, 0 without the pair (`BetaU`); `surface.beta_u()`.
    pub fn beta_u(&self) -> f64 {
        self.betas[0]
    }
    /// The trigonometric frequency in V (`BetaV`); `surface.beta_v()`.
    pub fn beta_v(&self) -> f64 {
        self.betas[1]
    }
    /// The number of poles in U, the grid rows (`NbPolesU`); `surface.nb_poles_u()`.
    pub fn nb_poles_u(&self) -> i32 {
        self.poles.nb_rows() as i32
    }
    /// The number of poles in V, the grid columns (`NbPolesV`); `surface.nb_poles_v()`.
    pub fn nb_poles_v(&self) -> i32 {
        self.poles.nb_columns() as i32
    }
    /// Whether the surface is rational in U; both flags come from the constructor that
    /// was used (`IsURational`); `surface.is_u_rational()`.
    pub fn is_u_rational(&self) -> bool {
        self.rational
    }
    /// Whether the surface is rational in V (`IsVRational`); `surface.is_v_rational()`.
    pub fn is_v_rational(&self) -> bool {
        self.rational
    }
    fn view(&self) -> AhtSurfaceView<'_> {
        let axis = |k: usize| AxisShape {
            alg_degree: self.alg_degrees[k] as usize,
            alpha: self.alphas[k],
            beta: self.betas[k],
        };
        AhtSurfaceView {
            grid: PoleGrid {
                poles: self.poles.as_slice(),
                weights: self.rational.then(|| self.weights.as_slice()),
                rows: self.poles.nb_rows(),
                cols: self.poles.nb_columns(),
            },
            axes: [axis(0), axis(1)],
        }
    }
}
