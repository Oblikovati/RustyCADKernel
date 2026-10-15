// SPDX-License-Identifier: GPL-2.0-only
//! Source-bound planar Bézier pair and canonical off-diagonal self certificates.
//!
//! One general exact original-ideal row pipeline covers polynomial and rational inputs.
//! Nonlinear or unresolved exceptional fibres and unproved overlaps are explicitly incomplete.
//! No supplied resultant, point proximity or caller identity certifies a contact (ADR 0014).
//! ```
//! use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_pair::*};
//! let a = [[0., 0.], [0.5, 0.], [1., 1.]];
//! let b = [[0., 0.5], [0.5, 0.5], [1., -0.5]];
//! let curve = |controls| BezierPairCurve {controls, weights: None, interval: [0., 1.]};
//! let query = CapturedBezierPair::capture_pair(curve(&a), curve(&b))?;
//! let policy = BezierPairPolicy {version: 1, roots: RootIsolationOptions::new(1e-9),
//!     signs: BezierPairSignOptions::derived(), requested_world_error: 1e-8};
//! let report = query.certify(policy, || false);
//! assert_eq!(report.completion(), BezierPairCompletion::Complete);
//! let proof = report.complete().unwrap();
//! assert_eq!(proof.contact(0).unwrap().point, [0.5, 0.25]);
//! assert_eq!(proof.contact(0).unwrap().paired_parameter_bounds, [0.5, 0.5]);
//! # Ok::<(), BezierPairCaptureFailure>(())
//! ```
mod constant_exclusion;
mod equations;
mod polynomial;
mod projection;
mod source;

mod fiber_row;

mod algebraic;
mod operation;
#[cfg(test)]
mod operation_tests;

mod construction_refine;
mod contact_construction;
mod graph_enclosure;
mod membership;
#[cfg(test)]
mod native_replay;
mod operation_failure;
#[cfg(test)]
mod operation_limits_tests;

mod exceptional;

mod source_certificate;

#[cfg(test)]
mod algebraic_world_tests;
mod candidate_certificate;
mod membership_certificate;
#[cfg(test)]
mod retention_tests;

#[cfg(test)]
mod refinement_tests;
mod sign_policy;
mod work_budget;
#[cfg(test)]
mod work_budget_tests;

#[cfg(test)]
mod sign_policy_tests;

mod query;
#[cfg(test)]
mod query_tests;

mod public_certificate;
mod public_constant_certificate;
mod public_graph_certificate;
mod public_query;
mod public_report;
pub use super::certificate_arithmetic::world::EnvelopeFailure as BezierPairWorldFailure;
pub use equations::DiagonalDivisionFailure as BezierPairDiagonalFailure;
pub use graph_enclosure::GraphEnvelopeFailure as BezierPairGraphFailure;
pub use membership_certificate::MembershipRule as BezierPairMembershipRule;
pub use operation_failure::PairOperationFailure as BezierPairOperationFailure;
pub use public_certificate::{BezierPairCertificate, CertifiedBezierPairContact};
pub use public_constant_certificate::{BezierPairConstantExclusion, BezierPairSignStage};
pub use public_graph_certificate::{
    BezierPairGraphCertificate, BezierPairMembership, BezierPairProjection,
};
pub use public_query::BezierPairCurve;
pub use public_report::{BezierPairFailure, BezierPairReport};
pub use query::{CapturedBezierPair, PairQueryPolicy as BezierPairPolicy};
pub use sign_policy::{
    EffectivePairSigns as BezierPairEffectiveSigns, PairSignAdmission as BezierPairSignAdmission,
    PairSignOptions as BezierPairSignOptions, PairSignPolicyFailure as BezierPairSignFailure,
};
pub use source::PairSourceFailure as BezierPairCaptureFailure;
pub use work_budget::PairWorkReceipt as BezierPairWork;

mod coefficient_view;
pub use coefficient_view::{ExactPairCoefficient, ExactPairPolynomial};

mod completion;
pub use completion::BezierPairCompletion;

mod public_parameter_enclosure;
pub use public_parameter_enclosure::ExactBezierPairParameterEnclosure;
#[cfg(test)]
mod exact_parameter_retention_tests;

mod native_parameter_construction;
pub use native_parameter_construction::{
    MeasuredPairParameter, PairParameterConstructionFailure, PairParameterDecision,
};
#[cfg(test)]
mod native_parameter_construction_tests;
#[cfg(test)]
mod rational_circle_tests;

mod carrier;
mod carrier_spans;
mod public_carrier;
pub use carrier::BezierPairCarrierKind;
pub use public_carrier::{BezierPairCarrierParameter, BezierPairCommonCarrier};
