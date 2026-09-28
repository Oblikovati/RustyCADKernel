// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_OffsetCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_OffsetCurve`: the curve at a constant distance along the normal of a basis.
use super::bspline_curve::Geom2dBSplineCurve;
use super::curve::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, dump_curve};
use super::errors;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_field};
use super::offset_curve_utils::{
    adjust_derivative, calculate_d0, calculate_d1, calculate_d2, calculate_d3,
};
use super::trimmed_curve::Geom2dTrimmedCurve;
use crate::geom2d_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2, resolution};
use std::io;

struct OffsetState {
    basis: Handle<dyn Geom2dCurve>,
    eval_rep: Handle<dyn RepCurveDesc>,
    offset_value: f64,
    basis_continuity: Shape,
}

/// An offset curve; `Handle::new(Geom2dOffsetCurve::new(&circle.cast(), 1.0, false)?)`.
///
/// Unless trimmed, the basis curve is shared with the caller, as natively.
pub struct Geom2dOffsetCurve {
    parent: StandardTransient,
    state: SharedState<OffsetState>,
}
impl_transient!(
    Geom2dOffsetCurve,
    "Geom2d_OffsetCurve",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);

// The result of SetBasisCurve's unwrapping, kept even when it fails part-way.
struct BasisResolution {
    offset: f64,
    continuity: Option<Shape>,
}

// SetBasisCurve without the object's lock: unwrap trimmed and offset curves (adding
// the nested offsets), reject a C0 B-spline that is not G1, re-trim if needed.
fn resolve_basis(
    curve: &Handle<dyn Geom2dCurve>,
    not_check_c0: bool,
    resolution_state: &mut BasisResolution,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let Some(source) = curve.get() else {
        return Err(errors::construction(
            "Geom2d_OffsetCurve: the basis curve is null",
        ));
    };
    let (uf, ul) = (source.first_parameter(), source.last_parameter());
    let mut checking: Handle<dyn Geom2dCurve> = curve.clone();
    let mut is_trimmed: bool = false;
    loop {
        let trimmed: Handle<Geom2dTrimmedCurve> = checking.cast();
        let offset: Handle<Geom2dOffsetCurve> = checking.cast();
        if trimmed.is_null() && offset.is_null() {
            break;
        }
        if let Some(trimmed) = trimmed.get() {
            checking = trimmed.basis_curve();
            is_trimmed = true;
        }
        let offset: Handle<Geom2dOffsetCurve> = checking.cast();
        if let Some(offset) = offset.get() {
            let nested: f64 = offset.offset();
            checking = offset.basis_curve();
            resolution_state.offset += nested;
        }
    }
    check_c0(&checking, not_check_c0, uf, ul, resolution_state)?;
    if is_trimmed {
        let trimmed = Geom2dTrimmedCurve::new(&checking, uf, ul, true, true)?;
        return Ok(Handle::new(trimmed).cast());
    }
    Ok(checking)
}

fn check_c0(
    checking: &Handle<dyn Geom2dCurve>,
    not_check_c0: bool,
    uf: f64,
    ul: f64,
    resolution_state: &mut BasisResolution,
) -> Result<(), Failure> {
    let mut continuity: Shape = checking.continuity();
    resolution_state.continuity = Some(continuity);
    let mut is_c0: bool = !not_check_c0 && continuity == Shape::C0;
    let bspline: Handle<Geom2dBSplineCurve> = checking.cast();
    if let (true, Some(bspline)) = (is_c0, bspline.get()) {
        if bspline.is_g1(uf, ul, precision::angular())? {
            continuity = Shape::G1;
            resolution_state.continuity = Some(continuity);
            is_c0 = false;
        }
        if is_c0 {
            return Err(errors::construction(
                "Offset on C0 curve: the B-spline basis is not G1; expected a G1 basis",
            ));
        }
    }
    Ok(())
}

impl Geom2dOffsetCurve {
    /// Offset of `curve` by `offset`; trimmed and offset inputs are unwrapped, and a
    /// C0 B-spline basis must be G1 unless `not_check_c0`
    /// (`Geom2d_OffsetCurve(C, Offset, isNotCheckC0)`);
    /// `Geom2dOffsetCurve::new(&curve, 1.0, false)?`.
    pub fn new(
        curve: &Handle<dyn Geom2dCurve>,
        offset: f64,
        not_check_c0: bool,
    ) -> Result<Self, Failure> {
        let mut resolved = BasisResolution {
            offset,
            continuity: None,
        };
        let basis: Handle<dyn Geom2dCurve> = resolve_basis(curve, not_check_c0, &mut resolved)?;
        let state = OffsetState {
            basis,
            eval_rep: Handle::default(),
            offset_value: resolved.offset,
            basis_continuity: resolved.continuity.unwrap_or(Shape::C0),
        };
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// The native copy constructor: the basis and the evaluation representation are
    /// deep-copied (`Geom2d_OffsetCurve(const Geom2d_OffsetCurve&)`);
    /// `Geom2dOffsetCurve::copy_of(&curve)?`.
    pub fn copy_of(other: &Geom2dOffsetCurve) -> Result<Self, Failure> {
        let source = other.state.read();
        let state = OffsetState {
            basis: source.basis.copy()?.cast(),
            eval_rep: rep_utils::clone_curve_desc(&source.eval_rep)?,
            offset_value: source.offset_value,
            basis_continuity: source.basis_continuity,
        };
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// Whether an evaluation representation is set (`HasEvalRepresentation`);
    /// `curve.has_eval_representation()`.
    pub fn has_eval_representation(&self) -> bool {
        self.state.read().eval_rep.is_some()
    }
    /// The evaluation representation, possibly null (`EvalRepresentation`); `curve.eval_representation()`.
    pub fn eval_representation(&self) -> Handle<dyn RepCurveDesc> {
        self.state.read().eval_rep.clone()
    }
    /// Validate and set the evaluation representation (`SetEvalRepresentation`);
    /// `curve.set_eval_representation(desc)?`.
    pub fn set_eval_representation(&self, desc: Handle<dyn RepCurveDesc>) -> Result<(), Failure> {
        rep_utils::validate_curve_desc(&desc, std::ptr::from_ref(self).cast())?;
        self.state.write().eval_rep = desc;
        Ok(())
    }
    /// Remove the evaluation representation (`ClearEvalRepresentation`); `curve.clear_eval_representation()`.
    pub fn clear_eval_representation(&self) {
        self.state.write().eval_rep = Handle::default();
    }
    /// Replace the basis curve, as the constructor does (`SetBasisCurve`). Like the native
    /// code, a rejected basis leaves the accumulated offset and continuity stored;
    /// `curve.set_basis_curve(&basis, false)?`.
    pub fn set_basis_curve(
        &self,
        curve: &Handle<dyn Geom2dCurve>,
        not_check_c0: bool,
    ) -> Result<(), Failure> {
        self.clear_eval_representation();
        let mut resolved = BasisResolution {
            offset: self.offset(),
            continuity: None,
        };
        let result: Result<Handle<dyn Geom2dCurve>, Failure> =
            resolve_basis(curve, not_check_c0, &mut resolved);
        let mut state = self.state.write();
        state.offset_value = resolved.offset;
        if let Some(continuity) = resolved.continuity {
            state.basis_continuity = continuity;
        }
        state.basis = result?;
        Ok(())
    }
    /// Change the offset distance (`SetOffsetValue`); `curve.set_offset_value(2.0)`.
    pub fn set_offset_value(&self, offset: f64) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.offset_value = offset;
    }
    /// The basis curve (`BasisCurve`); `curve.basis_curve()`.
    pub fn basis_curve(&self) -> Handle<dyn Geom2dCurve> {
        self.state.read().basis.clone()
    }
    /// The offset distance (`Offset`); `curve.offset()`.
    pub fn offset(&self) -> f64 {
        self.state.read().offset_value
    }
    /// Continuity of the unwrapped basis, G1 for a G1 C0 B-spline (`GetBasisCurveContinuity`);
    /// `curve.basis_curve_continuity()`.
    pub fn basis_curve_continuity(&self) -> Shape {
        self.state.read().basis_continuity
    }
}
impl Geom2dGeometry for Geom2dOffsetCurve {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.transform(transform)?;
        state.offset_value *= transform.scale_factor().abs();
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom2d_OffsetCurve")?;
        dump_base(stream, depth, dump_curve)?;
        dump_field(stream, depth, "basisCurve", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        dump::dump_real(stream, "offsetValue", state.offset_value)?;
        dump::dump_integer(
            stream,
            "BasisCurveContinuity",
            state.basis_continuity as i32,
        )
    }
}

// The Continuity switch: offsetting loses one order of continuity of the basis.
fn offset_continuity(basis: Shape) -> Shape {
    match basis {
        Shape::C0 | Shape::C1 => Shape::C0,
        Shape::C2 => Shape::C1,
        Shape::C3 => Shape::C2,
        Shape::CN => Shape::CN,
        Shape::G1 => Shape::G1,
        Shape::G2 => Shape::G2,
    }
}

fn undefined(member: &str) -> Failure {
    errors::undefined_derivative(&format!(
        "Geom2d_OffsetCurve::{member}: the offset normal is undefined; expected a non-degenerate basis tangent"
    ))
}

impl Geom2dCurve for Geom2dOffsetCurve {
    fn reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.reverse()?;
        state.offset_value = -state.offset_value;
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let basis: Handle<dyn Geom2dCurve> = self.state.read().basis.clone();
        basis.reversed_parameter(u)
    }
    fn transformed_parameter(&self, u: f64, transform: Transform2) -> f64 {
        self.state.read().basis.transformed_parameter(u, transform)
    }
    fn parametric_transformation(&self, transform: Transform2) -> f64 {
        self.state.read().basis.parametric_transformation(transform)
    }
    fn first_parameter(&self) -> f64 {
        self.state.read().basis.first_parameter()
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().basis.last_parameter()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let first: Point2 = self.d0(self.first_parameter())?;
        let last: Point2 = self.d0(self.last_parameter())?;
        Ok(first.square_distance(last) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        self.state.read().basis.is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        self.state.read().basis.period()
    }
    fn continuity(&self) -> Shape {
        offset_continuity(self.state.read().basis_continuity)
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom2d_OffsetCurve::IsCN: N = {n}; expected N >= 0"
            )));
        }
        self.state.read().basis.is_cn(n + 1)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let state = self.state.read();
        if let Some(point) = rep_utils::try_eval_curve_d0(&state.eval_rep, u)? {
            return Ok(point);
        }
        let basis: Geom2dCurveD1 = state.basis.eval_d1(u)?;
        let mut value: Point2 = basis.point;
        if !calculate_d0(&mut value, basis.d1, state.offset_value)? {
            return Err(errors::undefined_value(
                "Geom2d_OffsetCurve::EvalD0: the basis tangent is null; expected a regular point",
            ));
        }
        Ok(value)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d1(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: Geom2dCurveD2 = state.basis.eval_d2(u)?;
        let (mut point, mut d1) = (basis.point, basis.d1);
        if !calculate_d1(&mut point, &mut d1, basis.d2, state.offset_value) {
            return Err(undefined("EvalD1"));
        }
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d2(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: Geom2dCurveD3 = state.basis.eval_d3(u)?;
        let (mut point, mut d1, mut d2, mut d3) = (basis.point, basis.d1, basis.d2, basis.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let mut dummy_d4 = Vector2::default();
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut dummy_d4];
            is_direction_change = adjust_derivative(&*state.basis, 3, u, derivatives)?;
        }
        if !calculate_d2(
            &mut point,
            &mut d1,
            &mut d2,
            d3,
            is_direction_change,
            state.offset_value,
        ) {
            return Err(undefined("EvalD2"));
        }
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d3(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: Geom2dCurveD3 = state.basis.eval_d3(u)?;
        let mut d4: Vector2 = state.basis.eval_dn(u, 4)?;
        let (mut point, mut d1, mut d2, mut d3) = (basis.point, basis.d1, basis.d2, basis.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut d4];
            is_direction_change = adjust_derivative(&*state.basis, 4, u, derivatives)?;
        }
        let derivatives = [&mut d1, &mut d2, &mut d3];
        if !calculate_d3(
            &mut point,
            derivatives,
            d4,
            is_direction_change,
            state.offset_value,
        ) {
            return Err(undefined("EvalD3"));
        }
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Geom2d_OffsetCurve::EvalDN: N = {n}; expected N >= 1"
            )));
        }
        if let Some(vector) = rep_utils::try_eval_curve_dn(&self.state.read().eval_rep, u, n)? {
            return Ok(vector);
        }
        match n {
            1 => Ok(self.eval_d1(u)?.d1),
            2 => Ok(self.eval_d2(u)?.d2),
            3 => Ok(self.eval_d3(u)?.d3),
            _ => self.state.read().basis.eval_dn(u, n),
        }
    }
}
