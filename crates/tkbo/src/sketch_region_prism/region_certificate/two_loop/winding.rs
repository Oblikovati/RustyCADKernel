// SPDX-License-Identifier: GPL-2.0-only
//! Existing exact winding queries share their subdivision budget; captured proof values own no store.
use super::{failure::*, geometry, outcome::TwoLoopSourceProof, pair_capture, request::*, work::*};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::predicates::roots::{RootIsolation, RootIsolationOptions, bezier_line::*};

#[derive(Debug)]
pub(crate) struct TwoLoopWindingProof {
    pub source: RegionSourceId,
    pub ray: [[f64; 2]; 2],
    pub reversed: bool,
    pub effective: RootIsolationOptions,
    pub result: CapturedBezierWireResult,
    pub denominator: RootIsolation,
    pub evidence: CapturedWindingEvidence,
}
pub(super) fn query<'a>(
    source: &TwoLoopSourceProof,
    seed: [f64; 2],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopWindingProof> {
    let occurrence = occurrence(source, work)?;
    let ray = [seed, [seed[0] + 1., seed[1]]];
    let effective = remaining(work);
    let mut store = CapturedBezierWireStore::new(ray, effective);
    let result = execute(&mut store, &occurrence, work)?;
    work.count("one actual source winding proof", store.proofs().len(), 1)?;
    let proof = &store.proofs()[0];
    work.units("source winding record", 1)?;
    Ok(TwoLoopWindingProof {
        source: source.name,
        ray,
        reversed: occurrence.reversed,
        effective,
        denominator: pair_capture::isolation(proof.denominator(), work)?,
        evidence: evidence(proof.evidence(), work)?,
        result,
    })
}
fn occurrence<'a>(
    source: &TwoLoopSourceProof,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedBezierWireOccurrence> {
    let CapturedMetricCurve::Bezier { poles, weights } = &source.pcurve else {
        return Err(work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "polynomial source winding basis",
        }));
    };
    work.units("winding input poles", poles.len())?;
    work.units(
        "winding input weights",
        weights.as_ref().map_or(0, Vec::len),
    )?;
    Ok(CapturedBezierWireOccurrence {
        id: source.name.0,
        reversed: geometry::reversed(source.orientation, work)?,
        input: CapturedBezierWireInput::capture(source.name.0, poles, weights.as_deref(), [0., 1.]),
    })
}
fn remaining(work: &TwoLoopWork<'_>) -> RootIsolationOptions {
    RootIsolationOptions {
        max_bisections: work.requested.winding.max_bisections - work.winding_bisections,
        ..work.requested.winding
    }
}
fn execute<'a>(
    store: &mut CapturedBezierWireStore,
    occurrence: &CapturedBezierWireOccurrence,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedBezierWireResult> {
    let mut callback_failure = None;
    let result = store.classify(std::slice::from_ref(occurrence), || {
        callback_failure = work.poll(TwoLoopRegionStage::Nesting).err();
        callback_failure.is_some()
    });
    let spent = match &result {
        Ok(result) => result.work(),
        Err(error) => error.work(),
    };
    work.winding_bisections += spent.new_bisections;
    if let Some(failure) = callback_failure {
        return Err(failure);
    }
    result.map_err(|error| {
        work.failure(TwoLoopRegionCause::Winding {
            cause: error.cause().clone(),
            work: error.work(),
        })
    })
}
fn evidence<'a>(
    source: &CapturedWindingEvidence,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, CapturedWindingEvidence> {
    Ok(match source {
        CapturedWindingEvidence::Normal {
            evaluated,
            contributions,
        } => {
            work.units("winding contribution", contributions.len())?;
            CapturedWindingEvidence::Normal {
                evaluated: pair_capture::signs(evaluated, work)?,
                contributions: contributions.clone(),
            }
        }
        CapturedWindingEvidence::Coincident { directed } => CapturedWindingEvidence::Coincident {
            directed: pair_capture::isolation(directed, work)?,
        },
        CapturedWindingEvidence::ZeroDirected => CapturedWindingEvidence::ZeroDirected,
    })
}
pub(super) fn unit_interior<'a>(
    proof: &TwoLoopWindingProof,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, i64> {
    if let BezierWireLocation::Inside {
        winding: value @ (-1 | 1),
    } = proof.result.location()
    {
        return Ok(*value);
    }
    Err(work.failure(TwoLoopRegionCause::WindingLocation {
        source: proof.source,
        seed: proof.ray[0],
        actual: proof.result.location().clone(),
        expected: "strict Jordan interior with winding +1 or -1",
    }))
}
