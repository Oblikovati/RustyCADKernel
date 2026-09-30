// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomFill_GordonUtilities.pxx (OrderedCurve, orderedCurves,
// orientedCurve, orientedParameter, orderedContactScore, seamTangentMismatch,
// chooseOrderedContact) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ordering of a Gordon network: curve ranks, orientations and the choice of one
//! contact per pair.
use super::gordon_contacts::{Contact, ContactSet, curve_bounds};
use super::gordon_params::{null_curve, parameter_resolution};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::stl;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomGeometry};
use rustycad_tkmath::gp;

/// `OrderedCurve`: a curve index and its rank.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct OrderedCurve {
    pub(crate) index: usize,
    pub(crate) rank: f64,
}

/// `orderedCurves(Ranks)`: the curves sorted by rank (libstdc++ `std::sort`); when the
/// two lowest (or highest) ranks are within parameterResolution(0, 1), the second
/// (the second to last) curve moves to the other end, which opens a closed family.
pub(crate) fn ordered_curves(the_ranks: &[f64]) -> Vec<OrderedCurve> {
    let mut an_order: Vec<OrderedCurve> = the_ranks
        .iter()
        .enumerate()
        .map(|(index, rank)| OrderedCurve { index, rank: *rank })
        .collect();
    stl::sort_by(&mut an_order, |left, right| left.rank < right.rank);
    let a_tol: f64 = parameter_resolution(0.0, 1.0);
    let n: usize = an_order.len();
    if n > 2 && (an_order[0].rank - an_order[1].rank).abs() <= a_tol {
        let mut an_opened_order: Vec<OrderedCurve> = vec![an_order[0]];
        an_opened_order.extend_from_slice(&an_order[2..]);
        an_opened_order.push(an_order[1]);
        return an_opened_order;
    }
    if n > 2 && (an_order[n - 1].rank - an_order[n - 2].rank).abs() <= a_tol {
        let mut an_opened_order: Vec<OrderedCurve> = vec![an_order[n - 2]];
        an_opened_order.extend_from_slice(&an_order[..n - 2]);
        an_opened_order.push(an_order[n - 1]);
        return an_opened_order;
    }
    an_order
}

/// `orientedCurve`: a copy of the curve, reversed unless `the_is_forward`.
pub(crate) fn oriented_curve(
    the_curve: &Handle<GeomBSplineCurve>,
    the_is_forward: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let copy: Handle<GeomBSplineCurve> = the_curve.get().ok_or_else(null_curve)?.copy()?.cast();
    if !the_is_forward {
        copy.get().ok_or_else(null_curve)?.reverse()?;
    }
    Ok(copy)
}

/// `orientedParameter`: the parameter in the requested orientation of the curve.
pub(crate) fn oriented_parameter(
    the_curve: &Handle<GeomBSplineCurve>,
    the_parameter: f64,
    the_is_forward: bool,
) -> Result<f64, Failure> {
    if the_is_forward {
        return Ok(the_parameter);
    }
    let (first, last) = curve_bounds(the_curve)?;
    Ok(first + last - the_parameter)
}

/// The expected grid position of one pair and the selections before it
/// (`chooseOrderedContact` arguments).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ContactTargets {
    pub(crate) profile_forward: bool,
    pub(crate) guide_forward: bool,
    pub(crate) profile_target: f64,
    pub(crate) guide_target: f64,
    pub(crate) previous_profile_param: f64,
    pub(crate) previous_guide_param: f64,
    pub(crate) has_previous_profile: bool,
    pub(crate) has_previous_guide: bool,
    pub(crate) previous_profile_target: f64,
    pub(crate) previous_guide_target: f64,
    pub(crate) first_profile_param: f64,
    pub(crate) first_guide_param: f64,
    pub(crate) is_profile_seam: bool,
    pub(crate) is_guide_seam: bool,
}

/// `orderedContactScore`: the scaled gap, the distances to the targets and, after a
/// predecessor, the mismatch of the advances along the row or column.
fn ordered_contact_score(
    the_contact: &Contact,
    (the_profile_param, the_guide_param): (f64, f64),
    t: &ContactTargets,
    the_tolerance: f64,
) -> f64 {
    let mut a_score: f64 = the_contact.gap / the_tolerance.max(precision::confusion())
        + (the_profile_param - t.profile_target).abs()
        + (the_guide_param - t.guide_target).abs();
    if t.has_previous_profile {
        // Favor branches with a smooth advance along the current profile.
        a_score += ((the_profile_param - t.previous_profile_param)
            - (t.profile_target - t.previous_profile_target))
            .abs();
    }
    if t.has_previous_guide {
        // Favor branches with a smooth advance along the current guide.
        a_score += ((the_guide_param - t.previous_guide_param)
            - (t.guide_target - t.previous_guide_target))
            .abs();
    }
    a_score
}

/// `seamTangentMismatch`: `1 - cos` of the angle between the oriented tangents at two
/// parameters that give the same point; None when the points differ or a tangent
/// vanishes.
fn seam_tangent_mismatch(
    the_curve: &Handle<GeomBSplineCurve>,
    (the_first_parameter, the_last_parameter): (f64, f64),
    the_is_forward: bool,
    the_tolerance: f64,
) -> Result<Option<f64>, Failure> {
    let raw_first: f64 = oriented_parameter(the_curve, the_first_parameter, the_is_forward)?;
    let raw_last: f64 = oriented_parameter(the_curve, the_last_parameter, the_is_forward)?;
    let curve = the_curve.get().ok_or_else(null_curve)?;
    let (a_first_d1, a_last_d1) = (curve.eval_d1(raw_first)?, curve.eval_d1(raw_last)?);
    if a_first_d1.point.distance(&a_last_d1.point) > the_tolerance.max(precision::confusion()) {
        return Ok(None);
    }
    let (mut a_first_tangent, mut a_last_tangent) = (a_first_d1.d1, a_last_d1.d1);
    if !the_is_forward {
        a_first_tangent.reverse();
        a_last_tangent.reverse();
    }
    let (a_first_magnitude, a_last_magnitude) =
        (a_first_tangent.magnitude(), a_last_tangent.magnitude());
    if a_first_magnitude <= gp::resolution() || a_last_magnitude <= gp::resolution() {
        return Ok(None);
    }
    let cosine: f64 = a_first_tangent.dot(a_last_tangent) / (a_first_magnitude * a_last_magnitude);
    Ok(Some(1.0 - cosine.clamp(-1.0, 1.0)))
}

// The seam tangent mismatch of a candidate: the sum over the seams it lies on, None
// when no seam gives a match.
fn candidate_mismatch(
    curves: (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    (a_profile_param, a_guide_param): (f64, f64),
    t: &ContactTargets,
    the_tolerance: f64,
) -> Result<Option<f64>, Failure> {
    let mut mismatch: Option<f64> = None;
    if t.is_profile_seam {
        let params = (t.first_profile_param, a_profile_param);
        if let Some(m) = seam_tangent_mismatch(curves.0, params, t.profile_forward, the_tolerance)?
        {
            mismatch = Some(mismatch.unwrap_or(0.0) + m);
        }
    }
    if t.is_guide_seam {
        let params = (t.first_guide_param, a_guide_param);
        if let Some(m) = seam_tangent_mismatch(curves.1, params, t.guide_forward, the_tolerance)? {
            mismatch = Some(mismatch.unwrap_or(0.0) + m);
        }
    }
    Ok(mismatch)
}

/// `chooseOrderedContact`: among the candidates that advance past the predecessors,
/// the best seam tangent match (within Precision::Angular(), then the best score), or
/// without any seam match the best score; the selection carries the oriented
/// parameters. None when no candidate qualifies.
pub(crate) fn choose_ordered_contact(
    the_contacts: &ContactSet,
    curves: (&Handle<GeomBSplineCurve>, &Handle<GeomBSplineCurve>),
    t: &ContactTargets,
    the_tolerance: f64,
) -> Result<Option<Contact>, Failure> {
    let a_param_tol: f64 = parameter_resolution(0.0, 1.0);
    let (mut a_best_tangent_mismatch, mut a_best) = (f64::MAX, f64::MAX);
    let mut the_selected = Contact::default();
    for a_candidate in &the_contacts.candidates {
        let a_profile_param =
            oriented_parameter(curves.0, a_candidate.profile_param, t.profile_forward)?;
        let a_guide_param = oriented_parameter(curves.1, a_candidate.guide_param, t.guide_forward)?;
        if (t.has_previous_profile && a_profile_param <= t.previous_profile_param + a_param_tol)
            || (t.has_previous_guide && a_guide_param <= t.previous_guide_param + a_param_tol)
        {
            continue;
        }
        let params = (a_profile_param, a_guide_param);
        let a_score = ordered_contact_score(a_candidate, params, t, the_tolerance);
        let mismatch = candidate_mismatch(curves, params, t, the_tolerance)?;
        let better: bool = match mismatch {
            Some(m) => {
                m < a_best_tangent_mismatch - precision::angular()
                    || ((m - a_best_tangent_mismatch).abs() <= precision::angular()
                        && a_score < a_best)
            }
            None => a_best_tangent_mismatch == f64::MAX && a_score < a_best,
        };
        if better {
            a_best_tangent_mismatch = mismatch.unwrap_or(f64::MAX);
            a_best = a_score;
            the_selected = Contact {
                profile_param: a_profile_param,
                guide_param: a_guide_param,
                ..*a_candidate
            };
        }
    }
    Ok(the_selected.is_valid.then_some(the_selected))
}
