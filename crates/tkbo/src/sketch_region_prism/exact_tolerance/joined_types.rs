// SPDX-License-Identifier: GPL-2.0-only
//! Owned exact geometry evidence and private partial diagnostics never promote ordered coverage.
use super::{closure_scalar::ExactClosureFailure, frames::*, scalar::*};
use crate::sketch_region_prism::{frozen::proof_join::JoinedPrismProof, frozen::*};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::DyadicBound;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedConformancePhase {
    Admission,
    Placements,
    Endpoints,
    Closures,
    Trims,
    Commit,
}

pub(crate) trait JoinedConformanceProgress {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool;
}

pub(crate) struct JoinedPrismConformance {
    pub(super) binding: Vec<u8>,
    pub(super) operation_limits: FrozenPrismLimits,
    pub(super) evidence: JoinedConformanceEvidence,
}

#[derive(Default)]
pub(crate) struct JoinedConformanceEvidence {
    pub placements: Vec<JoinedPlacementEvidence>,
    pub endpoints: Vec<JoinedEndpointEvidence>,
    pub closures: Vec<JoinedClosureEvidence>,
    pub trims: Vec<JoinedTrimEvidence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedFrameRole {
    Occurrence,
    CurveRepresentation,
    FaceRepresentation,
    SourceRepresentation,
    TrimRepresentation,
}

pub(crate) struct JoinedPlacementEvidence {
    pub entity: FrozenPrismKey,
    pub occurrence: Option<FrozenUseKey>,
    pub role: JoinedFrameRole,
    pub certificate: UsableToleranceFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanonicalEndpointSide {
    First,
    Last,
}

pub(crate) struct JoinedEndpointEvidence {
    pub edge: FrozenPrismKey,
    pub vertex: FrozenPrismKey,
    pub side: CanonicalEndpointSide,
    pub parameter: f64,
    pub contexts: Vec<FrozenUseKey>,
    pub local_point: [f64; 3],
    pub certificate: ExactEndpointCertificate,
}

pub(crate) struct JoinedClosureEvidence {
    pub contact: AuthoredPrismId,
    pub original: FrozenOriginalKey,
    pub stage: &'static str,
    pub orientations: Vec<Orientation>,
    pub local_point: [f64; 3],
    pub certificate: ExactEndpointCertificate,
}

pub(crate) struct JoinedTrimEvidence {
    pub face: FrozenPrismKey,
    pub occurrence: FrozenUseKey,
    pub squared_upper: DyadicBound,
}

pub(crate) struct JoinedConformanceFailure {
    pub operation: &'static str,
    pub phase: JoinedConformancePhase,
    pub entity: Option<FrozenPrismKey>,
    pub occurrence: Option<FrozenUseKey>,
    pub cause: JoinedConformanceCause,
    pub partial: JoinedConformanceEvidence,
    pub binding: Vec<u8>,
    pub operation_limits: FrozenPrismLimits,
}

pub(crate) enum JoinedConformanceCause {
    Count {
        quantity: &'static str,
        actual: usize,
        expected: usize,
    },
    Missing {
        required: &'static str,
    },
    Mismatch {
        quantity: &'static str,
    },
    Orientation {
        actual: Orientation,
    },
    Frame {
        role: JoinedFrameRole,
        cause: ToleranceFrameFailure,
    },
    Endpoint {
        vertex: FrozenPrismKey,
        side: CanonicalEndpointSide,
        contexts: Vec<FrozenUseKey>,
        cause: Box<ExactEndpointFailure>,
    },
    Closure {
        contact: AuthoredPrismId,
        original: FrozenOriginalKey,
        stage: &'static str,
        cause: Box<ExactClosureFailure>,
    },
    ClosureCensus {
        stage: &'static str,
        actual: usize,
        expected: usize,
    },
    ClosureOperands {
        stage: &'static str,
        contact: AuthoredPrismId,
        original: FrozenOriginalKey,
    },
    Cancelled,
}

impl JoinedPrismConformance {
    /// Exact complete joined operands must match; example: `proof.binds(join)`.
    pub(crate) fn binds(&self, join: &JoinedPrismProof) -> bool {
        self.binding == join.replay_bytes()
    }
    /// Keep exact policy separate from legacy inspection; example: `proof.version()`.
    pub(crate) fn version(&self) -> u32 {
        2
    }
    /// Borrow unchanged operation limits; example: `proof.operation_limits().spatial_limit`.
    pub(crate) fn operation_limits(&self) -> &FrozenPrismLimits {
        &self.operation_limits
    }
    /// Borrow completed geometric evidence; example: `proof.evidence().endpoints.len()`.
    pub(crate) fn evidence(&self) -> &JoinedConformanceEvidence {
        &self.evidence
    }
}
