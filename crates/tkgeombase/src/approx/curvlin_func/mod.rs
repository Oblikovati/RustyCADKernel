// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvlinFunc.hxx/.cxx (fields, constructors, accessors)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_CurvlinFunc`: a curve (or one or two curves on surfaces) seen as a
//! function of its normalized curvilinear abscissa `S` in `[0, 1]`.
mod eval;
mod intervals;
mod parameters;

use crate::native_failure::failure;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};

/// The locked fields: the curves, the parameter tables `Ui`/`Si` (`Si` the
/// normalized length at `Ui`), the lengths and the last parameters found.
#[derive(Clone)]
struct CurvlinState {
    c3d: Handle<dyn Adaptor3dCurve>,
    c2d1: Handle<dyn Adaptor2dCurve2d>,
    c2d2: Handle<dyn Adaptor2dCurve2d>,
    surf1: Handle<dyn Adaptor3dSurface>,
    surf2: Handle<dyn Adaptor3dSurface>,
    case: i32,
    first_s: f64,
    last_s: f64,
    first_u1: f64,
    last_u1: f64,
    first_u2: f64,
    last_u2: f64,
    length: f64,
    length1: f64,
    length2: f64,
    tol_len: f64,
    ui_1: Option<Array1<'static, f64>>,
    si_1: Option<Array1<'static, f64>>,
    ui_2: Option<Array1<'static, f64>>,
    si_2: Option<Array1<'static, f64>>,
    prev_s: f64,
    prev_u: f64,
}

/// `Approx_CurvlinFunc`: case 1 a 3D curve, case 2 a curve on a surface, case 3 two
/// curves on two surfaces (their lengths averaged). Shared through handles; every
/// member takes `&self`, and the members the native class declares `const` still
/// record the last `(S, U)` found, as the native `const_cast` does.
///
/// ```ignore
/// let f = CurvlinFunc::from_curve(&adaptor, 1e-6)?;
/// let u = f.get_u_parameter(&*adaptor, 0.5, 1)?;
/// ```
pub struct CurvlinFunc {
    parent: StandardTransient,
    state: SharedState<CurvlinState>,
}
impl_transient!(CurvlinFunc, "Approx_CurvlinFunc", StandardTransient, parent);

/// The Standard_NullObject of a null handle the native code dereferences.
pub(crate) fn null_handle(what: &str) -> Failure {
    failure(
        "Standard_NullObject",
        &format!("Approx_CurvlinFunc: null {what}"),
    )
}

/// `Adaptor3d_CurveOnSurface CurOnSur; CurOnSur.Load(C2D); CurOnSur.Load(S);`.
pub(crate) fn curve_on_surface(
    c2d: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
) -> Result<Adaptor3dCurveOnSurface, Failure> {
    let adaptor = Adaptor3dCurveOnSurface::default();
    reload(&adaptor, c2d, surf)?;
    Ok(adaptor)
}

/// `CurOnSur.Load(C2D); CurOnSur.Load(S);` on an adaptor already in use: the
/// native members reuse one local adaptor for both curves of case 3.
pub(crate) fn reload(
    cons: &Adaptor3dCurveOnSurface,
    c2d: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
) -> Result<(), Failure> {
    cons.load_curve(c2d)?;
    cons.load_surface(surf)
}

impl CurvlinFunc {
    fn with_state(case: i32, tol: f64, curves: CurvlinCurves) -> Result<Self, Failure> {
        let state = CurvlinState {
            c3d: curves.c3d,
            c2d1: curves.c2d1,
            c2d2: curves.c2d2,
            surf1: curves.surf1,
            surf2: curves.surf2,
            case,
            first_s: 0.0,
            last_s: 1.0,
            first_u1: 0.0,
            last_u1: 0.0,
            first_u2: 0.0,
            last_u2: 0.0,
            length: 0.0,
            length1: 0.0,
            length2: 0.0,
            tol_len: tol,
            ui_1: None,
            si_1: None,
            ui_2: None,
            si_2: None,
            prev_s: 0.0,
            prev_u: 0.0,
        };
        let func = Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        };
        func.init()?;
        Ok(func)
    }

    /// `Approx_CurvlinFunc(C, Tol)`: case 1, the lengths integrated within `tol`;
    /// `CurvlinFunc::from_curve(&adaptor, 1e-6)?`.
    pub fn from_curve(c: &Handle<dyn Adaptor3dCurve>, tol: f64) -> Result<Self, Failure> {
        let curves = CurvlinCurves {
            c3d: c.clone(),
            ..CurvlinCurves::default()
        };
        Self::with_state(1, tol, curves)
    }

    /// `Approx_CurvlinFunc(C2D, S, Tol)`: case 2;
    /// `CurvlinFunc::from_curve_on_surface(&c2d, &surface, 1e-6)?`.
    pub fn from_curve_on_surface(
        c2d: &Handle<dyn Adaptor2dCurve2d>,
        s: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let curves = CurvlinCurves {
            c2d1: c2d.clone(),
            surf1: s.clone(),
            ..CurvlinCurves::default()
        };
        Self::with_state(2, tol, curves)
    }

    /// `Approx_CurvlinFunc(C2D1, C2D2, S1, S2, Tol)`: case 3;
    /// `CurvlinFunc::from_two_curves_on_surfaces(&c1, &c2, &s1, &s2, 1e-6)?`.
    pub fn from_two_curves_on_surfaces(
        c2d1: &Handle<dyn Adaptor2dCurve2d>,
        c2d2: &Handle<dyn Adaptor2dCurve2d>,
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        tol: f64,
    ) -> Result<Self, Failure> {
        let curves = CurvlinCurves {
            c3d: Handle::default(),
            c2d1: c2d1.clone(),
            c2d2: c2d2.clone(),
            surf1: s1.clone(),
            surf2: s2.clone(),
        };
        Self::with_state(3, tol, curves)
    }

    /// `SetTol(Tol)`: the tolerance of the length integrations; `f.set_tol(1e-7)`.
    pub fn set_tol(&self, tol: f64) {
        self.state.write().tol_len = tol;
    }

    /// `FirstParameter()`: the first S (0 until Trim); `f.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        self.state.read().first_s
    }

    /// `LastParameter()`: the last S (1 until Trim); `f.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        self.state.read().last_s
    }

    /// `GetLength()`: the length (the mean length in case 3); `f.get_length()`.
    pub fn get_length(&self) -> f64 {
        self.state.read().length
    }

    fn snapshot(&self) -> CurvlinState {
        self.state.snapshot()
    }
}

/// The curves of the three cases (null where unused).
#[derive(Default)]
struct CurvlinCurves {
    c3d: Handle<dyn Adaptor3dCurve>,
    c2d1: Handle<dyn Adaptor2dCurve2d>,
    c2d2: Handle<dyn Adaptor2dCurve2d>,
    surf1: Handle<dyn Adaptor3dSurface>,
    surf2: Handle<dyn Adaptor3dSurface>,
}
