// SPDX-License-Identifier: GPL-2.0-only
//! Positive directed winding is material in the right-handed plane; negative loops are complements/holes.
use super::{
    captured_store::{
        CandidateCaptureReceipt, CandidateCapturedFailure, CandidateWindingOperation,
    },
    outcome::*,
};
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    bezier_line::{BezierWireError, BezierWireLocation, CapturedBezierWireResult},
};
#[derive(Default)]
struct SeedAccumulation {
    interiors: Vec<usize>,
    boundary: Vec<u64>,
}
/// Classify all captured candidates, retaining missing construction/search/body certificates.
/// Example: `classify_candidates(&batch,[1.,1.],RootIsolationOptions::new(0.001),||false)`.
pub(crate) fn classify_candidates(
    batch: &CandidateBatch,
    seed: [f64; 2],
    options: RootIsolationOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<CandidateClassification, CandidateCause> {
    let mut operation = CandidateWindingOperation::new(seed, options);
    match accumulate_seed(batch, &mut operation, &mut cancelled) {
        Ok(accumulated) => {
            accumulated.finish(options, !batch.unrepresented.is_empty(), operation.receipt)
        }
        Err(cause) => Err(retained_failure(cause, operation.receipt)),
    }
}
fn accumulate_seed<'a>(
    batch: &'a CandidateBatch,
    operation: &mut CandidateWindingOperation<'a>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<SeedAccumulation, CandidateCause> {
    let mut accumulated = SeedAccumulation::default();
    for (index, candidate) in batch.candidates.iter().enumerate() {
        let winding = classify_candidate(candidate, operation, cancelled, &mut accumulated)?;
        operation.receipt.candidate_windings.push(winding);
        if winding > 0 {
            accumulated.interiors.push(index);
        }
    }
    Ok(accumulated)
}

fn classify_candidate<'a>(
    candidate: &'a RegionCandidate,
    operation: &mut CandidateWindingOperation<'a>,
    cancelled: &mut impl FnMut() -> bool,
    accumulated: &mut SeedAccumulation,
) -> Result<i64, CandidateCause> {
    let mut winding = 0;
    for wire in &candidate.wires {
        let result = operation
            .classify(wire, cancelled)
            .map_err(CandidateCause::CapturedNumerical)?;
        accumulated.append_result(&result, &mut winding)?;
    }
    facing_winding(candidate, winding)
}

impl SeedAccumulation {
    fn append_result(
        &mut self,
        result: &CapturedBezierWireResult,
        winding: &mut i64,
    ) -> Result<(), CandidateCause> {
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
        unrepresented: bool,
        captured: CandidateCaptureReceipt,
    ) -> Result<CandidateClassification, CandidateCause> {
        let bisections = u32::from(captured.work.operation_bisections);
        let remaining = remaining_options(requested, bisections)?;
        Ok(CandidateClassification::Incomplete {
            selection: selection(self.interiors, self.boundary),
            missing: missing_certificates(unrepresented),
            bisections,
            requested,
            remaining,
            achieved_error: None,
            captured: Box::new(captured),
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
fn missing_certificates(unrepresented: bool) -> Vec<MissingCertificate> {
    let mut missing = vec![
        MissingCertificate::Construction,
        MissingCertificate::SearchCompleteness,
        MissingCertificate::SelfIntersection,
        MissingCertificate::Tolerance,
    ];
    if unrepresented {
        missing.push(MissingCertificate::CandidateHistoryCoverage);
    }
    missing
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

fn facing_winding(candidate: &RegionCandidate, winding: i64) -> Result<i64, CandidateCause> {
    use rustycad_tkg3d::top_abs::Orientation;
    match candidate.face.orientation() {
        Orientation::Forward => Ok(winding),
        Orientation::Reversed => winding.checked_neg().ok_or(CandidateCause::Incomplete(
            CandidateGap::WindingOrientationOverflow { winding },
        )),
        orientation => Err(CandidateCause::Incomplete(
            CandidateGap::UnsupportedFaceOrientation { orientation },
        )),
    }
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
fn retained_failure(cause: CandidateCause, retained: CandidateCaptureReceipt) -> CandidateCause {
    CandidateCause::CapturedOperation(Box::new(CandidateCapturedFailure {
        cause: Box::new(cause),
        retained,
    }))
}
