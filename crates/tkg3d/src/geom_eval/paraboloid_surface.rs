// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_ParaboloidSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_ParaboloidSurface`: the paraboloid of revolution
//! `P(u, v) = O + v cos(u) XDir + v sin(u) YDir + v^2 / (4 F) ZDir` on
//! `[0, 2 Pi] x (-inf, +inf)`.
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
use std::f64::consts::{FRAC_PI_2, PI};
use std::io;

const CLASS: &str = "GeomEval_ParaboloidSurface";

#[derive(Clone, Copy)]
struct ParaboloidState {
    pos: Axis3,
    focal: f64,
}

/// A paraboloid of revolution about ZDir with its apex at the origin; `UReverse`,
/// `VReverse`, the isos and `Transform` raise Standard_NotImplemented;
/// `GeomEvalParaboloidSurface::new(Axis3::default(), 1.0)?`.
pub struct GeomEvalParaboloidSurface {
    parent: StandardTransient,
    state: SharedState<ParaboloidState>,
}
impl_transient!(
    GeomEvalParaboloidSurface,
    "GeomEval_ParaboloidSurface",
    dyn crate::geom::GeomElementarySurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomElementarySurface
    ]
);
// Copy() reruns the constructor on the stored, already valid values.
impl Clone for GeomEvalParaboloidSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomEvalParaboloidSurface {
    fn stored(state: ParaboloidState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Paraboloid of focal distance `focal`; Standard_ConstructionError unless > 0;
    /// `GeomEvalParaboloidSurface::new(frame, 1.0)?`.
    pub fn new(position: Axis3, focal: f64) -> Result<Self, Failure> {
        let focal: f64 = positive(focal, "GeomEval_ParaboloidSurface: focal distance")?;
        Ok(Self::stored(ParaboloidState {
            pos: position,
            focal,
        }))
    }
    /// The focal distance (`Focal`); `surface.focal()`.
    pub fn focal(&self) -> f64 {
        self.state.read().focal
    }
    /// Set the focal distance; Standard_ConstructionError unless > 0 (`SetFocal`);
    /// `surface.set_focal(2.0)?`.
    pub fn set_focal(&self, focal: f64) -> Result<(), Failure> {
        self.state.write().focal = positive(
            focal,
            "GeomEval_ParaboloidSurface::SetFocal: focal distance",
        )?;
        Ok(())
    }
    /// The implicit equation `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` in the absolute
    /// frame, from `X^2 + Y^2 - 4 F Z = 0` (`Coefficients`); `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: ParaboloidState = *self.state.read();
        let [
            [t11, t12, t13, t14],
            [t21, t22, t23, t24],
            [t31, t32, t33, t34],
        ] = frame_rows(state.pos);
        let f: f64 = state.focal;
        [
            t11 * t11 + t21 * t21,
            t12 * t12 + t22 * t22,
            t13 * t13 + t23 * t23,
            t11 * t12 + t21 * t22,
            t11 * t13 + t21 * t23,
            t12 * t13 + t22 * t23,
            t11 * t14 + t21 * t24 - 2.0 * f * t31,
            t12 * t14 + t22 * t24 - 2.0 * f * t32,
            t13 * t14 + t23 * t24 - 2.0 * f * t33,
            t14 * t14 + t24 * t24 - 4.0 * f * t34,
        ]
    }
}

/// The paraboloid formulas at one `(U, V)`. The native code sums coordinate by
/// coordinate (`anO.X() + V * aCosU * aXDir.X() + ...`), which is the same
/// arithmetic as the vector sums here.
pub(crate) struct ParaboloidPatch {
    frame: EvalFrame,
    focal: f64,
    v: f64,
    cos_u: f64,
    sin_u: f64,
}
impl PatchDerivatives for ParaboloidPatch {
    fn point(&self) -> Xyz {
        let (f, v) = (&self.frame, self.v);
        let z_comp: f64 = v * v / (4.0 * self.focal);
        f.origin + f.x * (v * self.cos_u) + f.y * (v * self.sin_u) + f.z * z_comp
    }
    fn first(&self) -> [Xyz; 2] {
        let (f, v, cu, su) = (&self.frame, self.v, self.cos_u, self.sin_u);
        let v_over_2f: f64 = v / (2.0 * self.focal);
        [
            f.x * (-v * su) + f.y * (v * cu),
            f.x * cu + f.y * su + f.z * v_over_2f,
        ]
    }
    fn second(&self) -> [Xyz; 3] {
        let (f, v, cu, su) = (&self.frame, self.v, self.cos_u, self.sin_u);
        let inv_2f: f64 = 1.0 / (2.0 * self.focal);
        [
            f.x * (-v * cu) - f.y * (v * su),
            f.z * inv_2f,
            f.x * (-su) + f.y * cu,
        ]
    }
    fn third(&self) -> [Xyz; 4] {
        let (f, v, cu, su) = (&self.frame, self.v, self.cos_u, self.sin_u);
        [
            f.x * (v * su) - f.y * (v * cu),
            Xyz::default(),
            f.x * (-cu) - f.y * su,
            Xyz::default(),
        ]
    }
}

// The V derivative of order `nv` of the Z term `v^2 / (4F)`, present only when Nu == 0.
fn z_derivative(v: f64, focal: f64, nv: i32) -> f64 {
    match nv {
        0 => v * v / (4.0 * focal),
        1 => v / (2.0 * focal),
        2 => 1.0 / (2.0 * focal),
        _ => 0.0,
    }
}

impl EvalElementarySurface for GeomEvalParaboloidSurface {
    const CLASS: &'static str = CLASS;
    const BOUNDS: (f64, f64, f64, f64) =
        (0.0, 2.0 * PI, -precision::infinite(), precision::infinite());
    const U_CLOSED: bool = true;
    type Patch = ParaboloidPatch;
    fn patch(&self, u: f64, v: f64) -> ParaboloidPatch {
        let state: ParaboloidState = *self.state.read();
        ParaboloidPatch {
            frame: EvalFrame::new(state.pos),
            focal: state.focal,
            v,
            cos_u: u.cos(),
            sin_u: u.sin(),
        }
    }
    // XD and YD carry `d^Nu cos/sin(u) * d^Nv v`, ZD `d^Nv (v^2 / 4F)` when Nu == 0.
    fn derivative_n(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
        let state: ParaboloidState = *self.state.read();
        let frame: EvalFrame = EvalFrame::new(state.pos);
        let v_derivative: f64 = match nv {
            0 => v,
            1 => 1.0,
            _ => 0.0,
        };
        let phase: f64 = u + f64::from(nu) * FRAC_PI_2;
        let x_coeff: f64 = phase.cos() * v_derivative;
        let y_coeff: f64 = phase.sin() * v_derivative;
        let z_coeff: f64 = if nu == 0 {
            z_derivative(v, state.focal, nv)
        } else {
            0.0
        };
        Vector3::from(frame.x * x_coeff + frame.y * y_coeff + frame.z * z_coeff)
    }
    fn dump_parameters(&self, stream: &mut DumpStream) -> io::Result<()> {
        dump::dump_real(stream, "Focal", self.state.read().focal)
    }
}
impl_eval_elementary_surface!(GeomEvalParaboloidSurface);
