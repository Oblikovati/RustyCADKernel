// SPDX-License-Identifier: GPL-2.0-only
//! Public callers capture actual sources, not caller-asserted identities or coefficients after solving.
use super::{
    BezierPairCaptureFailure, BezierPairPolicy, CapturedBezierPair,
    public_report::BezierPairReport, source::OwnedBezierSource,
};
/// Borrowed planar Bernstein controls, optional rational weights and exact binary64 trim.
#[derive(Clone, Copy, Debug)]
pub struct BezierPairCurve<'a> {
    /// Source Bernstein controls; the basis remains `[0,1]`, including extrapolated trims.
    pub controls: &'a [[f64; 2]],
    /// Optional homogeneous weights; `None` means unit weights.
    pub weights: Option<&'a [f64]>,
    /// Closed source parameter bounds, independent of display tessellation.
    pub interval: [f64; 2],
}
impl CapturedBezierPair {
    /// Own both actual source curves before querying. Example: `CapturedBezierPair::capture_pair(left, right)?`.
    pub fn capture_pair(
        left: BezierPairCurve<'_>,
        right: BezierPairCurve<'_>,
    ) -> Result<Self, BezierPairCaptureFailure> {
        let left = OwnedBezierSource::capture(left.controls, left.weights, left.interval)
            .map_err(|cause| super::source::capture_context(0, cause))?;
        let right = OwnedBezierSource::capture(right.controls, right.weights, right.interval)
            .map_err(|cause| super::source::capture_context(1, cause))?;
        Ok(Self::distinct(left, right))
    }
    /// Read the retained controls and trims without an owning snapshot. Example: `query.captured_curves()[0].controls`.
    pub fn captured_curves(&self) -> [BezierPairCurve<'_>; 2] {
        curve_views(self.sources())
    }
    /// Execute one owned-source query with explicit policies and a global split budget. Example: `query.certify(policy, || false)`.
    pub fn certify(
        &self,
        policy: BezierPairPolicy,
        mut cancelled: impl FnMut() -> bool,
    ) -> BezierPairReport {
        let outcome = self.evaluate(policy, &mut cancelled);
        // The report owns its captured binding even when the caller drops this query.
        BezierPairReport::new(outcome)
    }
}

pub(super) fn curve_views(sources: &[OwnedBezierSource; 2]) -> [BezierPairCurve<'_>; 2] {
    sources.each_ref().map(|source| BezierPairCurve {
        controls: &source.controls,
        weights: source.weights.as_deref(),
        interval: source.interval,
    })
}
