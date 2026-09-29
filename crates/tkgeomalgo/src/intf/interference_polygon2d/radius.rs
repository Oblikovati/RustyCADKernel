// Created on: 1991-06-24
// Created by: Didier PIFFAULT
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the near-parallel part of Intf_InterferencePolygon2d::Intersect of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygon2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The near-parallel part of Intf_InterferencePolygon2d::Intersect: the ends of the
//! interference radius around the crossing of two segments become EXTERNAL points.
use super::intersect::{Candidates, Crossing, SegmentPair};
use crate::intf::{std_max, std_min};
use rustycad_tkmath::gp::{Point2, Point3};

/// The ends of the interference radius around the crossing, on both segments.
#[derive(Clone, Copy, Debug)]
pub(super) struct RadiusEnds {
    par_odeb: f64,
    par_ofin: f64,
    par_tdeb: f64,
    par_tfin: f64,
    delta_o: f64,
    delta_t: f64,
    sig_ps: f64,
}

impl RadiusEnds {
    pub(super) fn new(pair: &SegmentPair, crossing: Crossing, ray_intf: f64, sig_ps: f64) -> Self {
        let delta_o: f64 = ray_intf / pair.lg_o;
        let delta_t: f64 = ray_intf / pair.lg_t;
        Self {
            par_odeb: crossing.par_o - delta_o,
            par_ofin: crossing.par_o + delta_o,
            par_tdeb: crossing.par_t - sig_ps * delta_t,
            par_tfin: crossing.par_t + sig_ps * delta_t,
            delta_o,
            delta_t,
            sig_ps,
        }
    }
}

/// Near-parallel segments: the ends of the interference radius become EXTERNAL points
/// (both with no contact, the one far enough from a single contact, or those beyond
/// the parameter span of several contacts).
pub(super) fn radius_candidates(
    pair: &SegmentPair,
    tolerance: f64,
    ends: RadiusEnds,
    candidates: &mut Candidates,
) {
    match candidates.nbpi {
        0 => {
            candidates.push(
                ends.par_odeb,
                ends.par_tdeb,
                pair.external(ends.par_odeb, ends.par_tdeb),
            );
            candidates.push(
                ends.par_ofin,
                ends.par_tfin,
                pair.external(ends.par_ofin, ends.par_tfin),
            );
        }
        1 => second_radius_point(pair, tolerance, ends, candidates),
        _ => span_radius_points(pair, clamp_radius_ends(ends), candidates),
    }
}

/// One contact: the first radius end inside both segments is added when it is at
/// least a quarter of the tolerance away from the contact.
fn second_radius_point(
    pair: &SegmentPair,
    tolerance: f64,
    ends: RadiusEnds,
    candidates: &mut Candidates,
) {
    let inside = |po: f64, pt: f64| 0.0 < po && po < 1.0 && 0.0 < pt && pt < 1.0;
    let (par_o, par_t) = if inside(ends.par_odeb, ends.par_tdeb) {
        (ends.par_odeb, ends.par_tdeb)
    } else if inside(ends.par_ofin, ends.par_tfin) {
        (ends.par_ofin, ends.par_tfin)
    } else {
        return;
    };
    // The native code stores the parameters before the distance test.
    candidates.par_o[2] = par_o;
    candidates.par_t[2] = par_t;
    let at: Point2 = pair.on_first(par_o);
    let gap: f64 = candidates
        .at(1)
        .pnt()
        .distance(&Point3::new(at.x(), at.y(), 0.0));
    if gap >= (tolerance / 4.0) {
        candidates.push(par_o, par_t, pair.external(par_o, par_t));
    }
}

/// Clips the radius ends to the segments, sliding the other parameter along.
fn clamp_radius_ends(mut e: RadiusEnds) -> RadiusEnds {
    if e.par_odeb < 0.0 {
        e.par_tdeb += e.sig_ps * (-e.par_odeb * (e.delta_t / e.delta_o));
        e.par_odeb = 0.0;
    }
    if e.par_ofin > 1.0 {
        e.par_tfin -= e.sig_ps * ((e.par_ofin - 1.0) * (e.delta_t / e.delta_o));
        e.par_ofin = 1.0;
    }
    let (low, high) = if e.sig_ps > 0.0 {
        (0.0, 1.0)
    } else {
        (1.0, 0.0)
    };
    let beyond = |value: f64, limit: f64| {
        if limit == 0.0 {
            value < 0.0
        } else {
            value > 1.0
        }
    };
    if beyond(e.par_tdeb, low) {
        e.par_odeb += (e.par_tdeb - low).abs() * (e.delta_o / e.delta_t);
        e.par_tdeb = low;
    }
    if beyond(e.par_tfin, high) {
        e.par_ofin -= (e.par_tfin - high).abs() * (e.delta_o / e.delta_t);
        e.par_tfin = high;
    }
    e
}

/// Several contacts: a clipped radius end beyond their parameter span is added.
fn span_radius_points(pair: &SegmentPair, e: RadiusEnds, candidates: &mut Candidates) {
    let n: usize = candidates.nbpi;
    let span = |values: &[f64; 8]| {
        let mut bounds = (values[1], values[1]);
        for &value in &values[2..=n] {
            bounds = (std_min(bounds.0, value), std_max(bounds.1, value));
        }
        bounds
    };
    let ((o_min, o_max), (t_min, t_max)) = (span(&candidates.par_o), span(&candidates.par_t));
    let forward: bool = e.sig_ps > 0.0;
    if (e.par_odeb < o_min && o_min > 0.0)
        || (forward && e.par_tdeb < t_min && t_min > 0.0)
        || (e.sig_ps < 0.0 && e.par_tdeb > t_max && t_max < 1.0)
    {
        let (po, pt) = (
            std_max(0.0, std_min(1.0, e.par_odeb)),
            std_max(0.0, std_min(1.0, e.par_tdeb)),
        );
        candidates.push(po, pt, pair.external(po, pt));
    }
    if (e.par_ofin > o_max && o_max < 1.0)
        || (e.sig_ps < 0.0 && e.par_tfin < t_min && t_min > 0.0)
        || (forward && e.par_tfin > t_max && t_max < 1.0)
    {
        let (po, pt) = (
            std_min(1.0, std_max(0.0, e.par_ofin)),
            std_min(1.0, std_max(0.0, e.par_tfin)),
        );
        candidates.push(po, pt, pair.external(po, pt));
    }
}
