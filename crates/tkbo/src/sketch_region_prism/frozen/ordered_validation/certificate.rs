// SPDX-License-Identifier: GPL-2.0-only
//! A complete validation receipt deliberately has no operation publication or body-valid flag.
use super::{joined_orientation::*, topology::*, *};
use crate::sketch_region_prism::exact_tolerance::*;

pub(crate) enum OrderedPrismValidationOutcome {
    CompleteValidation(Box<OrderedPrismValidationCertificate>),
    Failed(Box<OrderedPrismValidationFailure>),
    Incomplete(Box<OrderedPrismValidationFailure>),
    Cancelled(Box<OrderedPrismValidationFailure>),
}
pub(crate) struct OrderedPrismValidationCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) requested: OrderedPrismValidationOptions,
    pub(super) trace: OrderedPrismStageTrace,
    pub(super) topology: Box<FrozenTopologyCertificate>,
    pub(super) orientation: JoinedPrismOrientationCertificate,
    pub(super) conformance: JoinedPrismConformance,
    pub(super) intersection: JoinedPrismIntersectionCertificate,
    pub(super) tolerance: JoinedPrismTolerance,
    pub(super) historical: super::super::coverage::FrozenPrismCoverage,
}
impl OrderedPrismValidationCertificate {
    /// Select explicit current proof authority, independently of the historical checker; example: `proof.mode()`.
    pub(crate) fn mode(&self) -> OrderedPrismValidationMode {
        // This operation has one admitted exact domain and no sampled fallback.
        OrderedPrismValidationMode::ExactSingleJordanPolynomialPrism
    }
    /// Match the one immutable source; example: `validation.binds(&join)`.
    pub(crate) fn binds(&self, join: &JoinedPrismProof) -> bool {
        let expected = join.replay_bytes();
        self.source.as_ref() == expected
    }
    /// Retain the exact behavior and explicit cap; example: `validation.requested().version`.
    pub(crate) fn requested(&self) -> OrderedPrismValidationOptions {
        // The cap remains caller-selected; no default policy is substituted.
        self.requested
    }
    /// Borrow ordered completed coverage; example: `validation.trace().status(stage)`.
    pub(crate) fn trace(&self) -> &OrderedPrismStageTrace {
        // Historical mapped coverage remains separate from this current ledger.
        &self.trace
    }
    /// Borrow independent combinatorial proof; example: `validation.topology().achieved_error()`.
    pub(crate) fn topology(&self) -> &FrozenTopologyCertificate {
        // Incidence has no metric budget; preserve its separate proof type.
        self.topology.as_ref()
    }
    /// Borrow named outward and trim directions; example: `validation.orientation().faces()`.
    pub(crate) fn orientation(&self) -> &JoinedPrismOrientationCertificate {
        // Named directions remain tied to authored uses, not face enumeration.
        &self.orientation
    }
    /// Borrow the single conformance calculation and exact residual bounds; example: `validation.conformance().evidence()`.
    pub(crate) fn conformance(&self) -> &JoinedPrismConformance {
        // Tolerance borrows this same proof; no second geometry evaluation occurs.
        &self.conformance
    }
    /// Borrow the complete same-body embedding table; example: `validation.intersection().incidences()`.
    pub(crate) fn intersection(&self) -> &JoinedPrismIntersectionCertificate {
        // The complete table distinguishes rim contacts from disjoint pairs.
        &self.intersection
    }
    /// Borrow metadata decisions without repeating geometry; example: `validation.tolerance().metadata()`.
    pub(crate) fn tolerance(&self) -> &JoinedPrismTolerance {
        // Reported exceedances must remain inspectable after completion.
        &self.tolerance
    }
    /// Retain mapped historical failure independently; example: `validation.historical().findings`.
    pub(crate) fn historical(&self) -> &super::super::coverage::FrozenPrismCoverage {
        // A fresh exact result cannot erase a prior checker finding.
        &self.historical
    }
}
