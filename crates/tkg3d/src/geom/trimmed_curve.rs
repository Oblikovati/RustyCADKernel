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
// Rust translation of ModelingData/TKG3d/Geom/Geom_TrimmedCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_TrimmedCurve`: a copy of a basis curve limited to `[U1, U2]`.
use super::bounded_curve::{GeomBoundedCurve, dump_bounded_curve};
use super::curve::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3};
use super::errors;
use super::geometry::GeomGeometry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::SharedState;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::trim_bounds::{TrimBasis, TrimmedBounds};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::io;

type TrimmedState = TrimmedBounds<dyn GeomCurve>;

const CLASS: &str = "Geom_TrimmedCurve";

impl TrimBasis for dyn GeomCurve {
    fn trim_basis_range(&self) -> (f64, f64) {
        (self.first_parameter(), self.last_parameter())
    }
    fn trim_basis_periodic(&self) -> bool {
        self.is_periodic()
    }
    fn trim_basis_period(&self) -> Result<f64, Failure> {
        self.period()
    }
    fn trim_basis_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.reversed_parameter(u)
    }
    fn trim_basis_reverse(&self) -> Result<(), Failure> {
        self.reverse()
    }
}

/// A trimmed curve; `Handle::new(GeomTrimmedCurve::new(&circle.cast(), 0.0, PI, true, true)?)`.
///
/// The basis curve is a private copy, so its lock is never this object's lock.
pub struct GeomTrimmedCurve {
    parent: StandardTransient,
    state: SharedState<TrimmedState>,
}
impl_transient!(
    GeomTrimmedCurve,
    "Geom_TrimmedCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);

impl GeomTrimmedCurve {
    /// Trim a copy of `curve` to `[u1, u2]`; with `sense` false the result runs the
    /// other way, and `adjust_periodic` moves periodic bounds into the basis range
    /// (`Geom_TrimmedCurve(C, U1, U2, Sense, theAdjustPeriodic)`). A trimmed input
    /// contributes its own basis. Standard_ConstructionError for `u1 == u2` or bounds
    /// outside a non-periodic basis; a null curve, which the native constructor
    /// dereferences, is refused with Standard_ConstructionError;
    /// `GeomTrimmedCurve::new(&curve, 0.0, 1.0, true, true)?`.
    pub fn new(
        curve: &Handle<dyn GeomCurve>,
        u1: f64,
        u2: f64,
        sense: bool,
        adjust_periodic: bool,
    ) -> Result<Self, Failure> {
        let Some(source) = curve.get() else {
            return Err(errors::construction(
                "Geom_TrimmedCurve: the basis curve is null; expected a curve",
            ));
        };
        let trimmed: Handle<GeomTrimmedCurve> = curve.cast();
        let basis: Handle<dyn GeomCurve> = match trimmed.get() {
            Some(inner) => inner.basis_curve().copy()?.cast(),
            None => source.copy()?.cast(),
        };
        let mut state = TrimmedBounds {
            basis,
            u_trim1: u1,
            u_trim2: u2,
        };
        state.set_trim(u1, u2, sense, adjust_periodic, CLASS)?;
        Ok(Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        })
    }
    /// Change the bounds (`SetTrim`); see [`GeomTrimmedCurve::new`];
    /// `trimmed.set_trim(0.5, 1.0, true, true)?`.
    pub fn set_trim(
        &self,
        u1: f64,
        u2: f64,
        sense: bool,
        adjust_periodic: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .set_trim(u1, u2, sense, adjust_periodic, CLASS)
    }
    /// The basis curve, shared with this object (`BasisCurve`); `trimmed.basis_curve()`.
    pub fn basis_curve(&self) -> Handle<dyn GeomCurve> {
        self.state.read().basis.clone()
    }
}
impl GeomGeometry for GeomTrimmedCurve {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.basis.transform(transform)?;
        let u1: f64 = state.basis.transformed_parameter(state.u_trim1, transform);
        let u2: f64 = state.basis.transformed_parameter(state.u_trim2, transform);
        state.set_trim(u1, u2, true, false, CLASS)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        // The native Copy runs the constructor with the default Sense and
        // theAdjustPeriodic, so periodic bounds are adjusted again.
        let state = self.state.read();
        let copy = Self::new(&state.basis, state.u_trim1, state.u_trim2, true, true)?;
        Ok(Handle::new(copy).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_TrimmedCurve")?;
        dump_base(stream, depth, dump_bounded_curve)?;
        dump_field(stream, depth, "basisCurve", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        dump::dump_real(stream, "uTrim1", state.u_trim1)?;
        dump::dump_real(stream, "uTrim2", state.u_trim2)
    }
}
impl GeomCurve for GeomTrimmedCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.state.write().reverse(CLASS)
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.state.read().basis.reversed_parameter(u)
    }
    fn transformed_parameter(&self, u: f64, transform: Transform3) -> f64 {
        self.state.read().basis.transformed_parameter(u, transform)
    }
    fn parametric_transformation(&self, transform: Transform3) -> f64 {
        self.state.read().basis.parametric_transformation(transform)
    }
    fn first_parameter(&self) -> f64 {
        self.state.read().u_trim1
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().u_trim2
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        if self.state.read().spans_whole_periods()? {
            return Ok(true);
        }
        let start: Point3 = self.start_point()?;
        Ok(start.square_distance(&self.end_point()?) <= precision::computational())
    }
    fn is_periodic(&self) -> bool {
        // Period() of a periodic basis cannot fail.
        self.state.read().spans_whole_periods().unwrap_or(false)
    }
    fn period(&self) -> Result<f64, Failure> {
        self.state.read().basis.period()
    }
    fn continuity(&self) -> Shape {
        self.state.read().basis.continuity()
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        if n < 0 {
            return Err(errors::range(&format!(
                "Geom_TrimmedCurve::IsCN: N = {n}; expected N >= 0"
            )));
        }
        self.state.read().basis.is_cn(n)
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.state.read().basis.eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        self.state.read().basis.eval_d1(u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        self.state.read().basis.eval_d2(u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        self.state.read().basis.eval_d3(u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.state.read().basis.eval_dn(u, n)
    }
}
impl GeomBoundedCurve for GeomTrimmedCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        let state = self.state.read();
        state.basis.value(state.u_trim2)
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        let state = self.state.read();
        state.basis.value(state.u_trim1)
    }
}
