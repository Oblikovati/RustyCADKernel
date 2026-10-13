// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootIsolationError, signs::RootSignError};
use super::super::{BezierLineError, BezierRayError};
use super::types::*;
use std::fmt;

impl BezierIncidenceReport {
    /// Read complete outcome including empty; example: `report.completion()`.
    pub fn completion(&self) -> BezierIncidenceCompletion {
        BezierIncidenceCompletion::Complete
    }
}
impl BezierIncidenceFailure {
    /// Distinguish invalid, declined, cancelled, incomplete; example: `failure.completion()`.
    pub fn completion(&self) -> BezierIncidenceCompletion {
        use BezierIncidenceCause::*;
        match &self.evidence.cause {
            Classification(cause) => classification_completion(cause),
            DomainPolicyVersion(_) => BezierIncidenceCompletion::Invalid,
            DomainSignAdmission(cause) => domain_admission_completion(cause),
            DomainDegreeRepresentation(_) => BezierIncidenceCompletion::Incomplete,
            WorldErrorLimit(_) => BezierIncidenceCompletion::Invalid,
            Cancelled => BezierIncidenceCompletion::Cancelled,
            RefinementExhausted { .. }
            | ConstructionResolutionLimit { .. }
            | RefinementInvariant => BezierIncidenceCompletion::Incomplete,
        }
    }
}
fn domain_admission_completion(cause: &BezierRayError) -> BezierIncidenceCompletion {
    if classification_completion(cause) == BezierIncidenceCompletion::Cancelled {
        return BezierIncidenceCompletion::Cancelled;
    }
    BezierIncidenceCompletion::Incomplete
}
fn classification_completion(cause: &BezierRayError) -> BezierIncidenceCompletion {
    match cause {
        BezierRayError::SupportingLineCoincidence => BezierIncidenceCompletion::Declined,
        BezierRayError::Line(BezierLineError::Root { cause, .. })
        | BezierRayError::Direction(cause)
        | BezierRayError::Endpoint(cause)
        | BezierRayError::Signs(RootSignError::Root { cause, .. }) => root_completion(cause),
        BezierRayError::Line(_) => BezierIncidenceCompletion::Invalid,
        BezierRayError::Signs(_) | BezierRayError::LocalNormalSign(_) => {
            BezierIncidenceCompletion::Incomplete
        }
    }
}
fn root_completion(cause: &RootIsolationError) -> BezierIncidenceCompletion {
    match cause {
        RootIsolationError::Cancelled => BezierIncidenceCompletion::Cancelled,
        RootIsolationError::SubdivisionLimit { .. } | RootIsolationError::Variation { .. } => {
            BezierIncidenceCompletion::Incomplete
        }
        RootIsolationError::Degree { .. } | RootIsolationError::ZeroPolynomial { .. } => {
            BezierIncidenceCompletion::Declined
        }
        _ => BezierIncidenceCompletion::Invalid,
    }
}
impl fmt::Display for BezierIncidenceCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::Classification(cause) = self {
            return cause.fmt(formatter);
        }
        write!(
            formatter,
            "{}; expected complete finite world construction within the requested error",
            cause_problem(self)
        )
    }
}
fn cause_problem(cause: &BezierIncidenceCause) -> String {
    use BezierIncidenceCause::*;
    match cause {
        Classification(cause) => cause.to_string(),
        DomainPolicyVersion(_) | DomainSignAdmission(_) | DomainDegreeRepresentation(_) => {
            domain_problem(cause)
        }
        ConstructionResolutionLimit { .. } => construction_problem(cause),
        _ => construction_state_problem(cause),
    }
}
fn domain_problem(cause: &BezierIncidenceCause) -> String {
    match cause {
        BezierIncidenceCause::DomainPolicyVersion(version) => {
            format!("domain policy version {version}; expected version 1")
        }
        BezierIncidenceCause::DomainSignAdmission(cause) => {
            format!("domain-sign admission: {cause}")
        }
        BezierIncidenceCause::DomainDegreeRepresentation(degree) => {
            format!("source basis degree {degree}; expected u16 domain conversion cap")
        }
        _ => unreachable!("cause {cause:?}; expected domain policy cause"),
    }
}
fn construction_problem(cause: &BezierIncidenceCause) -> String {
    match cause {
        BezierIncidenceCause::ConstructionResolutionLimit {
            requested_error,
            effective_error_limit,
            last_enclosure,
        } => format!(
            "construction request {requested_error}, effective limit {effective_error_limit}, last enclosure {last_enclosure:?}"
        ),
        _ => unreachable!("cause {cause:?}; expected resolution limit"),
    }
}
fn construction_state_problem(cause: &BezierIncidenceCause) -> String {
    match cause {
        BezierIncidenceCause::WorldErrorLimit(value) => {
            format!("world error limit {value} is not finite positive")
        }
        BezierIncidenceCause::Cancelled => "incidence cancelled".into(),
        BezierIncidenceCause::RefinementExhausted { limit } => {
            format!("subdivision budget {limit} exhausted")
        }
        BezierIncidenceCause::RefinementInvariant => {
            "refined bracket root count differs from one".into()
        }
        _ => unreachable!("cause {cause:?}; expected construction state cause"),
    }
}
impl fmt::Display for BezierIncidenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "curve {}, line {}, bracket {:?}: {}",
            self.evidence.source.curve_id,
            self.evidence.source.line_id,
            self.evidence.last_root,
            self.evidence.cause
        )
    }
}
impl std::error::Error for BezierIncidenceCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Classification(cause) | Self::DomainSignAdmission(cause) => Some(cause),
            _ => None,
        }
    }
}
impl std::error::Error for BezierIncidenceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.evidence.cause)
    }
}
