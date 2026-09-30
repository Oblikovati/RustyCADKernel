// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_HypParaboloidSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_HypParaboloidSurface`: the hyperbolic paraboloid (saddle)
//! `P(u, v) = O + u XDir + v YDir + (u^2 / A^2 - v^2 / B^2) ZDir` on the whole plane.
use super::elementary_eval::{
    EvalElementarySurface, EvalFrame, PatchDerivatives, impl_eval_elementary_surface, positive,
};
use crate::geom::quadric::frame_rows;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::{Axis3, Vector3, Xyz};
use std::io;

const CLASS: &str = "GeomEval_HypParaboloidSurface";

#[derive(Clone, Copy)]
struct HypParaboloidState {
    pos: Axis3,
    a: f64,
    b: f64,
}

/// A hyperbolic paraboloid; `UReverse`, `VReverse`, the isos and `Transform` raise
/// Standard_NotImplemented; `GeomEvalHypParaboloidSurface::new(Axis3::default(), 2.0, 3.0)?`.
pub struct GeomEvalHypParaboloidSurface {
    parent: StandardTransient,
    state: SharedState<HypParaboloidState>,
}
impl_transient!(
    GeomEvalHypParaboloidSurface,
    "GeomEval_HypParaboloidSurface",
    dyn crate::geom::GeomElementarySurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomElementarySurface
    ]
);
// Copy() reruns the constructor on the stored, already valid values.
impl Clone for GeomEvalHypParaboloidSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomEvalHypParaboloidSurface {
    fn stored(state: HypParaboloidState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Saddle of semi-axes `a` (along XDir) and `b` (along YDir);
    /// Standard_ConstructionError unless both are > 0;
    /// `GeomEvalHypParaboloidSurface::new(frame, 2.0, 3.0)?`.
    pub fn new(position: Axis3, a: f64, b: f64) -> Result<Self, Failure> {
        let a: f64 = positive(a, "GeomEval_HypParaboloidSurface: semi-axis A")?;
        let b: f64 = positive(b, "GeomEval_HypParaboloidSurface: semi-axis B")?;
        Ok(Self::stored(HypParaboloidState {
            pos: position,
            a,
            b,
        }))
    }
    /// The semi-axis A (`SemiAxisA`); `surface.semi_axis_a()`.
    pub fn semi_axis_a(&self) -> f64 {
        self.state.read().a
    }
    /// The semi-axis B (`SemiAxisB`); `surface.semi_axis_b()`.
    pub fn semi_axis_b(&self) -> f64 {
        self.state.read().b
    }
    /// Set A; Standard_ConstructionError unless > 0 (`SetSemiAxisA`);
    /// `surface.set_semi_axis_a(1.5)?`.
    pub fn set_semi_axis_a(&self, a: f64) -> Result<(), Failure> {
        self.state.write().a = positive(a, "GeomEval_HypParaboloidSurface::SetSemiAxisA: A")?;
        Ok(())
    }
    /// Set B; Standard_ConstructionError unless > 0 (`SetSemiAxisB`);
    /// `surface.set_semi_axis_b(1.5)?`.
    pub fn set_semi_axis_b(&self, b: f64) -> Result<(), Failure> {
        self.state.write().b = positive(b, "GeomEval_HypParaboloidSurface::SetSemiAxisB: B")?;
        Ok(())
    }
    /// The implicit equation `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` in the absolute
    /// frame, from `X^2/A^2 - Y^2/B^2 - Z = 0` (`Coefficients`); `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: HypParaboloidState = *self.state.read();
        let [r1, r2, r3] = frame_rows(state.pos);
        let (a2, b2) = (state.a * state.a, state.b * state.b);
        // `Ti * Tj / A^2 - T'i * T'j / B^2` for the columns (i, j) of a coefficient.
        let quadratic = |i: usize, j: usize| r1[i] * r1[j] / a2 - r2[i] * r2[j] / b2;
        [
            quadratic(0, 0),
            quadratic(1, 1),
            quadratic(2, 2),
            quadratic(0, 1),
            quadratic(0, 2),
            quadratic(1, 2),
            quadratic(0, 3) - 0.5 * r3[0],
            quadratic(1, 3) - 0.5 * r3[1],
            quadratic(2, 3) - 0.5 * r3[2],
            quadratic(3, 3) - r3[3],
        ]
    }
}

/// The saddle formulas at one `(U, V)`; like the paraboloid, the native code sums
/// coordinate by coordinate, which is the same arithmetic as these vector sums.
pub(crate) struct HypParaboloidPatch {
    frame: EvalFrame,
    a2: f64,
    b2: f64,
    u: f64,
    v: f64,
}
impl PatchDerivatives for HypParaboloidPatch {
    fn point(&self) -> Xyz {
        let (f, u, v) = (&self.frame, self.u, self.v);
        let z_comp: f64 = u * u / self.a2 - v * v / self.b2;
        f.origin + f.x * u + f.y * v + f.z * z_comp
    }
    fn first(&self) -> [Xyz; 2] {
        let f: &EvalFrame = &self.frame;
        let z_du: f64 = 2.0 * self.u / self.a2;
        let z_dv: f64 = -2.0 * self.v / self.b2;
        [f.x + f.z * z_du, f.y + f.z * z_dv]
    }
    fn second(&self) -> [Xyz; 3] {
        let f: &EvalFrame = &self.frame;
        [
            f.z * (2.0 / self.a2),
            f.z * (-2.0 / self.b2),
            Xyz::default(),
        ]
    }
    // A polynomial of degree 2: every third derivative is zero.
    fn third(&self) -> [Xyz; 4] {
        [Xyz::default(); 4]
    }
}

// `d^Nu (u^2 / A^2)` when Nv == 0, else 0 (a Nu >= 3 derivative adds nothing).
fn u_part(u: f64, a2: f64, nu: i32) -> Option<f64> {
    match nu {
        0 => Some(u * u / a2),
        1 => Some(2.0 * u / a2),
        2 => Some(2.0 / a2),
        _ => None,
    }
}

// `d^Nv (-v^2 / B^2)` when Nu == 0.
fn v_part(v: f64, b2: f64, nv: i32) -> Option<f64> {
    match nv {
        0 => Some(-v * v / b2),
        1 => Some(-2.0 * v / b2),
        2 => Some(-2.0 / b2),
        _ => None,
    }
}

// The ZD coefficient, accumulated from 0.0 with `+=` as natively (so a lone -0.0
// term becomes +0.0).
fn z_coefficient(state: &HypParaboloidState, u: f64, v: f64, nu: i32, nv: i32) -> f64 {
    let (a2, b2) = (state.a * state.a, state.b * state.b);
    let mut z_coeff: f64 = 0.0;
    if let Some(term) = u_part(u, a2, nu).filter(|_| nv == 0) {
        z_coeff += term;
    }
    if let Some(term) = v_part(v, b2, nv).filter(|_| nu == 0) {
        z_coeff += term;
    }
    z_coeff
}

// `d^n t / dt^n` of a linear parameter: t, 1, then 0.
fn linear_derivative(t: f64, n: i32) -> f64 {
    match n {
        0 => t,
        1 => 1.0,
        _ => 0.0,
    }
}

impl EvalElementarySurface for GeomEvalHypParaboloidSurface {
    const CLASS: &'static str = CLASS;
    const BOUNDS: (f64, f64, f64, f64) = (
        -precision::infinite(),
        precision::infinite(),
        -precision::infinite(),
        precision::infinite(),
    );
    const U_CLOSED: bool = false;
    type Patch = HypParaboloidPatch;
    fn patch(&self, u: f64, v: f64) -> HypParaboloidPatch {
        let state: HypParaboloidState = *self.state.read();
        HypParaboloidPatch {
            frame: EvalFrame::new(state.pos),
            a2: state.a * state.a,
            b2: state.b * state.b,
            u,
            v,
        }
    }
    fn derivative_n(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
        let state: HypParaboloidState = *self.state.read();
        let frame: EvalFrame = EvalFrame::new(state.pos);
        let x_coeff: f64 = if nv == 0 {
            linear_derivative(u, nu)
        } else {
            0.0
        };
        let y_coeff: f64 = if nu == 0 {
            linear_derivative(v, nv)
        } else {
            0.0
        };
        let z_coeff: f64 = z_coefficient(&state, u, v, nu, nv);
        Vector3::from(frame.x * x_coeff + frame.y * y_coeff + frame.z * z_coeff)
    }
    fn dump_parameters(&self, stream: &mut DumpStream) -> io::Result<()> {
        let state: HypParaboloidState = *self.state.read();
        dump::dump_real(stream, "A", state.a)?;
        dump::dump_real(stream, "B", state.b)
    }
}
impl_eval_elementary_surface!(GeomEvalHypParaboloidSurface);
