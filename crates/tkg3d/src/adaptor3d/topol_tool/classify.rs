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
// Rust translation of ModelingData/TKG3d/Adaptor3d/Adaptor3d_TopolTool.cxx (Classify, IsThePointOn) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Classification of a parametric point against the rectangle of the restrictions.
use super::TopolState;
use crate::top_abs::State;
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::Point2;

/// Where a coordinate lies along one side of the rectangle: inside the range (bounds
/// included within the tolerance) and on the lower or upper bound.
#[derive(Clone, Copy, Default)]
struct AxisPosition {
    inside: bool,
    on_min: bool,
    on_max: bool,
}

/// How a point of the whole rectangle is placed: outside, on a side, or inside.
enum Placement {
    Out,
    On,
    In,
}

// One axis of the general case (the `dans*`, `sur*min`, `sur*max` flags). Natively
// only U tests `>= Usup + Tol` on a range infinite below; V tests `>`.
fn axis_position(
    value: f64,
    (lo, hi): (f64, f64),
    tolerance: f64,
    inclusive_upper: bool,
) -> AxisPosition {
    let (below_infinite, above_infinite) = (
        precision::is_negative_infinite(lo),
        precision::is_positive_infinite(hi),
    );
    let on_min: bool = (value - lo).abs() <= tolerance;
    let on_max: bool = (value - hi).abs() <= tolerance;
    match (below_infinite, above_infinite) {
        (true, true) => AxisPosition {
            inside: true,
            ..Default::default()
        },
        (true, false) => {
            let beyond: bool = if inclusive_upper {
                value >= hi + tolerance
            } else {
                value > hi + tolerance
            };
            AxisPosition {
                inside: !beyond,
                on_max: !beyond && on_max,
                on_min: false,
            }
        }
        (false, true) => {
            // Written as the native else-branch so that a NaN coordinate is inside.
            let below: bool = value < lo - tolerance;
            let inside: bool = !below;
            AxisPosition {
                inside,
                on_min: inside && on_min,
                on_max: false,
            }
        }
        (false, false) => {
            let inside: bool = !(value < lo - tolerance || value > hi + tolerance);
            let on_min: bool = inside && on_min;
            AxisPosition {
                inside,
                on_min,
                on_max: inside && !on_min && on_max,
            }
        }
    }
}

// The placement of a point: the four-sided fast test, or the axis flags.
fn placement(state: &TopolState, point: Point2, tolerance: f64) -> Option<Placement> {
    let (u, v) = (point.x(), point.y());
    let (uinf, usup, vinf, vsup) = (state.uinf, state.usup, state.vinf, state.vsup);
    if state.nb_restr == 4 {
        if u < uinf - tolerance
            || u > usup + tolerance
            || v < vinf - tolerance
            || v > vsup + tolerance
        {
            return Some(Placement::Out);
        }
        let on: bool = (u - uinf).abs() <= tolerance
            || (u - usup).abs() <= tolerance
            || (v - vinf).abs() <= tolerance
            || (v - vsup).abs() <= tolerance;
        return Some(if on { Placement::On } else { Placement::In });
    }
    if state.nb_restr == 0 {
        return None;
    }
    let along_u = axis_position(u, (uinf, usup), tolerance, true);
    let along_v = axis_position(v, (vinf, vsup), tolerance, false);
    if !along_u.inside || !along_v.inside {
        return Some(Placement::Out);
    }
    let on: bool = along_u.on_min || along_v.on_min || along_u.on_max || along_v.on_max;
    Some(if on { Placement::On } else { Placement::In })
}

/// `Classify`: OUT, ON or IN; IN for a rectangle without restrictions;
/// `classify(&state, point, 1e-7)`.
pub(super) fn classify(state: &TopolState, point: Point2, tolerance: f64) -> State {
    match placement(state, point, tolerance) {
        Some(Placement::Out) => State::Out,
        Some(Placement::On) => State::On,
        Some(Placement::In) | None => State::In,
    }
}

/// `IsThePointOn`: whether the point is on a side; false without restrictions;
/// `is_the_point_on(&state, point, 1e-7)`.
pub(super) fn is_the_point_on(state: &TopolState, point: Point2, tolerance: f64) -> bool {
    matches!(placement(state, point, tolerance), Some(Placement::On))
}
