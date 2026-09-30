// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_TrihedronLaw.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_TrihedronLaw`: a moving frame (tangent, normal, binormal) along a curve.
use super::pipe_error::GeomFillPipeError;
use super::sweep_failures::{live, not_implemented};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Vector3;

/// The frame a trihedron law evaluates (`D0` outputs and `GetAverageLaw`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrihedronD0 {
    /// `Tangent`.
    pub tangent: Vector3,
    /// `Normal`.
    pub normal: Vector3,
    /// `BiNormal`.
    pub binormal: Vector3,
}

/// The frame and its first derivatives (`D1` outputs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrihedronD1 {
    /// `Tangent`.
    pub tangent: Vector3,
    /// `DTangent`.
    pub d_tangent: Vector3,
    /// `Normal`.
    pub normal: Vector3,
    /// `DNormal`.
    pub d_normal: Vector3,
    /// `BiNormal`.
    pub binormal: Vector3,
    /// `DBiNormal`.
    pub d_binormal: Vector3,
}

/// The frame and its first two derivatives (`D2` outputs).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrihedronD2 {
    /// `Tangent`.
    pub tangent: Vector3,
    /// `DTangent`.
    pub d_tangent: Vector3,
    /// `D2Tangent`.
    pub d2_tangent: Vector3,
    /// `Normal`.
    pub normal: Vector3,
    /// `DNormal`.
    pub d_normal: Vector3,
    /// `D2Normal`.
    pub d2_normal: Vector3,
    /// `BiNormal`.
    pub binormal: Vector3,
    /// `DBiNormal`.
    pub d_binormal: Vector3,
    /// `D2BiNormal`.
    pub d2_binormal: Vector3,
}

/// The two protected curves of `GeomFill_TrihedronLaw`.
#[derive(Clone, Default)]
struct LawCurves {
    curve: Handle<dyn Adaptor3dCurve>,
    trimmed: Handle<dyn Adaptor3dCurve>,
}

/// The protected data of `GeomFill_TrihedronLaw` (`myCurve`, `myTrimmed`), which
/// each concrete law embeds and returns from
/// [`GeomFillTrihedronLaw::trihedron_base`]; `GeomFillTrihedronLawBase::default()`.
#[derive(Default)]
pub struct GeomFillTrihedronLawBase {
    curves: SharedState<LawCurves>,
}

impl GeomFillTrihedronLawBase {
    /// The base `SetCurve`: `myCurve = myTrimmed = C`; `base.set_curve(&curve)`.
    pub fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) {
        let mut curves = self.curves.write();
        curves.curve = curve.clone();
        curves.trimmed = curve.clone();
    }

    /// The base `SetInterval`: `myTrimmed = myCurve->Trim(First, Last, 0)`;
    /// `base.set_interval(0.0, 1.0)?`.
    pub fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        let curve = self.curve();
        let trimmed = live(&curve, "GeomFill_TrihedronLaw::SetInterval", "myCurve")?.trim(
            first, last, 0.0,
        )?;
        self.curves.write().trimmed = trimmed;
        Ok(())
    }

    /// `GetInterval`: the bounds of `myTrimmed`; `base.get_interval()?`.
    pub fn get_interval(&self) -> Result<(f64, f64), Failure> {
        let trimmed = self.trimmed();
        let curve = live(&trimmed, "GeomFill_TrihedronLaw::GetInterval", "myTrimmed")?;
        Ok((curve.first_parameter()?, curve.last_parameter()?))
    }

    /// `myCurve` (a null handle before `SetCurve`); `base.curve()`.
    pub fn curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.curves.read().curve.clone()
    }

    /// `myTrimmed` (a null handle before `SetCurve`); `base.trimmed()`.
    pub fn trimmed(&self) -> Handle<dyn Adaptor3dCurve> {
        self.curves.read().trimmed.clone()
    }
}

/// `GeomFill_TrihedronLaw`: the abstract trihedron law, reached through
/// `Handle<dyn GeomFillTrihedronLaw>` (docs/geomalgo-design.md, section 14).
///
/// Every member takes `&self`: a law keeps its state behind locks, as the native
/// evaluators that change the object (`D0` is not `const`). Output references are
/// `&mut` frames, written as the native code writes them, and every member that
/// reaches the curve returns `Result`.
/// `let law: Handle<dyn GeomFillTrihedronLaw> = Handle::new(GeomFillFrenet::new()).cast();`
pub trait GeomFillTrihedronLaw: Transient {
    /// The protected `myCurve`/`myTrimmed` of the base class.
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase;

    /// `SetCurve(C)`: loads the curve; the base stores it and returns true.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.trihedron_base().set_curve(curve);
        Ok(true)
    }

    /// `Copy()`: a new law of the same kind on the same curve.
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure>;

    /// `ErrorStatus()`: PipeOk by default.
    fn error_status(&self) -> GeomFillPipeError {
        GeomFillPipeError::PipeOk
    }

    /// `D0(Param, Tangent, Normal, BiNormal)`: the frame at `param`.
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure>;

    /// `D1(...)`: the frame and its derivatives; the base raises
    /// Standard_NotImplemented with the native message " GeomFill_TrihedronLaw::D2".
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let _ = (param, frame);
        Err(not_implemented(" GeomFill_TrihedronLaw::D2"))
    }

    /// `D2(...)`: up to the second derivatives; the base raises
    /// Standard_NotImplemented.
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let _ = (param, frame);
        Err(not_implemented(" GeomFill_TrihedronLaw::D2"))
    }

    /// `NbIntervals(S)`: the intervals of continuity `shape`.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure>;

    /// `Intervals(T, S)`: their bounds, `NbIntervals + 1` values.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure>;

    /// `SetInterval(First, Last)`: the base trims `myCurve` into `myTrimmed`.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.trihedron_base().set_interval(first, last)
    }

    /// `GetInterval(First, Last)` (non-virtual): the bounds of `myTrimmed`.
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        self.trihedron_base().get_interval()
    }

    /// `GetAverageLaw(ATangent, ANormal, ABiNormal)`: a mean frame of the law.
    fn get_average_law(&self) -> Result<TrihedronD0, Failure>;

    /// `IsConstant()`: false by default.
    fn is_constant(&self) -> Result<bool, Failure> {
        Ok(false)
    }

    /// `IsOnlyBy3dCurve()`: false by default.
    fn is_only_by_3d_curve(&self) -> Result<bool, Failure> {
        Ok(false)
    }
}
impl_transient_interface!(
    dyn GeomFillTrihedronLaw,
    "GeomFill_TrihedronLaw",
    StandardTransient
);
