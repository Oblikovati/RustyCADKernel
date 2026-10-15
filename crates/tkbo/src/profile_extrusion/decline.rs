// SPDX-License-Identifier: GPL-2.0-only
//! Typed declines: operation, stage, entities and cause; no partial result accompanies them.
use super::profile::{ProfileCurveId, ProfileVertexId};
use std::fmt;

/// The public operation that declined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    BuildPlanarRegions,
    Extrude,
}

/// The ordered stage at which the operation declined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationStage {
    Admission,
    Loops,
    Certification,
    Construction,
    History,
    Validation,
    Tolerance,
    Measurement,
}

/// Failure class, independent of the detailed cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclineClass {
    /// The caller input violates the documented contract.
    InvalidInput,
    /// The input is valid but not yet supported.
    Unsupported,
    /// The input describes no valid region or solid.
    InvalidGeometry,
    /// A required proof or check did not complete; never treated as valid.
    Incomplete,
    /// The result exceeds the caller's tolerance cap.
    OverBudget,
    /// A native kernel stage failed.
    Native,
}

/// Ordered validation stages of a published body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationStageKind {
    Topology,
    Geometry,
    SelfIntersection,
    Tolerance,
}

/// The detailed cause, with the offending entities and values.
#[derive(Clone, Debug, PartialEq)]
pub enum DeclineCause {
    InvalidResolution {
        field: &'static str,
        value: f64,
    },
    InvalidPlane {
        reason: &'static str,
    },
    EmptyProfile,
    NonFiniteVertex {
        vertex: ProfileVertexId,
    },
    NonFiniteCurve {
        curve: ProfileCurveId,
    },
    DuplicateVertex {
        vertex: ProfileVertexId,
    },
    DuplicateCurve {
        curve: ProfileCurveId,
    },
    UnknownVertex {
        curve: ProfileCurveId,
        vertex: ProfileVertexId,
    },
    UnsupportedCurve {
        curve: ProfileCurveId,
        kind: &'static str,
    },
    DegenerateCurve {
        curve: ProfileCurveId,
        size: f64,
        weld: f64,
    },
    ArcRadiusMismatch {
        curve: ProfileCurveId,
        radii: [f64; 2],
        sew: f64,
    },
    CoincidentVertices {
        vertices: [ProfileVertexId; 2],
        distance: f64,
        weld: f64,
    },
    OpenProfile {
        vertex: ProfileVertexId,
        uses: usize,
    },
    BranchingProfile {
        vertex: ProfileVertexId,
        uses: usize,
    },
    /// Two curves meet away from an authored join; `point` is absent for an overlap.
    SelfIntersecting {
        curves: [ProfileCurveId; 2],
        point: Option<[f64; 2]>,
    },
    /// A required certificate did not complete; `reason` names the refused obligation.
    IncompleteCertification {
        reason: String,
    },
    InvalidDirection {
        reason: &'static str,
        direction: [f64; 3],
    },
    IncompleteHistory {
        reason: String,
    },
    ValidationFailed {
        stage: ValidationStageKind,
        findings: Vec<String>,
    },
    ToleranceOverBudget {
        achieved: f64,
        cap: f64,
    },
    Native {
        operation: &'static str,
        cause: String,
    },
}

impl DeclineCause {
    /// The failure class of this cause; `cause.class()`.
    pub fn class(&self) -> DeclineClass {
        use DeclineCause::*;
        match self {
            UnsupportedCurve { .. } => DeclineClass::Unsupported,
            SelfIntersecting { .. } | OpenProfile { .. } | BranchingProfile { .. } => {
                DeclineClass::InvalidGeometry
            }
            ValidationFailed { .. } => DeclineClass::InvalidGeometry,
            IncompleteCertification { .. } | IncompleteHistory { .. } => DeclineClass::Incomplete,
            ToleranceOverBudget { .. } => DeclineClass::OverBudget,
            Native { .. } => DeclineClass::Native,
            _ => DeclineClass::InvalidInput,
        }
    }
}

/// A typed decline of `build_planar_regions` or `extrude`.
#[derive(Clone, Debug, PartialEq)]
pub struct OperationDecline {
    pub operation: OperationKind,
    pub stage: OperationStage,
    pub cause: DeclineCause,
}

impl OperationDecline {
    /// The failure class; `decline.class()`.
    pub fn class(&self) -> DeclineClass {
        self.cause.class()
    }
}

impl fmt::Display for OperationDecline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} declined at {:?} ({:?}): {:?}",
            self.operation,
            self.stage,
            self.class(),
            self.cause
        )
    }
}

impl std::error::Error for OperationDecline {}

/// Stage-local builder of declines; `Decliner { operation }.at(stage, cause)`.
#[derive(Clone, Copy)]
pub(crate) struct Decliner {
    pub operation: OperationKind,
}

impl Decliner {
    pub(crate) fn at(self, stage: OperationStage, cause: DeclineCause) -> OperationDecline {
        OperationDecline {
            operation: self.operation,
            stage,
            cause,
        }
    }

    pub(crate) fn native(
        self,
        stage: OperationStage,
        operation: &'static str,
        failure: rustycad_tkernel::standard::Failure,
    ) -> OperationDecline {
        let cause = DeclineCause::Native {
            operation,
            cause: failure.to_string(),
        };
        self.at(stage, cause)
    }
}
