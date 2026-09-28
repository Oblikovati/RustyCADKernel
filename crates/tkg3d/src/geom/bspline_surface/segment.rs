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
// Rust translation of ModelingData/TKG3d/Geom/Geom_BSplineSurface.cxx (segment, Segment, CheckAndSegment) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The protected `segment` behind `Segment` and `CheckAndSegment`.
use super::GeomBSplineSurface;
use super::state::{Direction, SurfaceState, cell, grid, put_cell};
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, real};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// One direction of a segment request: the bounds, the tolerance and whether to segment.
#[derive(Clone, Copy)]
pub(super) struct SegmentRequest {
    pub(super) p1: f64,
    pub(super) p2: f64,
    pub(super) eps: f64,
    pub(super) segment: bool,
}

// The knots kept by one direction: the knot index range and the new knot arrays.
struct KeptKnots {
    first: i32,
    last: i32,
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
}

// `BSplCLib::LocateParameter` over the whole knot array, with a zero start index.
fn locate(direction: &Direction, param: f64) -> Result<(i32, f64), Failure> {
    let (mut index, mut located) = (0, param);
    let knots: &Array1<'static, f64> = &direction.knots;
    bsplclib::locate_parameter(
        direction.deg,
        knots,
        &direction.mults,
        param,
        direction.periodic,
        knots.lower(),
        knots.upper(),
        &mut index,
        &mut located,
    )?;
    Ok((index, located))
}

// The length of the kept range: `p2 - p1`, Standard_DomainError beyond one period of a
// periodic direction and clamped to it.
fn delta(direction: &Direction, request: SegmentRequest) -> Result<f64, Failure> {
    let span: f64 = request.p2 - request.p1;
    if !direction.periodic {
        return Ok(span);
    }
    let knots: &Array1<'static, f64> = &direction.knots;
    let period: f64 = at(knots, knots.upper())? - at(knots, knots.lower())?;
    if span - period > precision::p_confusion() {
        return Err(errors::domain(&format!(
            "Geom_BSplineSurface::Segment: range {span} is longer than the period {period}; \
             expected at most one period"
        )));
    }
    Ok(if span > period { period } else { span })
}

impl SurfaceState {
    /// The protected `segment(U1, U2, V1, V2, EpsU, EpsV, SegmentInU, SegmentInV)`.
    pub(super) fn segment(&mut self, u: SegmentRequest, v: SegmentRequest) -> Result<(), Failure> {
        self.clear_eval_rep();
        let delta_u: f64 = delta(&self.u, u)?;
        let delta_v: f64 = delta(&self.v, v)?;
        let new_u1: f64 = self.insert_segment_knots(true, u)?;
        let new_v1: f64 = self.insert_segment_knots(false, v)?;
        self.set_segment_origin(true, u)?;
        let kept_u: KeptKnots = self.kept_knots(true, new_u1, delta_u, u)?;
        self.set_segment_origin(false, v)?;
        let kept_v: KeptKnots = self.kept_knots(false, new_v1, delta_v, v)?;
        self.keep_poles(&kept_u, &kept_v)?;
        let (u_knots, v_knots) = (kept_u, kept_v);
        self.u.knots = u_knots.knots;
        self.u.mults = u_knots.mults;
        self.v.knots = v_knots.knots;
        self.v.mults = v_knots.mults;
        self.max_deriv_inv_ok = false;
        self.update_knots(true)?;
        self.update_knots(false)
    }
    // Locate both bounds and, when segmenting, insert them with multiplicity Deg;
    // returns the located first bound.
    fn insert_segment_knots(&mut self, u: bool, request: SegmentRequest) -> Result<f64, Failure> {
        let direction: &Direction = self.dir(u);
        let (_, new1) = locate(direction, request.p1)?;
        let (_, new2) = locate(direction, request.p2)?;
        if request.segment {
            let (mut knots, mut mults) = (one_based::<f64>(2)?, one_based::<i32>(2)?);
            put(&mut knots, 1, std_min(new1, new2))?;
            put(&mut knots, 2, std_max(new1, new2))?;
            mults.init(direction.deg);
            self.insert_knots(u, &knots, &mults, request.eps, true)?;
        }
        Ok(new1)
    }
    // A periodic segmented direction gets its origin at the first bound and becomes
    // non-periodic.
    fn set_segment_origin(&mut self, u: bool, request: SegmentRequest) -> Result<(), Failure> {
        if !(self.dir(u).periodic && request.segment) {
            return Ok(());
        }
        let (mut index, located) = locate(self.dir(u), request.p1)?;
        if (at(&self.dir(u).knots, index + 1)? - located).abs() <= request.eps {
            index += 1;
        }
        self.set_origin(u, index)?;
        self.set_not_periodic(u)
    }
    // The knot range from `new1` to `new1 + delta`; a segmented direction gets end
    // multiplicities Deg + 1.
    fn kept_knots(
        &self,
        u: bool,
        new1: f64,
        delta: f64,
        request: SegmentRequest,
    ) -> Result<KeptKnots, Failure> {
        let direction: &Direction = self.dir(u);
        let (mut first, located) = locate(direction, new1)?;
        if (at(&direction.knots, first + 1)? - located).abs() <= request.eps {
            first += 1;
        }
        let (mut last, located) = locate(direction, new1 + delta)?;
        if (at(&direction.knots, last + 1)? - located).abs() <= request.eps || last == first {
            last += 1;
        }
        let count: i32 = last - first + 1;
        let (mut knots, mut mults) = (one_based::<f64>(count)?, one_based::<i32>(count)?);
        for (k, i) in (1..).zip(first..=last) {
            put(&mut knots, k, at(&direction.knots, i)?)?;
            put(&mut mults, k, at(&direction.mults, i)?)?;
        }
        if request.segment {
            put(&mut mults, 1, direction.deg + 1)?;
            put(&mut mults, count, direction.deg + 1)?;
        }
        Ok(KeptKnots {
            first,
            last,
            knots,
            mults,
        })
    }
    // The pole range of the kept knots: `PoleIndex + 1` to `min(PoleIndex + 1, count)`.
    fn kept_poles(&self, u: bool, kept: &KeptKnots) -> Result<(i32, i32), Failure> {
        let direction: &Direction = self.dir(u);
        let first: i32 = bsplclib::pole_index(
            direction.deg,
            kept.first,
            direction.periodic,
            &direction.mults,
        )?;
        let last: i32 = bsplclib::pole_index(
            direction.deg,
            kept.last,
            direction.periodic,
            &direction.mults,
        )?;
        Ok((first + 1, (last + 1).min(self.nb_poles(u))))
    }
    // Keep the poles (and weights) of both kept ranges.
    fn keep_poles(&mut self, kept_u: &KeptKnots, kept_v: &KeptKnots) -> Result<(), Failure> {
        let rows: (i32, i32) = self.kept_poles(true, kept_u)?;
        let cols: (i32, i32) = self.kept_poles(false, kept_v)?;
        let poles: Array2<'static, Point3> = sub_grid(&self.poles, rows, cols)?;
        self.weights = if self.rational() {
            sub_grid(&self.weights, rows, cols)?
        } else {
            super::state::unit_weights(
                (rows.1 - rows.0 + 1) as usize,
                (cols.1 - cols.0 + 1) as usize,
            )?
        };
        self.poles = poles;
        Ok(())
    }
}

// Rows `rows.0..=rows.1` and columns `cols.0..=cols.1`, re-indexed from 1.
fn sub_grid<T: Copy + Default>(
    source: &Array2<'_, T>,
    rows: (i32, i32),
    cols: (i32, i32),
) -> Result<Array2<'static, T>, Failure> {
    let mut result: Array2<'static, T> = grid(rows.1 - rows.0 + 1, cols.1 - cols.0 + 1)?;
    for (k, i) in (1..).zip(rows.0..=rows.1) {
        for (l, j) in (1..).zip(cols.0..=cols.1) {
            put_cell(&mut result, k, l, cell(source, i, j)?)?;
        }
    }
    Ok(result)
}

impl GeomBSplineSurface {
    /// Restrict to `[u1, u2] x [v1, v2]`; Standard_DomainError for reversed bounds
    /// (`Segment`); `surface.segment(0.2, 0.8, 0.1, 0.9, 1e-9, 1e-9)?`.
    pub fn segment(
        &self,
        u: (f64, f64),
        v: (f64, f64),
        tolerances: (f64, f64),
    ) -> Result<(), Failure> {
        let (u_request, v_request) = segment_requests("Segment", u, v, tolerances)?;
        self.state.write().segment(u_request, v_request)
    }
    /// `Segment` restricted to the directions whose bounds differ from the end knots
    /// by more than the tolerance (`CheckAndSegment`); `surface.check_and_segment((0.0, 1.0), (0.2, 0.8), (1e-9, 1e-9))?`.
    pub fn check_and_segment(
        &self,
        u: (f64, f64),
        v: (f64, f64),
        tolerances: (f64, f64),
    ) -> Result<(), Failure> {
        let (mut u_request, mut v_request) = segment_requests("CheckAndSegment", u, v, tolerances)?;
        let mut state = self.state.write();
        u_request.segment = moves_bounds(&state.u, u, u_request.eps)?;
        v_request.segment = moves_bounds(&state.v, v, v_request.eps)?;
        state.segment(u_request, v_request)
    }
}

// The checks and tolerances of Segment and CheckAndSegment: Standard_DomainError for
// reversed bounds, then `max(Epsilon(max(|P1|, |P2|)), Tolerance)` per direction.
fn segment_requests(
    member: &str,
    u: (f64, f64),
    v: (f64, f64),
    tolerances: (f64, f64),
) -> Result<(SegmentRequest, SegmentRequest), Failure> {
    if u.1 < u.0 || v.1 < v.0 {
        return Err(errors::domain(&format!(
            "Geom_BSplineSurface::{member}: U [{}, {}], V [{}, {}]; expected U1 <= U2 and V1 <= V2",
            u.0, u.1, v.0, v.1
        )));
    }
    let eps = |p: (f64, f64), tolerance: f64| {
        let max: f64 = rustycad_tkg2d::geom2d::native::std_max(p.1.abs(), p.0.abs());
        rustycad_tkg2d::geom2d::native::std_max(real::epsilon(max), tolerance)
    };
    let u_request = SegmentRequest {
        p1: u.0,
        p2: u.1,
        eps: eps(u, tolerances.0),
        segment: true,
    };
    let v_request = SegmentRequest {
        p1: v.0,
        p2: v.1,
        eps: eps(v, tolerances.1),
        segment: true,
    };
    Ok((u_request, v_request))
}

// Whether CheckAndSegment segments a direction: a bound farther than `eps` from the
// first or last knot.
fn moves_bounds(direction: &Direction, p: (f64, f64), eps: f64) -> Result<bool, Failure> {
    let knots: &Array1<'static, f64> = &direction.knots;
    let first: f64 = rustycad_tkg2d::geom2d::arrays::at(knots, knots.lower())?;
    let last: f64 = rustycad_tkg2d::geom2d::arrays::at(knots, knots.upper())?;
    Ok((p.0 - first).abs() > eps || (p.1 - last).abs() > eps)
}
