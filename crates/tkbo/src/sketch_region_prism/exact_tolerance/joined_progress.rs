// SPDX-License-Identifier: GPL-2.0-only
//! Only immutable evidence is traversed after callbacks; live source rebind belongs to the composer.
use super::joined_types::*;
use crate::sketch_region_prism::frozen::{FrozenPrismKey, FrozenUseKey};

pub(super) struct ConformanceWork {
    pub phase: JoinedConformancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub occurrence: Option<FrozenUseKey>,
    pub evidence: JoinedConformanceEvidence,
}
pub(super) type ConformanceCheck<T = ()> = Result<T, JoinedConformanceCause>;

impl ConformanceWork {
    pub fn new() -> Self {
        Self {
            phase: JoinedConformancePhase::Admission,
            entity: None,
            occurrence: None,
            evidence: JoinedConformanceEvidence::default(),
        }
    }
    pub fn phase(
        &mut self,
        phase: JoinedConformancePhase,
        progress: &mut impl JoinedConformanceProgress,
    ) -> ConformanceCheck {
        self.phase = phase;
        self.entity = None;
        self.occurrence = None;
        self.poll(progress)
    }
    pub fn poll(&self, progress: &mut impl JoinedConformanceProgress) -> ConformanceCheck {
        if progress.cancelled(self.phase) {
            return Err(JoinedConformanceCause::Cancelled);
        }
        Ok(())
    }
    pub fn failed(
        self,
        cause: JoinedConformanceCause,
        join: &crate::sketch_region_prism::frozen::proof_join::JoinedPrismProof,
    ) -> JoinedConformanceFailure {
        JoinedConformanceFailure {
            operation: "joined exact represented-geometry conformance V2",
            phase: self.phase,
            entity: self.entity,
            occurrence: self.occurrence,
            cause,
            partial: self.evidence,
            binding: join.replay_bytes().to_vec(),
            operation_limits: *join.projection().limits(),
        }
    }
}

pub(super) fn count(quantity: &'static str, actual: usize, expected: usize) -> ConformanceCheck {
    if actual != expected {
        return Err(JoinedConformanceCause::Count {
            quantity,
            actual,
            expected,
        });
    }
    Ok(())
}

pub(super) fn missing(required: &'static str) -> JoinedConformanceCause {
    JoinedConformanceCause::Missing { required }
}

pub(super) fn one<'a, T>(values: &'a [T], quantity: &'static str) -> ConformanceCheck<&'a T> {
    count(quantity, values.len(), 1)?;
    Ok(&values[0])
}
