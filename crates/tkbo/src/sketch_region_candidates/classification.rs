// SPDX-License-Identifier: GPL-2.0-only
//! Positive directed winding is material in the right-handed plane; negative loops are complements/holes.
use super::outcome::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    bezier_line::{
        BezierWireError, BezierWireLocation, BezierWireQuery, BezierWireResult, BezierWireSegment,
        classify_bezier_wire,
    },
};
#[derive(Default)]
struct SeedAccumulation {
    interiors: Vec<usize>,
    boundary: Vec<u64>,
    bisections: u32,
}
/// Classify all captured candidates, retaining missing construction/search/body certificates.
/// Example: `classify_candidates(&batch,[1.,1.],RootIsolationOptions::new(0.001),||false)`.
pub(crate) fn classify_candidates(
    batch: &CandidateBatch,
    seed: [f64; 2],
    options: RootIsolationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CandidateClassification, CandidateCause> {
    let mut accumulated = SeedAccumulation::default();
    for (index, candidate) in batch.candidates.iter().enumerate() {
        let winding =
            classify_candidate(candidate, seed, options, &mut cancelled, &mut accumulated)?;
        if winding > 0 {
            accumulated.interiors.push(index);
        }
    }
    accumulated.finish(options)
}
fn classify_candidate(
    candidate: &RegionCandidate,
    seed: [f64; 2],
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
    accumulated: &mut SeedAccumulation,
) -> Result<i64, CandidateCause> {
    let mut winding = 0;
    for wire in &candidate.wires {
        let effective = remaining_options(options, accumulated.bisections)?;
        let result = classify_wire(wire, seed, effective, cancelled)?;
        accumulated.append_result(&result, &mut winding)?;
    }
    Ok(winding)
}
fn classify_wire(
    wire: &CandidateWire,
    seed: [f64; 2],
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BezierWireResult, CandidateCause> {
    let segments: Vec<BezierWireSegment<'_>> = wire.segments.iter().map(segment_query).collect();
    let query = BezierWireQuery {
        segments: &segments,
        line: [seed, [seed[0] + 1., seed[1]]],
        options,
    };
    classify_bezier_wire(query, cancelled).map_err(CandidateCause::Numerical)
}
fn segment_query(segment: &CandidateSegment) -> BezierWireSegment<'_> {
    BezierWireSegment {
        id: segment.occurrence,
        controls: &segment.controls,
        weights: segment.weights.as_deref(),
        interval: segment.interval,
        reversed: segment.reversed,
    }
}
impl SeedAccumulation {
    fn append_result(
        &mut self,
        result: &BezierWireResult,
        winding: &mut i64,
    ) -> Result<(), CandidateCause> {
        self.bisections += u32::from(result.bisections());
        match result.location() {
            BezierWireLocation::Boundary { segment_ids } => self.boundary.extend(segment_ids),
            BezierWireLocation::Inside { winding: count } => {
                *winding = add_winding(*winding, *count)?
            }
            BezierWireLocation::Outside => {}
        }
        Ok(())
    }
    fn finish(
        self,
        requested: RootIsolationOptions,
    ) -> Result<CandidateClassification, CandidateCause> {
        let remaining = remaining_options(requested, self.bisections)?;
        Ok(CandidateClassification::Incomplete {
            selection: selection(self.interiors, self.boundary),
            missing: missing_certificates(),
            bisections: self.bisections,
            requested,
            remaining,
            achieved_error: None,
        })
    }
}
fn selection(interiors: Vec<usize>, boundary: Vec<u64>) -> SeedSelection {
    if !boundary.is_empty() {
        return SeedSelection::Boundary {
            occurrences: boundary,
        };
    }
    match interiors.as_slice() {
        [] => SeedSelection::Empty,
        [candidate] => SeedSelection::Unique {
            candidate: *candidate,
        },
        _ => SeedSelection::Ambiguous {
            candidates: interiors,
        },
    }
}
fn missing_certificates() -> Vec<MissingCertificate> {
    vec![
        MissingCertificate::Construction,
        MissingCertificate::SharedIncidenceReuse,
        MissingCertificate::SearchCompleteness,
        MissingCertificate::SelfIntersection,
        MissingCertificate::Tolerance,
    ]
}
fn add_winding(partial: i64, contribution: i64) -> Result<i64, CandidateCause> {
    partial
        .checked_add(contribution)
        .ok_or(CandidateCause::Numerical(
            BezierWireError::WindingOverflow {
                partial,
                contribution,
            },
        ))
}

fn remaining_options(
    mut requested: RootIsolationOptions,
    used: u32,
) -> Result<RootIsolationOptions, CandidateCause> {
    let remaining = u32::from(requested.max_bisections)
        .checked_sub(used)
        .ok_or(CandidateCause::Incomplete(CandidateGap::WorkBudget {
            requested: requested.max_bisections,
            used,
        }))?;
    requested.max_bisections = remaining as u16;
    Ok(requested)
}
