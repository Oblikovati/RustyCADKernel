// SPDX-License-Identifier: GPL-2.0-only
//! Borrow immutable certificate fields so source provenance cannot be detached.
use super::super::super::{RootBracket, RootIsolation, RootIsolationOptions};
use super::types::*;
impl BezierIncidenceSource {
    /// Read requested independent domain policy; example: `source.domain_sign()`.
    pub fn domain_sign(&self) -> BezierDomainSignOptions {
        self.domain_sign
    }
    /// Read the retained `curve_id`; example: `result.curve_id()`.
    pub fn curve_id(&self) -> u64 {
        self.curve_id
    }
    /// Read the retained `line_id`; example: `result.line_id()`.
    pub fn line_id(&self) -> u64 {
        self.line_id
    }
    /// Read the retained `controls`; example: `result.controls()`.
    pub fn controls(&self) -> &[[f64; 2]] {
        &self.controls
    }
    /// Read the retained `weights`; example: `result.weights()`.
    pub fn weights(&self) -> Option<&[f64]> {
        self.weights.as_deref()
    }
    /// Read the retained `line`; example: `result.line()`.
    pub fn line(&self) -> [[f64; 2]; 2] {
        self.line
    }
    /// Read the retained `interval`; example: `result.interval()`.
    pub fn interval(&self) -> [f64; 2] {
        self.interval
    }
    /// Read the retained `domain`; example: `result.domain()`.
    pub fn domain(&self) -> BezierLineDomain {
        self.domain
    }
    /// Read the retained `root_options`; example: `result.root_options()`.
    pub fn root_options(&self) -> RootIsolationOptions {
        self.root_options
    }
    /// Read the retained `requested_world_error`; example: `result.requested_world_error()`.
    pub fn requested_world_error(&self) -> f64 {
        self.requested_world_error
    }
}
impl CertifiedBezierIncidence {
    /// Read the retained `support_root_index`; example: `result.support_root_index()`.
    pub fn support_root_index(&self) -> usize {
        self.support_root_index
    }
    /// Read the retained `parameter`; example: `result.parameter()`.
    pub fn parameter(&self) -> &RootBracket {
        &self.parameter
    }
    /// Read the retained `world_bounds`; example: `result.world_bounds()`.
    pub fn world_bounds(&self) -> [[f64; 2]; 2] {
        self.world_bounds
    }
    /// Read the retained `point`; example: `result.point()`.
    pub fn point(&self) -> [f64; 2] {
        self.point
    }
    /// Read the retained `achieved_world_error`; example: `result.achieved_world_error()`.
    pub fn achieved_world_error(&self) -> f64 {
        self.achieved_world_error
    }
}
impl BezierIncidenceReport {
    /// Read effective derived degrees; example: `report.degrees().normal()`.
    pub fn degrees(&self) -> BezierIncidenceDegrees {
        self.degrees
    }
    /// Read denominator root-freedom proof; example: `report.denominator_isolation().roots().is_empty()`.
    pub fn denominator_isolation(&self) -> &RootIsolation {
        &self.denominator_isolation
    }
    /// Read the retained `source`; example: `result.source()`.
    pub fn source(&self) -> &BezierIncidenceSource {
        &self.source
    }
    /// Read the retained `support_isolation`; example: `result.support_isolation()`.
    pub fn support_isolation(&self) -> &RootIsolation {
        &self.support_isolation
    }
    /// Read the retained `in_domain`; example: `result.in_domain()`.
    pub fn in_domain(&self) -> &[bool] {
        &self.in_domain
    }
    /// Read the retained `contacts`; example: `result.contacts()`.
    pub fn contacts(&self) -> &[CertifiedBezierIncidence] {
        &self.contacts
    }
    /// Read the retained `bisections`; example: `result.bisections()`.
    pub fn bisections(&self) -> u16 {
        self.bisections
    }
}
impl BezierIncidenceDegrees {
    /// Distinguish skipped signs from zero polynomials; example: `degrees.domain_sign_applied()`.
    pub fn domain_sign_applied(&self) -> bool {
        self.effective_domain_limit.is_some()
    }
    /// Read source basis degree; example: `degrees.source_basis()`.
    pub fn source_basis(&self) -> usize {
        self.source_basis
    }
    /// Read admitted root normal degree; example: `degrees.normal()`.
    pub fn normal(&self) -> usize {
        self.normal
    }
    /// Read admitted denominator degree; example: `degrees.denominator()`.
    pub fn denominator(&self) -> usize {
        self.denominator
    }
    /// Read actual origin conversion degree; example: `degrees.origin_conversion()`.
    pub fn origin_conversion(&self) -> Option<usize> {
        self.origin_conversion
    }
    /// Read actual terminal conversion degree; example: `degrees.terminal_conversion()`.
    pub fn terminal_conversion(&self) -> Option<usize> {
        self.terminal_conversion
    }
    /// Read post-product origin sign degree; example: `degrees.origin_sign_product()`.
    pub fn origin_sign_product(&self) -> Option<usize> {
        self.origin_sign_product
    }
    /// Read post-product terminal sign degree; example: `degrees.terminal_sign_product()`.
    pub fn terminal_sign_product(&self) -> Option<usize> {
        self.terminal_sign_product
    }
    /// Read applied endpoint cap, None when signs were skipped; example: `degrees.effective_domain_limit()`.
    pub fn effective_domain_limit(&self) -> Option<u16> {
        self.effective_domain_limit
    }
}
impl BezierIncidenceFailure {
    /// Read the retained `source`; example: `result.source()`.
    pub fn source(&self) -> &BezierIncidenceSource {
        &self.evidence.source
    }
    /// Read the retained `cause`; example: `result.cause()`.
    pub fn cause(&self) -> &BezierIncidenceCause {
        &self.evidence.cause
    }
    /// Read the retained `support_isolation`; example: `result.support_isolation()`.
    pub fn support_isolation(&self) -> Option<&RootIsolation> {
        self.evidence.support_isolation.as_ref()
    }
    /// Read the retained `last_root`; example: `result.last_root()`.
    pub fn last_root(&self) -> Option<&RootBracket> {
        self.evidence.last_root.as_ref()
    }
}

impl BezierConstructionEnclosure {
    /// Read conservative coordinate bounds; example: `enclosure.world_bounds()`.
    pub fn world_bounds(&self) -> [[f64; 2]; 2] {
        self.world_bounds
    }
    /// Read the candidate whose error was measured; example: `enclosure.point()`.
    pub fn point(&self) -> [f64; 2] {
        self.point
    }
    /// Read outward Euclidean error; example: `enclosure.achieved_world_error()`.
    pub fn achieved_world_error(&self) -> f64 {
        self.achieved_world_error
    }
}
