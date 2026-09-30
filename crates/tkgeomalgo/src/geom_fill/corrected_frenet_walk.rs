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
// Rust translation of the sampling loop of GeomFill_CorrectedFrenet::InitInterval
// (ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CorrectedFrenet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The adaptive walk of `InitInterval`: from one Frenet frame to the next with a
//! step bounded by the variation of `C' ^ C''`, halved while the tangent turns by
//! more than PI/3, accumulating the transported angle and the average frame.
use super::corrected_frenet::GeomFillCorrectedFrenet;
use super::corrected_frenet_init::{IntervalWalk, calc_angle_at};
use super::frame_math::std_max;
use super::snglr_func::GeomFillSnglrFunc;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::f64::consts::PI;

/// The samples of one interval: the parameters, the accumulated angles, and the
/// `isZero` and `isConst` flags.
pub(crate) type IntervalSteps = (Vec<f64>, Vec<f64>, bool, bool);

/// The loop variables of the walk (`Param`, `currParam`, `currStep`, `i`, the flags).
struct WalkCursor {
    param: f64,
    curr_param: f64,
    curr_step: f64,
    i: usize,
    is_zero: bool,
    is_const: bool,
}

impl GeomFillCorrectedFrenet {
    /// The `while (Param < Last)` loop of `InitInterval`; `bounds` is
    /// `[First, Last, Step, LengthMin]`.
    pub(crate) fn walk_interval(
        &self,
        cs: &GeomFillSnglrFunc,
        bounds: [f64; 4],
        is_planar: bool,
        walk: &mut IntervalWalk,
    ) -> Result<IntervalSteps, Failure> {
        let [first, last, step, length_min] = bounds;
        let d_last = last - precision::p_confusion();
        let mut c = WalkCursor {
            param: first,
            curr_param: first,
            curr_step: step,
            i: 1,
            is_zero: true,
            is_const: true,
        };
        let (mut parameters, mut evol_at) = (Vec::new(), Vec::new());
        let mut frame = TrihedronD0::default();
        while c.param < last {
            if c.curr_param > d_last {
                if (d_last - c.param).abs() < precision::square_confusion() {
                    c.param = c.curr_param;
                }
                c.curr_step = d_last - c.param;
                c.curr_param = last;
            }
            if is_planar {
                c.curr_param = last;
            }
            self.frenet().d0(c.curr_param, &mut frame)?;
            if walk.prev_tangent.angle(frame.tangent)? < PI / 3.0 || c.i == 1 {
                accept_step(&mut c, &frame, walk, &mut parameters, &mut evol_at)?;
                c.curr_step = next_step(cs, c.param, length_min, step)?;
            } else {
                c.curr_step /= 2.0;
            }
            c.curr_param = c.param + c.curr_step;
        }
        Ok((parameters, evol_at, c.is_zero, c.is_const))
    }
}

/// Records one accepted step: the parameters, the previous frame and angle, the new
/// accumulated angle and the running averages.
fn accept_step(
    c: &mut WalkCursor,
    frame: &TrihedronD0,
    walk: &mut IntervalWalk,
    parameters: &mut Vec<f64>,
    evol_at: &mut Vec<f64>,
) -> Result<(), Failure> {
    let previous = if c.i > 1 {
        evol_at[c.i - 2]
    } else {
        walk.start_ang
    };
    parameters.push(c.curr_param);
    walk.history.poles.push(c.param);
    walk.history.angle.push(previous);
    walk.history.tangent.push(walk.prev_tangent);
    walk.history.normal.push(walk.prev_normal);
    let mut angle_at = calc_angle_at(
        frame.tangent,
        frame.normal,
        walk.prev_tangent,
        walk.prev_normal,
    )?;
    if c.is_const && c.i > 1 && angle_at.abs() > precision::p_confusion() {
        c.is_const = false;
    }
    angle_at += previous;
    evol_at.push(angle_at);
    walk.prev_normal = frame.normal;
    if c.is_zero && angle_at.abs() > precision::p_confusion() {
        c.is_zero = false;
    }
    walk.at += frame.tangent;
    let cross = frame.tangent.crossed(frame.normal);
    let offset = frame.normal + walk.an;
    walk.an.set_linear_form_offset(
        angle_at.sin(),
        cross,
        1.0 - angle_at.cos(),
        frame.tangent.crossed(cross),
        offset,
    );
    walk.prev_tangent = frame.tangent;
    c.param = c.curr_param;
    c.i += 1;
    Ok(())
}

/// The next step: `max(|CS(Param)| / 2, LengthMin) / |CS'(Param)|`, at most `step`.
fn next_step(
    cs: &GeomFillSnglrFunc,
    param: f64,
    length_min: f64,
    step: f64,
) -> Result<f64, Failure> {
    let (ponc, d1) = cs.d1(param)?;
    let l = std_max(ponc.xyz().modulus() / 2.0, length_min);
    let mut norm = d1.magnitude();
    if norm < precision::confusion() {
        norm = precision::confusion();
    }
    let curr_step = l / norm;
    if curr_step > step {
        return Ok(step);
    }
    Ok(curr_step)
}
