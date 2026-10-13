// SPDX-License-Identifier: GPL-2.0-only
//! One owning operation binds physical edge identity, captured scalar inputs and actual work.
use super::outcome::*;
use crate::sketch_region::OriginalMemberIdentity;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};

pub(super) struct CandidateWindingOperation<'a> {
    registry: Vec<&'a CandidateSegment>,
    pub receipt: CandidateCaptureReceipt,
}
pub(crate) struct CandidateCaptureReceipt {
    pub store: CapturedBezierWireStore,
    pub work: CaptureWorkReceipt,
    pub occurrences: Vec<CandidateCapturedOccurrence>,
}
pub(crate) struct CandidateCapturedOccurrence {
    pub id: u64,
    pub physical_source: u64,
    pub parents: Vec<OriginalMemberIdentity>,
    pub original_interval: [f64; 2],
    pub reversed: bool,
}
pub(crate) struct CandidateCapturedFailure {
    pub cause: Box<CandidateCause>,
    pub retained: CandidateCaptureReceipt,
}
impl<'a> CandidateWindingOperation<'a> {
    /// Start an operation with a fixed seed and full requested policy; `CandidateWindingOperation::new(seed,options)`.
    pub fn new(seed: [f64; 2], requested: RootIsolationOptions) -> Self {
        let store = CapturedBezierWireStore::new([seed, [seed[0] + 1., seed[1]]], requested);
        Self {
            registry: Vec::new(),
            receipt: CandidateCaptureReceipt {
                store,
                work: CaptureWorkReceipt::default(),
                occurrences: Vec::new(),
            },
        }
    }
    /// Preserve current IDs/directions while reusing completed proofs; `operation.classify(wire,cancelled)`.
    pub fn classify(
        &mut self,
        wire: &'a CandidateWire,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<CapturedBezierWireResult, CapturedBezierWireFailure> {
        let occurrences = wire
            .segments
            .iter()
            .map(|segment| self.capture_occurrence(segment))
            .collect::<Vec<_>>();
        let result = self.receipt.store.classify(&occurrences, cancelled);
        let work = match &result {
            Ok(result) => result.work(),
            Err(failure) => failure.work(),
        };
        self.append_work(work);
        result
    }
    fn capture_occurrence(
        &mut self,
        segment: &'a CandidateSegment,
    ) -> CapturedBezierWireOccurrence {
        let physical_source = self.physical_source(segment);
        self.receipt
            .occurrences
            .push(retained_occurrence(segment, physical_source));
        CapturedBezierWireOccurrence {
            id: segment.occurrence,
            reversed: segment.reversed,
            input: CapturedBezierWireInput::capture(
                physical_source,
                &segment.controls,
                segment.weights.as_deref(),
                segment.interval,
            ),
        }
    }
    fn physical_source(&mut self, segment: &'a CandidateSegment) -> u64 {
        if let Some(index) = self
            .registry
            .iter()
            .position(|source| same_original_domain(source, segment))
        {
            return index as u64 + 1;
        }
        self.registry.push(segment);
        self.registry.len() as u64
    }
    fn append_work(&mut self, work: CaptureWorkReceipt) {
        let accumulated = &mut self.receipt.work;
        accumulated.new_bisections += work.new_bisections;
        accumulated.operation_bisections = work.operation_bisections;
        accumulated.reused_certificates += work.reused_certificates;
        accumulated.certificate_bisections += work.certificate_bisections;
    }
}
fn same_original_domain(first: &CandidateSegment, second: &CandidateSegment) -> bool {
    first.edge.is_same(&second.edge)
        && first.original_interval.map(f64::to_bits) == second.original_interval.map(f64::to_bits)
}
fn retained_occurrence(
    segment: &CandidateSegment,
    physical_source: u64,
) -> CandidateCapturedOccurrence {
    CandidateCapturedOccurrence {
        id: segment.occurrence,
        physical_source,
        parents: segment.parents.clone(),
        original_interval: segment.original_interval,
        reversed: segment.reversed,
    }
}
