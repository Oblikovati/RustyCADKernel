// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootBracket, RootIsolation, RootIsolationOptions};
use super::super::{BezierLineQuery, BezierRayError};

/// Which closed line domain is certified; the curve basis always remains `[0,1]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierLineDomain {
    /// Infinite line; every supporting contact is retained.
    Support,
    /// Closed segment; endpoint signs certify inclusion.
    Segment,
}
/// Caller occurrence identities plus the exact supplied curve and independent error policy.
#[derive(Clone, Copy, Debug)]
pub struct BezierIncidenceQuery<'a> {
    /// Application occurrence key; it does not certify geometry or root identity.
    pub curve_id: u64,
    /// Application line occurrence key.
    pub line_id: u64,
    /// Borrowed controls, weights, parameter bounds, and root work policy.
    pub curve: BezierLineQuery<'a>,
    /// Infinite support or closed segment between the query's two line points.
    pub domain: BezierLineDomain,
    /// Segment-only sign admission; version remains validated for supporting queries.
    pub domain_sign: BezierDomainSignOptions,
    /// Finite positive Euclidean construction-error limit, in source world units.
    pub max_world_error: f64,
}
/// Owned input provenance; coefficients and domain are bound to the reported certificates.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierIncidenceSource {
    pub(super) curve_id: u64,
    pub(super) line_id: u64,
    pub(super) controls: Vec<[f64; 2]>,
    pub(super) weights: Option<Vec<f64>>,
    pub(super) line: [[f64; 2]; 2],
    pub(super) interval: [f64; 2],
    pub(super) domain: BezierLineDomain,
    pub(super) domain_sign: BezierDomainSignOptions,
    pub(super) root_options: RootIsolationOptions,
    pub(super) requested_world_error: f64,
}
impl BezierIncidenceSource {
    pub(super) fn retain(query: BezierIncidenceQuery<'_>) -> Self {
        Self {
            curve_id: query.curve_id,
            line_id: query.line_id,
            controls: query.curve.controls.to_vec(),
            weights: query.curve.weights.map(<[f64]>::to_vec),
            line: query.curve.line,
            interval: query.curve.interval,
            domain: query.domain,
            domain_sign: query.domain_sign,
            root_options: query.curve.options,
            requested_world_error: query.max_world_error,
        }
    }
}
/// One selected root and its conservative binary64 coordinate enclosure.
#[derive(Clone, Debug, PartialEq)]
pub struct CertifiedBezierIncidence {
    /// Index into the complete supporting-line isolation, including excluded roots.
    pub(super) support_root_index: usize,
    /// Exactly one root of the retained source's projected numerator.
    pub(super) parameter: RootBracket,
    /// Per-axis lower/upper bounds containing the rational world point at that root.
    pub(super) world_bounds: [[f64; 2]; 2],
    /// Finite constructed point inside the coordinate enclosure.
    pub(super) point: [f64; 2],
    /// Outward Euclidean distance bound from `point` to the true root world point.
    pub(super) achieved_world_error: f64,
}
/// Complete isolated incidences; an empty contact list is a completed empty result.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierIncidenceReport {
    pub(super) source: BezierIncidenceSource,
    /// Complete empty denominator isolation over the retained source interval.
    pub(super) denominator_isolation: RootIsolation,
    pub(super) degrees: BezierIncidenceDegrees,
    /// Complete supporting-line roots before domain filtering, in parameter order.
    pub(super) support_isolation: RootIsolation,
    /// Exact endpoint classification aligned with every supporting-line root.
    pub(super) in_domain: Vec<bool>,
    /// Selected roots in the same deterministic order, with achieved construction error.
    pub(super) contacts: Vec<CertifiedBezierIncidence>,
    /// Total denominator, isolation, and subsequent root-refinement subdivisions.
    pub(super) bisections: u16,
}
/// A request lacking a complete root-and-world certificate; never a successful partial list.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierIncidenceFailure {
    pub(super) evidence: Box<BezierIncidenceFailureEvidence>,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct BezierIncidenceFailureEvidence {
    pub(super) source: BezierIncidenceSource,
    pub(super) cause: BezierIncidenceCause,
    /// Complete parameter isolation, if established before construction failed.
    pub(super) support_isolation: Option<RootIsolation>,
    /// Last certified bracket being refined, if parameter isolation completed.
    pub(super) last_root: Option<RootBracket>,
}
/// Original typed causes and explicit construction-stage noncompletion.
#[derive(Clone, Debug, PartialEq)]
pub enum BezierIncidenceCause {
    /// Original exact projection, denominator, root, or endpoint-sign cause.
    Classification(BezierRayError),
    /// Unsupported domain-sign policy version; expected version 1.
    DomainPolicyVersion(u32),
    /// Requested source-bound domain-sign admission could not complete.
    DomainSignAdmission(BezierRayError),
    /// Source basis degree exceeds the existing u16 conversion policy representation.
    DomainDegreeRepresentation(usize),
    /// Rejected nonfinite or nonpositive world error limit.
    WorldErrorLimit(f64),
    /// Caller cancellation; no partial result is successful.
    Cancelled,
    /// Parameter isolation completed but construction exhausted its work limit.
    RefinementExhausted {
        /// Total caller subdivision budget.
        limit: u16,
    },
    /// Current finite-point enclosure policy cannot certify the requested error.
    ConstructionResolutionLimit {
        /// Caller world error request.
        requested_error: f64,
        /// Effective world error limit; never widened from the request.
        effective_error_limit: f64,
        /// Last measured point/enclosure, or None when arithmetic could not enclose it.
        last_enclosure: Option<BezierConstructionEnclosure>,
    },
    /// A bracket that certified one root failed the refinement root-count invariant.
    RefinementInvariant,
}
/// Explicit computation outcome; completed empty remains `Complete`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierIncidenceCompletion {
    /// All admitted roots, domain signs, and selected world points are certified.
    Complete,
    /// Invalid input or policy, preserving the original offending values.
    Invalid,
    /// Representation was deliberately not admitted as isolated contacts.
    Declined,
    /// Caller stopped the computation before complete certification.
    Cancelled,
    /// Exhausted or inconclusive required certification.
    Incomplete,
}

/// Last conservative construction evidence retained on noncompletion.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierConstructionEnclosure {
    pub(super) world_bounds: [[f64; 2]; 2],
    pub(super) point: [f64; 2],
    pub(super) achieved_world_error: f64,
}
/// Versioned admission for derived domain signs, separate from root-polynomial admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BezierDomainSignOptions {
    /// This implementation admits version 1.
    pub version: u32,
    /// Longitudinal conversion cap; multiplying by the denominator can increase degree.
    pub degree_limit: BezierDomainDegreeLimit,
}
/// Explicit requested admission or a bound derived from the retained source basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierDomainDegreeLimit {
    /// Use the source Bernstein basis degree, without widening the root limit.
    SourceBasis,
    /// Admit longitudinal conversion only through this degree.
    Explicit(u16),
}
impl BezierDomainSignOptions {
    /// Request source-basis domain admission; example: `BezierDomainSignOptions::source_basis()`.
    pub fn source_basis() -> Self {
        Self {
            version: 1,
            degree_limit: BezierDomainDegreeLimit::SourceBasis,
        }
    }
}
/// Actual derived degrees; None means skipped segment stages or an exact zero sign query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BezierIncidenceDegrees {
    pub(super) source_basis: usize,
    pub(super) normal: usize,
    pub(super) denominator: usize,
    pub(super) origin_conversion: Option<usize>,
    pub(super) terminal_conversion: Option<usize>,
    pub(super) origin_sign_product: Option<usize>,
    pub(super) terminal_sign_product: Option<usize>,
    pub(super) effective_domain_limit: Option<u16>,
}
