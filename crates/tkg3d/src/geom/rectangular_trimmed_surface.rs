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
// Rust translation of ModelingData/TKG3d/Geom/Geom_RectangularTrimmedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_RectangularTrimmedSurface`: a copy of a basis surface limited to a
//! parametric rectangle, or to a band in one direction.
mod trim;
use super::bounded_surface::GeomBoundedSurface;
use super::curve::GeomCurve;
use super::errors;
use super::geometry::GeomGeometry;
use super::offset_surface::GeomOffsetSurface;
use super::surface::{
    GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3, SurfaceBounds, check_dn_orders,
    dump_surface,
};
use super::trimmed_curve::GeomTrimmedCurve;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field, dump_flag};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{GeneralTransform2, Point3, Transform3, Vector3};
use std::io;
use trim::{DirectionTrim, TrimState};

/// A rectangular trimmed surface; the basis surface is a private copy;
/// `Handle::new(GeomRectangularTrimmedSurface::new(&s, 0.0, 1.0, 0.0, 1.0, true, true)?)`.
pub struct GeomRectangularTrimmedSurface {
    parent: StandardTransient,
    state: GeometryCell<TrimState>,
}
impl_transient!(
    GeomRectangularTrimmedSurface,
    "Geom_RectangularTrimmedSurface",
    dyn GeomBoundedSurface,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomSurface, dyn GeomBoundedSurface]
);

// The basis of both constructors: a copy of the surface (of the basis of a trimmed
// surface), with an offset basis rebuilt around a trimmed copy of its own basis by
// `trim_offset_basis`.
fn copied_basis(
    surface: &Handle<dyn GeomSurface>,
) -> Result<
    (
        Handle<dyn GeomSurface>,
        Handle<GeomRectangularTrimmedSurface>,
    ),
    Failure,
> {
    let Some(source) = surface.get() else {
        return Err(errors::construction(
            "Geom_RectangularTrimmedSurface: the basis surface is null; expected a surface",
        ));
    };
    let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    let basis: Handle<dyn GeomSurface> = match trimmed.get() {
        Some(inner) => inner.basis_surface().copy()?.cast(),
        None => source.copy()?.cast(),
    };
    Ok((basis, trimmed))
}

// `new Geom_OffsetSurface(S2, O->Offset(), true)` around a trimmed basis `S2`, when
// the copied basis is an offset surface.
fn trim_offset_basis(
    basis: Handle<dyn GeomSurface>,
    trim_basis: impl FnOnce(&Handle<dyn GeomSurface>) -> Result<GeomRectangularTrimmedSurface, Failure>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let offset: Handle<GeomOffsetSurface> = basis.cast();
    let Some(offset) = offset.get() else {
        return Ok(basis);
    };
    let inner: Handle<dyn GeomSurface> = Handle::new(trim_basis(&offset.basis_surface())?).cast();
    Ok(Handle::new(GeomOffsetSurface::new(&inner, offset.offset(), true)?).cast())
}

impl GeomRectangularTrimmedSurface {
    fn stored(state: TrimState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// Trim a copy of `surface` to `[u1, u2] x [v1, v2]`; the senses reverse the
    /// directions, periodic bounds are moved into the basis period, and an offset
    /// basis is rebuilt on a trimmed basis
    /// (`Geom_RectangularTrimmedSurface(S, U1, U2, V1, V2, USense, VSense)`);
    /// `GeomRectangularTrimmedSurface::new(&s, 0.0, 1.0, 0.0, 1.0, true, true)?`.
    pub fn new(
        surface: &Handle<dyn GeomSurface>,
        u: (f64, f64),
        v: (f64, f64),
        senses: (bool, bool),
    ) -> Result<Self, Failure> {
        let (basis, _) = copied_basis(surface)?;
        let basis = trim_offset_basis(basis, |inner| Self::new(inner, u, v, senses))?;
        let mut state = TrimState {
            basis,
            utrim1: u.0,
            vtrim1: v.0,
            utrim2: u.1,
            vtrim2: v.1,
            isutrimmed: true,
            isvtrimmed: true,
        };
        state.set_trim(
            DirectionTrim {
                p1: u.0,
                p2: u.1,
                trimmed: true,
                sense: senses.0,
            },
            DirectionTrim {
                p1: v.0,
                p2: v.1,
                trimmed: true,
                sense: senses.1,
            },
        )?;
        Ok(Self::stored(state))
    }
    /// Trim a copy of `surface` in U (`u_trim`) or in V only; trimming a trimmed
    /// surface keeps its bounds in the other direction
    /// (`Geom_RectangularTrimmedSurface(S, Param1, Param2, UTrim, Sense)`).
    ///
    /// Natively the trim fields start uninitialised; the OCCT allocator clears new
    /// objects (MMGT_CLEAR), so they start at 0 and false as here;
    /// `GeomRectangularTrimmedSurface::with_one_direction(&s, 0.0, 1.0, true, true)?`.
    pub fn with_one_direction(
        surface: &Handle<dyn GeomSurface>,
        p1: f64,
        p2: f64,
        u_trim: bool,
        sense: bool,
    ) -> Result<Self, Failure> {
        let (basis, trimmed) = copied_basis(surface)?;
        let basis = trim_offset_basis(basis, |inner| {
            Self::with_one_direction(inner, p1, p2, u_trim, sense)
        })?;
        let mut state = TrimState {
            basis,
            utrim1: 0.0,
            vtrim1: 0.0,
            utrim2: 0.0,
            vtrim2: 0.0,
            isutrimmed: false,
            isvtrimmed: false,
        };
        let request = DirectionTrim {
            p1,
            p2,
            trimmed: true,
            sense,
        };
        let (u, v) = one_direction_request(&state, &trimmed, request, u_trim);
        state.set_trim(u, v)?;
        Ok(Self::stored(state))
    }
    /// Retrim both directions (`SetTrim(U1, U2, V1, V2, USense, VSense)`); the U bounds
    /// are stored before a V failure; `s.set_trim(0.0, 1.0, 0.0, 2.0, true, true)?`.
    pub fn set_trim(
        &self,
        u1: f64,
        u2: f64,
        v1: f64,
        v2: f64,
        u_sense: bool,
        v_sense: bool,
    ) -> Result<(), Failure> {
        self.state.write().set_trim(
            DirectionTrim {
                p1: u1,
                p2: u2,
                trimmed: true,
                sense: u_sense,
            },
            DirectionTrim {
                p1: v1,
                p2: v2,
                trimmed: true,
                sense: v_sense,
            },
        )
    }
    /// Retrim one direction, keeping the other one as it is (`SetTrim(Param1, Param2,
    /// UTrim, Sense)`); `s.set_trim_direction(0.0, 1.0, true, true)?`.
    pub fn set_trim_direction(
        &self,
        p1: f64,
        p2: f64,
        u_trim: bool,
        sense: bool,
    ) -> Result<(), Failure> {
        let mut state = self.state.write();
        let request = DirectionTrim {
            p1,
            p2,
            trimmed: true,
            sense,
        };
        let (u, v) = one_direction_request(&state, &Handle::default(), request, u_trim);
        state.set_trim(u, v)
    }
    /// The basis surface, shared with this object (`BasisSurface`); `s.basis_surface()`.
    pub fn basis_surface(&self) -> Handle<dyn GeomSurface> {
        self.state.read().basis.clone()
    }
}

// The two directions requested by the one-direction constructor and SetTrim: the
// other direction keeps the bounds of `source` (a trimmed input of the constructor)
// or of `state` when trimmed, and is otherwise untrimmed with dummy bounds 0.
fn one_direction_request(
    state: &TrimState,
    source: &Handle<GeomRectangularTrimmedSurface>,
    request: DirectionTrim,
    u_trim: bool,
) -> (DirectionTrim, DirectionTrim) {
    let other: Option<(f64, f64)> = match source.get() {
        Some(inner) => inner.trimmed_bounds(!u_trim),
        None => state_bounds(state, !u_trim),
    };
    let (keep, sense) = match other {
        Some(bounds) => (bounds, true),
        None => ((0., 0.), true),
    };
    let kept = DirectionTrim {
        p1: keep.0,
        p2: keep.1,
        trimmed: other.is_some(),
        sense,
    };
    if u_trim {
        (request, kept)
    } else {
        (kept, request)
    }
}

fn state_bounds(state: &TrimState, u: bool) -> Option<(f64, f64)> {
    if u {
        return state.isutrimmed.then_some((state.utrim1, state.utrim2));
    }
    state.isvtrimmed.then_some((state.vtrim1, state.vtrim2))
}

impl GeomRectangularTrimmedSurface {
    fn trimmed_bounds(&self, u: bool) -> Option<(f64, f64)> {
        state_bounds(&self.state.read(), u)
    }
}

// Whether a trimmed length is a whole number of basis periods.
fn whole_periods(length: f64, period: f64) -> bool {
    length > precision::p_confusion()
        && rustycad_cmath::remainder(length, period).abs() <= precision::p_confusion()
}

impl GeomGeometry for GeomRectangularTrimmedSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.basis.transform(transform)?;
        (state.utrim1, state.vtrim1) =
            state
                .basis
                .transform_parameters(state.utrim1, state.vtrim1, transform);
        (state.utrim2, state.vtrim2) =
            state
                .basis
                .transform_parameters(state.utrim2, state.vtrim2, transform);
        Ok(())
    }
    /// The native Copy re-runs the constructor that matches the trimmed directions,
    /// and returns a null handle when neither is trimmed (not reachable through the
    /// constructors).
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let state = self.state.read();
        let (u, v) = ((state.utrim1, state.utrim2), (state.vtrim1, state.vtrim2));
        let copy = match (state.isutrimmed, state.isvtrimmed) {
            (true, true) => Self::new(&state.basis, u, v, (true, true))?,
            (true, false) => Self::with_one_direction(&state.basis, u.0, u.1, true, true)?,
            (false, true) => Self::with_one_direction(&state.basis, v.0, v.1, false, true)?,
            (false, false) => return Ok(Handle::default()),
        };
        Ok(Handle::new(copy).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let state = self.state.read();
        dump_class_name(stream, "Geom_RectangularTrimmedSurface")?;
        // Geom_BoundedSurface inherits Geom_Surface::DumpJson, whose class name is
        // "Geom_Surface".
        dump_base(stream, depth, dump_surface)?;
        dump_field(stream, depth, "basisSurf", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        let reals = [
            ("utrim1", state.utrim1),
            ("vtrim1", state.vtrim1),
            ("utrim2", state.utrim2),
            ("vtrim2", state.vtrim2),
        ];
        for (name, value) in reals {
            dump::dump_real(stream, name, value)?;
        }
        dump_flag(stream, "isutrimmed", state.isutrimmed)?;
        dump_flag(stream, "isvtrimmed", state.isvtrimmed)
    }
}

impl GeomSurface for GeomRectangularTrimmedSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        self.state.write().u_reverse()
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.state.read().basis.u_reversed_parameter(u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        self.state.write().v_reverse()
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        self.state.read().basis.v_reversed_parameter(v)
    }
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        self.state
            .read()
            .basis
            .transform_parameters(u, v, transform)
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        self.state.read().basis.parametric_transformation(transform)
    }
    fn bounds(&self) -> SurfaceBounds {
        let state = self.state.read();
        (state.utrim1, state.utrim2, state.vtrim1, state.vtrim2)
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.isutrimmed {
            return state.basis.is_u_closed();
        }
        Ok(state.basis.is_u_periodic()
            && whole_periods(state.utrim2 - state.utrim1, state.basis.u_period()?))
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.isvtrimmed {
            return state.basis.is_v_closed();
        }
        Ok(state.basis.is_v_periodic()
            && whole_periods(state.vtrim2 - state.vtrim1, state.basis.v_period()?))
    }
    fn is_u_periodic(&self) -> bool {
        let state = self.state.read();
        if !state.basis.is_u_periodic() {
            return false;
        }
        // UPeriod() of a periodic basis cannot fail.
        !state.isutrimmed
            || state
                .basis
                .u_period()
                .is_ok_and(|period| whole_periods(state.utrim2 - state.utrim1, period))
    }
    fn u_period(&self) -> Result<f64, Failure> {
        self.state.read().basis.u_period()
    }
    fn is_v_periodic(&self) -> bool {
        let state = self.state.read();
        if !state.basis.is_v_periodic() {
            return false;
        }
        !state.isvtrimmed
            || state
                .basis
                .v_period()
                .is_ok_and(|period| whole_periods(state.vtrim2 - state.vtrim1, period))
    }
    fn v_period(&self) -> Result<f64, Failure> {
        self.state.read().basis.v_period()
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state = self.state.read();
        let curve: Handle<dyn GeomCurve> = state.basis.u_iso(u)?;
        if !state.isvtrimmed {
            return Ok(curve);
        }
        Ok(Handle::new(GeomTrimmedCurve::new(
            &curve,
            state.vtrim1,
            state.vtrim2,
            true,
            true,
        )?)
        .cast())
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        let state = self.state.read();
        let curve: Handle<dyn GeomCurve> = state.basis.v_iso(v)?;
        if !state.isutrimmed {
            return Ok(curve);
        }
        Ok(Handle::new(GeomTrimmedCurve::new(
            &curve,
            state.utrim1,
            state.utrim2,
            true,
            true,
        )?)
        .cast())
    }
    fn continuity(&self) -> Shape {
        self.state.read().basis.continuity()
    }
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        check_order("IsCNu", n)?;
        self.state.read().basis.is_cn_u(n)
    }
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        check_order("IsCNv", n)?;
        self.state.read().basis.is_cn_v(n)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.state.read().basis.eval_d0(u, v)
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        self.state.read().basis.eval_d1(u, v)
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        self.state.read().basis.eval_d2(u, v)
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        self.state.read().basis.eval_d3(u, v)
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_RectangularTrimmedSurface", nu, nv)?;
        self.state.read().basis.eval_dn(u, v, nu, nv)
    }
}
impl GeomBoundedSurface for GeomRectangularTrimmedSurface {}

fn check_order(member: &str, n: i32) -> Result<(), Failure> {
    if n < 0 {
        return Err(errors::range(&format!(
            "Geom_RectangularTrimmedSurface::{member}: N = {n}; expected N >= 0"
        )));
    }
    Ok(())
}
