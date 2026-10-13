// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::RootIsolationOptions;
use super::super::super::BezierLineQuery;
/// Owned scalar input; physical identity is supplied within one owning operation.
/// The token does not prove geometry ownership or persistent naming.
#[derive(Clone, Debug)]
pub struct CapturedBezierWireInput {
    pub(super) physical_source: u64,
    pub(super) controls: Vec<[f64; 2]>,
    pub(super) weights: Option<Vec<f64>>,
    pub(super) interval: [f64; 2],
}
impl CapturedBezierWireInput {
    /// Capture values before callbacks; query validation occurs in assembly order.
    /// Example: `CapturedBezierWireInput::capture(7, &[[0.,0.]], None, [0.,1.])`.
    pub fn capture(
        physical_source: u64,
        controls: &[[f64; 2]],
        weights: Option<&[f64]>,
        interval: [f64; 2],
    ) -> Self {
        Self {
            physical_source,
            controls: controls.to_vec(),
            weights: weights.map(<[f64]>::to_vec),
            interval,
        }
    }
    pub(super) fn request(
        &self,
        line: [[f64; 2]; 2],
        options: RootIsolationOptions,
    ) -> BezierLineQuery<'_> {
        BezierLineQuery {
            controls: &self.controls,
            weights: self.weights.as_deref(),
            line,
            interval: self.interval,
            options,
        }
    }
    pub(super) fn same_payload(&self, other: &Self) -> bool {
        self.physical_source == other.physical_source
            && self.controls.len() == other.controls.len()
            && self.controls.iter().flatten().map(|v| v.to_bits()).eq(other
                .controls
                .iter()
                .flatten()
                .map(|v| v.to_bits()))
            && same_weights(&self.weights, &other.weights)
            && self.interval.map(f64::to_bits) == other.interval.map(f64::to_bits)
    }
}
fn same_weights(left: &Option<Vec<f64>>, right: &Option<Vec<f64>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => a
            .iter()
            .map(|v| v.to_bits())
            .eq(b.iter().map(|v| v.to_bits())),
        _ => false,
    }
}
/// One traversal of an owned physical segment; direction is not a cache key.
#[derive(Clone, Debug)]
pub struct CapturedBezierWireOccurrence {
    /// Current traversal identity used for errors and boundary attribution.
    pub id: u64,
    /// Immutable owned scalar capture.
    pub input: CapturedBezierWireInput,
    /// Reverse the exact endpoints and increasing-parameter contribution.
    pub reversed: bool,
}
