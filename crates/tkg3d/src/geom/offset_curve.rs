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
// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_OffsetCurve`: the curve at a constant distance from a basis curve, along
//! `D1 ^ V` for a fixed reference direction `V`.
use super::bspline_curve::GeomBSplineCurve;
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, dump_curve};
use super::errors;
use super::geometry::GeomGeometry;
use super::offset_curve_utils::{
    adjust_derivative, calculate_d0, calculate_d1, calculate_d2, calculate_d3,
};
use super::trimmed_curve::GeomTrimmedCurve;
use crate::geom_eval::{RepCurveDesc, rep_utils};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::offset_curve_utils::offset_curve_continuity;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Direction3, Point3, Transform3, Vector3, resolution};
use std::io;

struct OffsetState {
    basis: Handle<dyn GeomCurve>,
    eval_rep: Handle<dyn RepCurveDesc>,
    direction: Direction3,
    offset_value: f64,
    basis_continuity: Shape,
}

/// An offset curve; `Handle::new(GeomOffsetCurve::new(&circle.cast(), 1.0, dir, false)?)`.
///
/// The basis curve is a private copy of the curve given to the constructor or to
/// `SetBasisCurve`, as natively.
pub struct GeomOffsetCurve {
    parent: StandardTransient,
    state: SharedState<OffsetState>,
}
impl_transient!(
    GeomOffsetCurve,
    "Geom_OffsetCurve",
    dyn GeomCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve]
);

// The offset, direction and continuity that SetBasisCurve writes into the object,
// kept even when it fails part-way, as the native member leaves them.
struct BasisResolution {
    offset: f64,
    direction: Direction3,
    continuity: Option<Shape>,
}

// One nested offset folded into the accumulated offset and direction
// (`Vdir = PrevOff * V1 + offsetValue * V2`).
fn fold_nested_offset(
    nested: &GeomOffsetCurve,
    resolved: &mut BasisResolution,
) -> Result<(), Failure> {
    let previous: f64 = nested.offset();
    let v1 = Vector3::from(nested.direction());
    let v2 = Vector3::from(resolved.direction);
    let vdir: Vector3 = v1 * previous + v2 * resolved.offset;
    if resolved.offset >= 0. {
        resolved.offset = vdir.magnitude();
        resolved.direction.set_xyz(*vdir.xyz())?;
    } else {
        resolved.offset = -vdir.magnitude();
        resolved.direction.set_xyz(*(-vdir).xyz())?;
    }
    Ok(())
}

// SetBasisCurve without the object's lock: copy the curve, unwrap trimmed and offset
// curves (folding the nested offsets), reject a C0 B-spline that is not G1, re-trim.
fn resolve_basis(
    curve: &Handle<dyn GeomCurve>,
    not_check_c0: bool,
    resolved: &mut BasisResolution,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let Some(source) = curve.get() else {
        return Err(errors::construction(
            "Geom_OffsetCurve: the basis curve is null; expected a curve",
        ));
    };
    let (uf, ul) = (source.first_parameter(), source.last_parameter());
    let mut checking: Handle<dyn GeomCurve> = source.copy()?.cast();
    let mut is_trimmed: bool = false;
    loop {
        let trimmed: Handle<GeomTrimmedCurve> = checking.cast();
        let offset: Handle<GeomOffsetCurve> = checking.cast();
        if trimmed.is_null() && offset.is_null() {
            break;
        }
        if let Some(trimmed) = trimmed.get() {
            checking = trimmed.basis_curve();
            is_trimmed = true;
        }
        let offset: Handle<GeomOffsetCurve> = checking.cast();
        if let Some(offset) = offset.get() {
            checking = offset.basis_curve();
            fold_nested_offset(offset, resolved)?;
        }
    }
    check_c0(&checking, not_check_c0, (uf, ul), resolved)?;
    if is_trimmed {
        let trimmed = GeomTrimmedCurve::new(&checking, uf, ul, true, true)?;
        return Ok(Handle::new(trimmed).cast());
    }
    Ok(checking)
}

fn check_c0(
    checking: &Handle<dyn GeomCurve>,
    not_check_c0: bool,
    range: (f64, f64),
    resolved: &mut BasisResolution,
) -> Result<(), Failure> {
    resolved.continuity = Some(checking.continuity());
    let is_c0: bool = !not_check_c0 && resolved.continuity == Some(Shape::C0);
    let bspline: Handle<GeomBSplineCurve> = checking.cast();
    let (true, Some(bspline)) = (is_c0, bspline.get()) else {
        return Ok(());
    };
    if bspline.is_g1(range.0, range.1, precision::angular())? {
        resolved.continuity = Some(Shape::G1);
        return Ok(());
    }
    Err(errors::construction(
        "Offset on C0 curve: the B-spline basis is not G1; expected a G1 basis",
    ))
}

impl GeomOffsetCurve {
    /// Offset of `curve` by `offset` along `D1 ^ direction`; the curve is copied,
    /// trimmed and offset inputs are unwrapped (nested offsets fold into one offset
    /// and direction), and a C0 B-spline basis must be G1 unless `not_check_c0`
    /// (`Geom_OffsetCurve(C, Offset, V, isNotCheckC0)`);
    /// `GeomOffsetCurve::new(&curve, 1.0, dir, false)?`.
    pub fn new(
        curve: &Handle<dyn GeomCurve>,
        offset: f64,
        direction: Direction3,
        not_check_c0: bool,
    ) -> Result<Self, Failure> {
        let mut resolved = BasisResolution {
            offset,
            direction,
            continuity: None,
        };
        let basis: Handle<dyn GeomCurve> = resolve_basis(curve, not_check_c0, &mut resolved)?;
        Ok(Self::stored(OffsetState {
            basis,
            eval_rep: Handle::default(),
            direction: resolved.direction,
            offset_value: resolved.offset,
            basis_continuity: resolved.continuity.unwrap_or(Shape::C0),
        }))
    }
    fn stored(state: OffsetState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }
    /// The native copy constructor: the basis and the evaluation representation are
    /// deep-copied (`Geom_OffsetCurve(const Geom_OffsetCurve&)`); `GeomOffsetCurve::copy_of(&c)?`.
    pub fn copy_of(other: &GeomOffsetCurve) -> Result<Self, Failure> {
        let source = other.state.read();
        Ok(Self::stored(OffsetState {
            basis: source.basis.copy()?.cast(),
            eval_rep: rep_utils::clone_curve_desc(&source.eval_rep)?,
            direction: source.direction,
            offset_value: source.offset_value,
            basis_continuity: source.basis_continuity,
        }))
    }
    rustycad_tkg2d::eval_representation_members!(
        dyn RepCurveDesc,
        rep_utils::validate_curve_desc,
        "curve"
    );
    /// Replace the basis curve, as the constructor does (`SetBasisCurve`). Like the
    /// native member, a failure leaves the folded offset, direction and continuity
    /// stored; `curve.set_basis_curve(&basis, false)?`.
    pub fn set_basis_curve(
        &self,
        curve: &Handle<dyn GeomCurve>,
        not_check_c0: bool,
    ) -> Result<(), Failure> {
        self.clear_eval_representation();
        let mut resolved = BasisResolution {
            offset: self.offset(),
            direction: self.direction(),
            continuity: None,
        };
        let result: Result<Handle<dyn GeomCurve>, Failure> =
            resolve_basis(curve, not_check_c0, &mut resolved);
        let mut state = self.state.write();
        state.offset_value = resolved.offset;
        state.direction = resolved.direction;
        if let Some(continuity) = resolved.continuity {
            state.basis_continuity = continuity;
        }
        state.basis = result?;
        Ok(())
    }
    /// Change the reference direction (`SetDirection`); `curve.set_direction(dir)`.
    pub fn set_direction(&self, direction: Direction3) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.direction = direction;
    }
    /// Change the offset distance (`SetOffsetValue`); `curve.set_offset_value(2.0)`.
    pub fn set_offset_value(&self, offset: f64) {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.offset_value = offset;
    }
    /// The basis curve (`BasisCurve`); `curve.basis_curve()`.
    pub fn basis_curve(&self) -> Handle<dyn GeomCurve> {
        self.state.read().basis.clone()
    }
    /// The reference direction (`Direction`); `curve.direction()`.
    pub fn direction(&self) -> Direction3 {
        self.state.read().direction
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
impl GeomGeometry for GeomOffsetCurve {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.transform(transform)?;
        state.direction.transform(transform);
        // Unlike Geom2d_OffsetCurve, the native code multiplies by the signed factor.
        state.offset_value *= transform.scale_factor();
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_OffsetCurve")?;
        dump_base(stream, depth, dump_curve)?;
        dump_field(stream, depth, "basisCurve", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        let direction: Direction3 = state.direction;
        dump_field(stream, depth, "direction", |nested, _| {
            direction.dump_json(nested)
        })?;
        dump::dump_real(stream, "offsetValue", state.offset_value)?;
        dump::dump_integer(
            stream,
            "BasisCurveContinuity",
            state.basis_continuity as i32,
        )
    }
}

fn undefined(member: &str) -> Failure {
    errors::undefined_derivative(&format!(
        "Geom_OffsetCurve::{member}: the offset normal is undefined; expected D1 ^ V not null"
    ))
}

impl GeomCurve for GeomOffsetCurve {
    fn reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.reverse()?;
        state.offset_value = -state.offset_value;
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.state.read().basis.reversed_parameter(u)
    }
    rustycad_tkg2d::basis_parameter_members!(Transform3);
    rustycad_tkg2d::basis_range_members!();
    fn is_closed(&self) -> Result<bool, Failure> {
        let first: Point3 = self.d0(self.first_parameter())?;
        let last: Point3 = self.d0(self.last_parameter())?;
        Ok(first.square_distance(&last) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        self.state.read().basis.is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        self.state.read().basis.period()
    }
    fn continuity(&self) -> Shape {
        offset_curve_continuity(self.state.read().basis_continuity)
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom_OffsetCurve::IsCN: N = {n}; expected N >= 0"
            )));
        }
        self.state.read().basis.is_cn(n + 1)
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        let state = self.state.read();
        if let Some(point) = rep_utils::try_eval_curve_d0(&state.eval_rep, u)? {
            return Ok(point);
        }
        let basis: GeomCurveD1 = state.basis.eval_d1(u)?;
        let mut value: Point3 = basis.point;
        let dir = state.direction.xyz();
        if !calculate_d0(&mut value, basis.d1, dir, state.offset_value, resolution()) {
            return Err(errors::undefined_value(
                "Geom_OffsetCurve::EvalD0: D1 ^ V is null; expected a regular point",
            ));
        }
        Ok(value)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d1(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: GeomCurveD2 = state.basis.eval_d2(u)?;
        let (mut point, mut d1) = (basis.point, basis.d1);
        let dir = state.direction.xyz();
        if !calculate_d1(
            &mut point,
            &mut d1,
            basis.d2,
            dir,
            state.offset_value,
            resolution(),
        ) {
            return Err(undefined("EvalD1"));
        }
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d2(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: GeomCurveD3 = state.basis.eval_d3(u)?;
        let (mut point, mut d1, mut d2, mut d3) = (basis.point, basis.d1, basis.d2, basis.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let mut dummy_d4 = Vector3::default();
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut dummy_d4];
            is_direction_change = adjust_derivative(&*state.basis, 3, u, derivatives)?;
        }
        let (dir, offset) = (state.direction.xyz(), state.offset_value);
        let derivatives = [&mut d1, &mut d2];
        if !calculate_d2(
            &mut point,
            derivatives,
            d3,
            dir,
            offset,
            is_direction_change,
            resolution(),
        ) {
            return Err(undefined("EvalD2"));
        }
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let state = self.state.read();
        if let Some(result) = rep_utils::try_eval_curve_d3(&state.eval_rep, u)? {
            return Ok(result);
        }
        let basis: GeomCurveD3 = state.basis.eval_d3(u)?;
        let mut d4: Vector3 = state.basis.eval_dn(u, 4)?;
        let (mut point, mut d1, mut d2, mut d3) = (basis.point, basis.d1, basis.d2, basis.d3);
        let mut is_direction_change: bool = false;
        if d1.square_magnitude() <= resolution() {
            let derivatives = [&mut d1, &mut d2, &mut d3, &mut d4];
            is_direction_change = adjust_derivative(&*state.basis, 4, u, derivatives)?;
        }
        let (dir, offset) = (state.direction.xyz(), state.offset_value);
        let derivatives = [&mut d1, &mut d2, &mut d3];
        if !calculate_d3(
            &mut point,
            derivatives,
            d4,
            dir,
            offset,
            is_direction_change,
            resolution(),
        ) {
            return Err(undefined("EvalD3"));
        }
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    rustycad_tkg2d::offset_curve_eval_dn!(Vector3, "Geom_OffsetCurve");
}
