// SPDX-License-Identifier: GPL-2.0-only
//! Typed declines: operation, stage, entities and cause; no partial result accompanies them.
use super::profile::{ProfileCurveId, ProfileVertexId};
use std::fmt;

/// The public operation that declined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    /// `build_planar_regions`.
    BuildPlanarRegions,
    /// `extrude`.
    Extrude,
}

/// The ordered stage at which the operation declined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationStage {
    /// Caller values were checked before any construction.
    Admission,
    /// Every vertex was checked to close the curve graph (no dangling end).
    Loops,
    /// Contacts, branch order, faces, nesting and areas were certified.
    Certification,
    /// Native topology was constructed.
    Construction,
    /// Source to result history was projected.
    History,
    /// Ordered topology, geometry and self-intersection validation ran.
    Validation,
    /// The achieved tolerance was compared with the cap.
    Tolerance,
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
    /// Native topology checks (BRepCheck without geometry).
    Topology,
    /// Native geometry checks (BRepCheck with geometry).
    Geometry,
    /// Self-intersection of the published shape.
    SelfIntersection,
    /// Achieved tolerance against the cap.
    Tolerance,
}

/// The detailed cause, with the offending entities and values.
#[derive(Clone, Debug, PartialEq)]
pub enum DeclineCause {
    /// A resolution field is not finite and positive.
    InvalidResolution {
        /// Field name, such as `"sew"` or `"cap"`.
        field: &'static str,
        /// The offending value.
        value: f64,
    },
    /// The sketch plane frame is unusable.
    InvalidPlane {
        /// What is wrong with the frame.
        reason: &'static str,
    },
    /// The profile has no curve.
    EmptyProfile,
    /// A vertex has a nonfinite coordinate.
    NonFiniteVertex {
        /// The vertex.
        vertex: ProfileVertexId,
    },
    /// A curve has a nonfinite parameter.
    NonFiniteCurve {
        /// The curve.
        curve: ProfileCurveId,
    },
    /// Two vertices share one id.
    DuplicateVertex {
        /// The repeated id.
        vertex: ProfileVertexId,
    },
    /// Two curves share one id.
    DuplicateCurve {
        /// The repeated id.
        curve: ProfileCurveId,
    },
    /// A curve names a vertex that the profile does not list.
    UnknownVertex {
        /// The curve.
        curve: ProfileCurveId,
        /// The missing vertex.
        vertex: ProfileVertexId,
    },
    /// The curve kind is not yet supported.
    UnsupportedCurve {
        /// The curve.
        curve: ProfileCurveId,
        /// Its kind, such as `"ellipse"`.
        kind: &'static str,
    },
    /// A curve is shorter than the weld tolerance, or an arc sweeps less than the angular
    /// tolerance or starts and ends at one vertex.
    DegenerateCurve {
        /// The curve.
        curve: ProfileCurveId,
        /// Its length, radius or chord-scale size.
        size: f64,
        /// The weld tolerance.
        weld: f64,
    },
    /// An arc's ends are at different distances from its center.
    ArcRadiusMismatch {
        /// The arc.
        curve: ProfileCurveId,
        /// Start and end radius.
        radii: [f64; 2],
        /// The sew tolerance.
        sew: f64,
    },
    /// Two distinct used vertices are closer than the weld tolerance.
    CoincidentVertices {
        /// The two vertices.
        vertices: [ProfileVertexId; 2],
        /// Their distance.
        distance: f64,
        /// The weld tolerance.
        weld: f64,
    },
    /// A vertex ends fewer than two curves, so its loop is open.
    OpenProfile {
        /// The dangling vertex.
        vertex: ProfileVertexId,
        /// Curve ends at it.
        uses: usize,
    },
    /// A vertex ends more than two curves. No longer produced: branches are admitted by
    /// the plane graph proof (ADR 0084); kept so existing matches compile.
    BranchingProfile {
        /// The branching vertex.
        vertex: ProfileVertexId,
        /// Curve ends at it.
        uses: usize,
    },
    /// Two or more curves leave a vertex of three or more curve ends in the same direction,
    /// so the order of the faces around it is undecided (ADR 0084).
    TangentBranches {
        /// The vertex.
        vertex: ProfileVertexId,
    },
    /// A face boundary (`hole == false`) or the boundary of a nested group of loops
    /// (`hole == true`) passes through `vertex` twice, so it is not a simple loop; for
    /// example a loop touching its enclosing loop from inside (ADR 0084).
    PinchedRegion {
        /// The vertex visited twice.
        vertex: ProfileVertexId,
        /// Whether the pinched boundary is a hole.
        hole: bool,
    },
    /// Two curves meet away from a shared vertex.
    SelfIntersecting {
        /// The two curves, in caller order.
        curves: [ProfileCurveId; 2],
        /// A contact point; absent for an overlap.
        point: Option<[f64; 2]>,
    },
    /// A required certificate did not complete.
    IncompleteCertification {
        /// The refused obligation and its typed cause.
        reason: String,
    },
    /// The extrusion vector is unusable.
    InvalidDirection {
        /// What is wrong with it.
        reason: &'static str,
        /// The vector.
        direction: [f64; 3],
    },
    /// A result entity lacks exactly one source, or a source lacks its image.
    IncompleteHistory {
        /// The missing or ambiguous relation.
        reason: String,
    },
    /// An ordered validation stage did not pass.
    ValidationFailed {
        /// The first stage that did not pass.
        stage: ValidationStageKind,
        /// Every native finding, rendered.
        findings: Vec<String>,
    },
    /// The achieved tolerance exceeds a `ToleranceCap::Refuse` limit.
    ToleranceOverBudget {
        /// Largest stored tolerance.
        achieved: f64,
        /// The refused limit.
        cap: f64,
    },
    /// A native kernel call failed.
    Native {
        /// The native step.
        operation: &'static str,
        /// The native failure, rendered.
        cause: String,
    },
}

impl DeclineCause {
    /// The failure class of this cause; `cause.class()`.
    pub fn class(&self) -> DeclineClass {
        use DeclineCause::*;
        match self {
            UnsupportedCurve { .. } | TangentBranches { .. } | PinchedRegion { .. } => {
                DeclineClass::Unsupported
            }
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
    /// The declining operation.
    pub operation: OperationKind,
    /// The stage that declined.
    pub stage: OperationStage,
    /// The detailed cause.
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
