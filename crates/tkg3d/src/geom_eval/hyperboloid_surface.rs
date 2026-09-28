// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_HyperboloidSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_HyperboloidSurface`: a hyperboloid of revolution about ZDir, of one sheet
//! `P = O + R1 cosh(v) (cos(u) XDir + sin(u) YDir) + R2 sinh(v) ZDir` or of two sheets
//! (the upper one) `P = O + R2 sinh(v) (cos(u) XDir + sin(u) YDir) + R1 cosh(v) ZDir`.
use super::elementary_eval::{
    EvalElementarySurface, EvalFrame, PatchDerivatives, diagonal_quadric_coefficients,
    impl_eval_elementary_surface, positive,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, StandardTransient};
use rustycad_tkmath::gp::{Axis3, Vector3, Xyz};
use std::f64::consts::{FRAC_PI_2, PI};
use std::io;

const CLASS: &str = "GeomEval_HyperboloidSurface";

/// The sheet count of a hyperboloid (`GeomEval_HyperboloidSurface::SheetMode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum HyperboloidSheetMode {
    /// Hyperboloid of one sheet (`OneSheet`).
    #[default]
    OneSheet = 0,
    /// Hyperboloid of two sheets, of which the surface is the upper one (`TwoSheets`).
    TwoSheets = 1,
}

#[derive(Clone, Copy)]
struct HyperboloidState {
    pos: Axis3,
    r1: f64,
    r2: f64,
    mode: HyperboloidSheetMode,
}

/// A hyperboloid of revolution; `UReverse`, `VReverse`, the isos and `Transform`
/// raise Standard_NotImplemented;
/// `GeomEvalHyperboloidSurface::new(frame, 2.0, 1.0, HyperboloidSheetMode::OneSheet)?`.
pub struct GeomEvalHyperboloidSurface {
    parent: StandardTransient,
    state: SharedState<HyperboloidState>,
}
impl_transient!(
    GeomEvalHyperboloidSurface,
    "GeomEval_HyperboloidSurface",
    dyn crate::geom::GeomElementarySurface,
    parent,
    interfaces: [
        dyn crate::geom::GeomGeometry,
        dyn crate::geom::GeomSurface,
        dyn crate::geom::GeomElementarySurface
    ]
);
// Copy() reruns the constructor on the stored, already valid values.
impl Clone for GeomEvalHyperboloidSurface {
    fn clone(&self) -> Self {
        Self::stored(*self.state.read())
    }
}

impl GeomEvalHyperboloidSurface {
    fn stored(state: HyperboloidState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// Hyperboloid of radii `r1` and `r2` (the native default mode is `OneSheet`);
    /// Standard_ConstructionError unless both are > 0;
    /// `GeomEvalHyperboloidSurface::new(frame, 2.0, 1.0, HyperboloidSheetMode::OneSheet)?`.
    pub fn new(
        position: Axis3,
        r1: f64,
        r2: f64,
        mode: HyperboloidSheetMode,
    ) -> Result<Self, Failure> {
        let r1: f64 = positive(r1, "GeomEval_HyperboloidSurface: R1")?;
        let r2: f64 = positive(r2, "GeomEval_HyperboloidSurface: R2")?;
        Ok(Self::stored(HyperboloidState {
            pos: position,
            r1,
            r2,
            mode,
        }))
    }
    /// The first radius (`R1`); `surface.r1()`.
    pub fn r1(&self) -> f64 {
        self.state.read().r1
    }
    /// The second radius (`R2`); `surface.r2()`.
    pub fn r2(&self) -> f64 {
        self.state.read().r2
    }
    /// The sheet mode (`Mode`); `surface.mode()`.
    pub fn mode(&self) -> HyperboloidSheetMode {
        self.state.read().mode
    }
    /// Set R1; Standard_ConstructionError unless > 0 (`SetR1`); `surface.set_r1(3.0)?`.
    pub fn set_r1(&self, r1: f64) -> Result<(), Failure> {
        self.state.write().r1 = positive(r1, "GeomEval_HyperboloidSurface::SetR1: R1")?;
        Ok(())
    }
    /// Set R2; Standard_ConstructionError unless > 0 (`SetR2`); `surface.set_r2(3.0)?`.
    pub fn set_r2(&self, r2: f64) -> Result<(), Failure> {
        self.state.write().r2 = positive(r2, "GeomEval_HyperboloidSurface::SetR2: R2")?;
        Ok(())
    }
    /// Set the sheet mode (`SetMode`); `surface.set_mode(HyperboloidSheetMode::TwoSheets)`.
    pub fn set_mode(&self, mode: HyperboloidSheetMode) {
        self.state.write().mode = mode;
    }
    /// The implicit equation `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` in the absolute
    /// frame, from `X^2/R1^2 + Y^2/R1^2 - Z^2/R2^2 - 1 = 0` (one sheet) or
    /// `X^2/R2^2 + Y^2/R2^2 - Z^2/R1^2 + 1 = 0` (two sheets) (`Coefficients`);
    /// `surface.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let state: HyperboloidState = *self.state.read();
        let (radial, axial, d) = match state.mode {
            HyperboloidSheetMode::OneSheet => (state.r1, state.r2, -1.0),
            HyperboloidSheetMode::TwoSheets => (state.r2, state.r1, 1.0),
        };
        let local: [f64; 3] = [
            1.0 / (radial * radial),
            1.0 / (radial * radial),
            -1.0 / (axial * axial),
        ];
        diagonal_quadric_coefficients(state.pos, local, d)
    }
}

/// The hyperboloid formulas at one `(U, V)`: both modes are
/// `radial * f(v) (cos(u) XD + sin(u) YD) + axial * g(v) ZD`, with
/// `(f, g) = (cosh, sinh)` and radii `(R1, R2)` for one sheet, `(sinh, cosh)` and
/// `(R2, R1)` for two; `f[1]` and `g[1]` are the V derivatives.
pub(crate) struct HyperboloidPatch {
    frame: EvalFrame,
    radial: f64,
    axial: f64,
    f: [f64; 2],
    g: [f64; 2],
    cos_u: f64,
    sin_u: f64,
}
impl HyperboloidPatch {
    // `XD * (radial * f[k] * cx) + YD * (radial * f[k] * cy) (+ ZD * (axial * g[k]))`.
    fn combine(&self, k: usize, [cx, cy]: [f64; 2], with_axial: bool) -> Xyz {
        let fr: &EvalFrame = &self.frame;
        let planar: Xyz =
            fr.x * (self.radial * self.f[k] * cx) + fr.y * (self.radial * self.f[k] * cy);
        if with_axial {
            return planar + fr.z * (self.axial * self.g[k]);
        }
        planar
    }
}
impl PatchDerivatives for HyperboloidPatch {
    // `O + XD * (..) + YD * (..) + ZD * (..)`, summed from the origin as natively.
    fn point(&self) -> Xyz {
        let (fr, radial) = (&self.frame, self.radial);
        let (cu, su) = (self.cos_u, self.sin_u);
        fr.origin
            + fr.x * (radial * self.f[0] * cu)
            + fr.y * (radial * self.f[0] * su)
            + fr.z * (self.axial * self.g[0])
    }
    fn first(&self) -> [Xyz; 2] {
        let (cu, su) = (self.cos_u, self.sin_u);
        [
            self.combine(0, [-su, cu], false),
            self.combine(1, [cu, su], true),
        ]
    }
    fn second(&self) -> [Xyz; 3] {
        let (cu, su) = (self.cos_u, self.sin_u);
        [
            self.combine(0, [-cu, -su], false),
            self.combine(0, [cu, su], true),
            self.combine(1, [-su, cu], false),
        ]
    }
    fn third(&self) -> [Xyz; 4] {
        let (cu, su) = (self.cos_u, self.sin_u);
        [
            self.combine(0, [su, -cu], false),
            self.combine(1, [cu, su], true),
            self.combine(1, [-cu, -su], false),
            self.combine(0, [-su, cu], false),
        ]
    }
}

impl GeomEvalHyperboloidSurface {
    // The radii and the hyperbolic pairs of the current mode, as HyperboloidPatch
    // documents them.
    fn mode_terms(state: &HyperboloidState, v: f64) -> (f64, f64, [f64; 2], [f64; 2]) {
        let (cosh, sinh) = (v.cosh(), v.sinh());
        match state.mode {
            HyperboloidSheetMode::OneSheet => (state.r1, state.r2, [cosh, sinh], [sinh, cosh]),
            HyperboloidSheetMode::TwoSheets => (state.r2, state.r1, [sinh, cosh], [cosh, sinh]),
        }
    }
}

impl EvalElementarySurface for GeomEvalHyperboloidSurface {
    const CLASS: &'static str = CLASS;
    const BOUNDS: (f64, f64, f64, f64) =
        (0.0, 2.0 * PI, -precision::infinite(), precision::infinite());
    const U_CLOSED: bool = true;
    type Patch = HyperboloidPatch;
    fn patch(&self, u: f64, v: f64) -> HyperboloidPatch {
        let state: HyperboloidState = *self.state.read();
        let (radial, axial, f, g) = Self::mode_terms(&state, v);
        HyperboloidPatch {
            frame: EvalFrame::new(state.pos),
            radial,
            axial,
            f,
            g,
            cos_u: u.cos(),
            sin_u: u.sin(),
        }
    }
    // `cos/sin(U + Nu * Pi/2)`; the V derivative alternates the hyperbolic pair.
    // The Z coefficient is 0.0 when Nu > 0 but is still added, as natively.
    fn derivative_n(&self, u: f64, v: f64, nu: i32, nv: i32) -> Vector3 {
        let state: HyperboloidState = *self.state.read();
        let frame: EvalFrame = EvalFrame::new(state.pos);
        let u_shift: f64 = u + f64::from(nu) * FRAC_PI_2;
        let (dn_cos_u, dn_sin_u) = (u_shift.cos(), u_shift.sin());
        let (radial, axial, f, g) = Self::mode_terms(&state, v);
        let k: usize = if nv % 2 == 0 { 0 } else { 1 };
        let x_coeff: f64 = radial * f[k] * dn_cos_u;
        let y_coeff: f64 = radial * f[k] * dn_sin_u;
        let z_coeff: f64 = if nu == 0 { axial * g[k] } else { 0.0 };
        Vector3::from(frame.x * x_coeff + frame.y * y_coeff + frame.z * z_coeff)
    }
    // `(int)myMode` keeps its spelling: DumpFieldToName strips only `my`/`a`/`an` prefixes.
    fn dump_parameters(&self, stream: &mut DumpStream) -> io::Result<()> {
        let state: HyperboloidState = *self.state.read();
        dump::dump_real(stream, "R1", state.r1)?;
        dump::dump_real(stream, "R2", state.r2)?;
        dump::dump_integer(stream, "(int)myMode", state.mode as i32)
    }
}
impl_eval_elementary_surface!(GeomEvalHyperboloidSurface);
