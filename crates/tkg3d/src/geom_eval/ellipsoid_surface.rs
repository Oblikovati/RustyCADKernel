// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_EllipsoidSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_EllipsoidSurface`:
//! `P(u, v) = O + A cos(v) cos(u) XDir + B cos(v) sin(u) YDir + C sin(v) ZDir`
//! on `[0, 2 Pi] x [-Pi/2, Pi/2]`.
use super::elementary_eval::{
    EvalElementarySurface, EvalFrame, PatchDerivatives, diagonal_quadric_coefficients,
    impl_eval_elementary_surface, positive,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::{Axis3, Vector3, Xyz};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "GeomEval_EllipsoidSurface";

#[derive(Clone, Copy)]
struct EllipsoidState {
    pos: Axis3,
    a: f64,
    b: f64,
    c: f64,
}

/// A triaxial ellipsoid centered on the origin of its frame; `UReverse`, `VReverse`,
/// the isos and `Transform` raise Standard_NotImplemented;
/// `Handle::new(GeomEvalEllipsoidSurface::new(Axis3::default(), 3.0, 2.0, 1.0)?)`.
pub struct GeomEvalEllipsoidSurface {
    parent: StandardTransient,
    state: SharedState<EllipsoidState>,
}
impl_transient!(
    GeomEvalEllipsoidSurface,
    "GeomEval_EllipsoidSurface",
    dyn crate::geom::GeomElementarySurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomElementarySurface
    ]
);
// Copy() reruns the constructor on the stored, already valid values.
impl Clone for GeomEvalEllipsoidSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomEvalEllipsoidSurface {
    fn stored(state: EllipsoidState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Ellipsoid of semi-axes `a` (XDir), `b` (YDir), `c` (ZDir);
    /// Standard_ConstructionError unless all three are > 0;
    /// `GeomEvalEllipsoidSurface::new(frame, 3.0, 2.0, 1.0)?`.
    pub fn new(position: Axis3, a: f64, b: f64, c: f64) -> Result<Self, Failure> {
        let a: f64 = positive(a, "GeomEval_EllipsoidSurface: semi-axis A")?;
        let b: f64 = positive(b, "GeomEval_EllipsoidSurface: semi-axis B")?;
        let c: f64 = positive(c, "GeomEval_EllipsoidSurface: semi-axis C")?;
        Ok(Self::stored(EllipsoidState {
            pos: position,
            a,
            b,
            c,
        }))
    }
    /// The semi-axis along XDir (`SemiAxisA`); `surface.semi_axis_a()`.
    pub fn semi_axis_a(&self) -> f64 {
        self.state.read().a
    }
    /// The semi-axis along YDir (`SemiAxisB`); `surface.semi_axis_b()`.
    pub fn semi_axis_b(&self) -> f64 {
        self.state.read().b
    }
    /// The semi-axis along ZDir (`SemiAxisC`); `surface.semi_axis_c()`.
    pub fn semi_axis_c(&self) -> f64 {
        self.state.read().c
    }
    /// Set A; Standard_ConstructionError unless > 0 (`SetSemiAxisA`);
    /// `surface.set_semi_axis_a(4.0)?`.
    pub fn set_semi_axis_a(&self, a: f64) -> Result<(), Failure> {
        self.state.write().a = positive(a, "GeomEval_EllipsoidSurface::SetSemiAxisA: A")?;
        Ok(())
    }
    /// Set B; Standard_ConstructionError unless > 0 (`SetSemiAxisB`);
    /// `surface.set_semi_axis_b(4.0)?`.
    pub fn set_semi_axis_b(&self, b: f64) -> Result<(), Failure> {
        self.state.write().b = positive(b, "GeomEval_EllipsoidSurface::SetSemiAxisB: B")?;
        Ok(())
    }
    /// Set C; Standard_ConstructionError unless > 0 (`SetSemiAxisC`);
    /// `surface.set_semi_axis_c(4.0)?`.
    pub fn set_semi_axis_c(&self, c: f64) -> Result<(), Failure> {
        self.state.write().c = positive(c, "GeomEval_EllipsoidSurface::SetSemiAxisC: C")?;
        Ok(())
    }
    /// The implicit equation `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` in the absolute
    /// frame, from `X^2/A^2 + Y^2/B^2 + Z^2/C^2 - 1 = 0` (`Coefficients`);
    /// `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: EllipsoidState = *self.state.read();
        let inverse_squares: [f64; 3] = [state.a, state.b, state.c].map(|r| 1.0 / (r * r));
        diagonal_quadric_coefficients(state.pos, inverse_squares, -1.0)
    }
}

/// The ellipsoid formulas at one `(U, V)`.
pub(crate) struct EllipsoidPatch {
    frame: EvalFrame,
    axes: [f64; 3],
    cos_u: f64,
    sin_u: f64,
    cos_v: f64,
    sin_v: f64,
}
impl EllipsoidPatch {
    // `A * a1 * a2 * XD + B * b1 * b2 * YD (+ C * c1 * ZD)`, products left to right.
    fn combine(&self, [a1, a2]: [f64; 2], [b1, b2]: [f64; 2], z: Option<f64>) -> Xyz {
        let [a, b, c] = self.axes;
        let f: &EvalFrame = &self.frame;
        let planar: Xyz = f.x * (a * a1 * a2) + f.y * (b * b1 * b2);
        match z {
            Some(c1) => planar + f.z * (c * c1),
            None => planar,
        }
    }
}
impl PatchDerivatives for EllipsoidPatch {
    fn point(&self) -> Xyz {
        let (cu, su, cv, sv) = (self.cos_u, self.sin_u, self.cos_v, self.sin_v);
        let [a, b, c] = self.axes;
        let f: &EvalFrame = &self.frame;
        f.origin + f.x * (a * cv * cu) + f.y * (b * cv * su) + f.z * (c * sv)
    }
    fn first(&self) -> [Xyz; 2] {
        let (cu, su, cv, sv) = (self.cos_u, self.sin_u, self.cos_v, self.sin_v);
        [
            self.combine([cv, -su], [cv, cu], None),
            self.combine([-sv, cu], [-sv, su], Some(cv)),
        ]
    }
    fn second(&self) -> [Xyz; 3] {
        let (cu, su, cv, sv) = (self.cos_u, self.sin_u, self.cos_v, self.sin_v);
        [
            self.combine([cv, -cu], [cv, -su], None),
            self.combine([-cv, cu], [-cv, su], Some(-sv)),
            self.combine([sv, su], [-sv, cu], None),
        ]
    }
    fn third(&self) -> [Xyz; 4] {
        let (cu, su, cv, sv) = (self.cos_u, self.sin_u, self.cos_v, self.sin_v);
        [
            self.combine([cv, su], [cv, -cu], None),
            self.combine([sv, cu], [sv, su], Some(-cv)),
            self.combine([sv, cu], [sv, su], None),
            self.combine([cv, su], [-cv, cu], None),
        ]
    }
}

impl EvalElementarySurface for GeomEvalEllipsoidSurface {
    const CLASS: &'static str = CLASS;
    const BOUNDS: (f64, f64, f64, f64) = (0.0, 2.0 * PI, -PI / 2.0, PI / 2.0);
    const U_CLOSED: bool = true;
    type Patch = EllipsoidPatch;
    fn patch(&self, u: f64, v: f64) -> EllipsoidPatch {
        let state: EllipsoidState = *self.state.read();
        EllipsoidPatch {
            frame: EvalFrame::new(state.pos),
            axes: [state.a, state.b, state.c],
            cos_u: u.cos(),
            sin_u: u.sin(),
            cos_v: v.cos(),
            sin_v: v.sin(),
        }
    }
    // The phase-shift identities `d^n cos(t) = cos(t + n Pi/2)`; the Z term is
    // multiplied by 0.0 (not dropped) when Nu > 0, as natively.
    fn derivative_n(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
        let state: EllipsoidState = *self.state.read();
        let frame: EvalFrame = EvalFrame::new(state.pos);
        let phase_u: f64 = f64::from(nu) * PI / 2.0;
        let phase_v: f64 = f64::from(nv) * PI / 2.0;
        let x_coeff: f64 = state.a * (v + phase_v).cos() * (u + phase_u).cos();
        let y_coeff: f64 = state.b * (v + phase_v).cos() * (u + phase_u).sin();
        let z_coeff: f64 = state.c * (v + phase_v).sin() * if nu == 0 { 1.0 } else { 0.0 };
        Vector3::from(frame.x * x_coeff + frame.y * y_coeff + frame.z * z_coeff)
    }
    fn dump_parameters(&self, stream: &mut DumpStream) -> io::Result<()> {
        let state: EllipsoidState = *self.state.read();
        dump::dump_real(stream, "A", state.a)?;
        dump::dump_real(stream, "B", state.b)?;
        dump::dump_real(stream, "C", state.c)
    }
}
impl_eval_elementary_surface!(GeomEvalEllipsoidSurface);
