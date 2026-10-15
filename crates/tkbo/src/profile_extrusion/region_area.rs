// SPDX-License-Identifier: GPL-2.0-only
//! Region area: the certified area of the region's Bézier boundary (ADR 0066), checked
//! against the closed form. Lines are represented exactly, so a line-only region keeps the
//! certified bound. Arc pieces carry rounded split points and weights, so their regions add
//! a representation estimate and are tagged `RepresentationEnclosure` (ADR 0081).
use super::decline::DeclineCause;
use crate::measured::{IntegrationMethod, Measured};

/// Inputs of one region: certified `(area, error)`, closed-form loop areas (outer first),
/// and whether every loop is line-only.
pub(crate) struct RegionAreaInput {
    pub certified: (f64, f64),
    pub loop_areas: Vec<f64>,
    pub line_only: bool,
}

/// The region measure, or an incomplete decline when the certified and closed-form areas
/// disagree (the proof would then have bound another face).
pub(crate) fn region_area(input: &RegionAreaInput) -> Result<Measured, DeclineCause> {
    let (certified, certified_error) = input.certified;
    let closed_form = closed_form(&input.loop_areas);
    let rounding = 64.0 * f64::EPSILON * closed_form.abs().max(certified.abs());
    let (representation, method) = match input.line_only {
        true => (0.0, IntegrationMethod::CertifiedEnclosure),
        false => (rounding, IntegrationMethod::RepresentationEnclosure),
    };
    let error = certified_error + representation;
    if (certified - closed_form).abs() > error + rounding {
        let reason = format!("certified area {certified} vs closed form {closed_form}");
        return Err(DeclineCause::IncompleteCertification { reason });
    }
    Ok(Measured {
        value: certified,
        error_bound: error,
        method,
    })
}

/// Outer area minus hole areas, from signed loop areas.
fn closed_form(loop_areas: &[f64]) -> f64 {
    let (outer, holes) = loop_areas
        .split_first()
        .expect("a region has an outer loop");
    outer.abs() - holes.iter().map(|hole| hole.abs()).sum::<f64>()
}
