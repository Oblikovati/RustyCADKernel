// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::RootIsolationOptions;
use super::super::super::winding::{WindingEvaluation, WindingQuery};
use super::super::{BezierWireError, endpoint, walk::WireWalk};
use super::CapturedBezierWireOccurrence;
use super::result::{CaptureWorkReceipt, CapturedBezierWireFailure, CapturedBezierWireResult};

use super::proof::CapturedBezierWindingProof;
/// Owned operation-local proof store; requested policy and ray never change.
/// New computations share one actual subdivision budget across wire assemblies.
pub struct CapturedBezierWireStore {
    line: [[f64; 2]; 2],
    requested: RootIsolationOptions,
    spent: u16,
    complete: Vec<CapturedBezierWindingProof>,
}
impl CapturedBezierWireStore {
    /// Start a new bounded operation; example: `CapturedBezierWireStore::new(ray, options)`.
    /// Invalid scalar policy is refused by the first actual query in native order.
    pub fn new(line: [[f64; 2]; 2], requested: RootIsolationOptions) -> Self {
        Self {
            line,
            requested,
            spent: 0,
            complete: Vec::new(),
        }
    }
    /// Assemble current occurrences with exact closure and separately charged reuse.
    /// Example: `store.classify(&occurrences, || false)`; no certificate proves a face.
    pub fn classify(
        &mut self,
        occurrences: &[CapturedBezierWireOccurrence],
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<CapturedBezierWireResult, CapturedBezierWireFailure> {
        let before = self.spent;
        let mut receipt = CaptureWorkReceipt::default();
        let result = self.assemble(occurrences, &mut cancelled, &mut receipt);
        receipt.new_bisections = self.spent - before;
        receipt.operation_bisections = self.spent;
        match result {
            Ok(location) => Ok(CapturedBezierWireResult {
                location,
                work: receipt,
            }),
            Err(cause) => Err(CapturedBezierWireFailure {
                cause,
                work: receipt,
            }),
        }
    }

    /// Borrow complete owned proofs only; example: `store.proofs()[0].denominator()`.
    pub fn proofs(&self) -> &[CapturedBezierWindingProof] {
        &self.complete
    }
    fn assemble(
        &mut self,
        occurrences: &[CapturedBezierWireOccurrence],
        cancelled: &mut impl FnMut() -> bool,
        receipt: &mut CaptureWorkReceipt,
    ) -> Result<super::super::BezierWireLocation, BezierWireError> {
        validate_occurrences(occurrences, cancelled)?;
        let mut walk = WireWalk::default();
        for occurrence in occurrences {
            super::super::poll(occurrence.id, cancelled)?;
            self.append_occurrence(&mut walk, occurrence, cancelled, receipt)?;
        }
        Ok(walk.finish()?.location().clone())
    }
    fn append_occurrence(
        &mut self,
        walk: &mut WireWalk,
        occurrence: &CapturedBezierWireOccurrence,
        cancelled: &mut impl FnMut() -> bool,
        receipt: &mut CaptureWorkReceipt,
    ) -> Result<(), BezierWireError> {
        let index = self
            .complete
            .iter()
            .position(|proof| proof.input.same_payload(&occurrence.input));
        if let Some(index) = index {
            receipt.reused_certificates += 1;
            return replay(walk, occurrence, &self.complete[index], receipt);
        }
        let proof = self.capture(walk, occurrence, cancelled)?;
        receipt.certificate_bisections += u64::from(proof.evaluated.bisections);
        self.complete.push(proof);
        Ok(())
    }
    fn capture(
        &mut self,
        walk: &mut WireWalk,
        occurrence: &CapturedBezierWireOccurrence,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CapturedBezierWindingProof, BezierWireError> {
        let effective = self.remaining_policy();
        let (query, endpoints) = self.prepare_capture(occurrence, effective, cancelled)?;
        join(walk, occurrence, endpoints.clone())?;
        let evaluated = self.finish_capture(query, occurrence.id, cancelled)?;
        walk.append_contribution(occurrence.id, occurrence.reversed, evaluated.incidence, 0)?;
        Ok(CapturedBezierWindingProof {
            input: occurrence.input.clone(),
            endpoints,
            evaluated,
            effective,
        })
    }
    fn prepare_capture(
        &mut self,
        occurrence: &CapturedBezierWireOccurrence,
        effective: RootIsolationOptions,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(WindingQuery, [endpoint::Endpoint; 2]), BezierWireError> {
        let request = occurrence.input.request(self.line, effective);
        let prepared =
            WindingQuery::prepare_accounted(request, cancelled, endpoint::capture, &mut || {
                self.spent += 1
            });
        prepared.map_err(|cause| BezierWireError::Segment {
            id: occurrence.id,
            cause,
        })
    }
    fn finish_capture(
        &mut self,
        query: WindingQuery,
        id: u64,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WindingEvaluation, BezierWireError> {
        let evaluated = query.evaluate_accounted(cancelled, &mut || self.spent += 1);
        evaluated.map_err(|cause| BezierWireError::Segment { id, cause })
    }
    fn remaining_policy(&self) -> RootIsolationOptions {
        RootIsolationOptions {
            max_bisections: self.requested.max_bisections - self.spent,
            ..self.requested
        }
    }
}
fn validate_occurrences(
    occurrences: &[CapturedBezierWireOccurrence],
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), BezierWireError> {
    let borrowed: Vec<_> = occurrences
        .iter()
        .map(|o| super::super::BezierWireSegment {
            id: o.id,
            controls: &o.input.controls,
            weights: o.input.weights.as_deref(),
            interval: o.input.interval,
            reversed: o.reversed,
        })
        .collect();
    super::super::validate_ids(&borrowed, cancelled)
}
fn replay(
    walk: &mut WireWalk,
    occurrence: &CapturedBezierWireOccurrence,
    proof: &CapturedBezierWindingProof,
    receipt: &mut CaptureWorkReceipt,
) -> Result<(), BezierWireError> {
    join(walk, occurrence, proof.endpoints.clone())?;
    receipt.certificate_bisections += u64::from(proof.evaluated.bisections);
    walk.append_contribution(
        occurrence.id,
        occurrence.reversed,
        proof.evaluated.incidence,
        0,
    )
}
fn join(
    walk: &mut WireWalk,
    occurrence: &CapturedBezierWireOccurrence,
    mut endpoints: [endpoint::Endpoint; 2],
) -> Result<(), BezierWireError> {
    if occurrence.reversed {
        endpoints.reverse();
    }
    walk.join(occurrence.id, endpoints)
}
