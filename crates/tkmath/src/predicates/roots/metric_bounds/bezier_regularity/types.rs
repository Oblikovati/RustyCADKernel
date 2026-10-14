// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{RootIsolation, RootIsolationOptions};
use super::super::MetricEmbedding;

/// Explicit version and shared root work policy for derivative regularity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierRegularityOptions {
    /// Only behavior version 1 is admitted.
    pub version: u32,
    /// Largest stored Bernstein degree admitted before power expansion, independent of reduced degrees.
    pub max_stored_degree: u16,
    /// One degree limit and total subdivision budget for denominator and squared speed.
    pub roots: RootIsolationOptions,
}
impl BezierRegularityOptions {
    /// Select behavior 1, initially copying the root degree cap into the separate stored cap; example: `BezierRegularityOptions::v1(roots)`.
    pub fn v1(roots: RootIsolationOptions) -> Self {
        Self {
            version: 1,
            max_stored_degree: roots.max_degree,
            roots,
        }
    }
}
/// Stored space-curve coefficients and placement; signed finite weights are admitted explicitly.
#[derive(Clone, Copy)]
pub struct BezierRegularityQuery<'a> {
    /// Caller occurrence key; coefficient evidence establishes the mathematical source.
    pub curve_id: u64,
    /// Nonempty stored three-coordinate Bernstein poles.
    pub poles: &'a [[f64; 3]],
    /// Matching signed finite weights; omitted weights are one.
    pub weights: Option<&'a [f64]>,
    /// Exact supplied affine placement; singular placements are not assumed invertible.
    pub placement: MetricEmbedding<3>,
    /// Closed native parameter interval inside `[0,1]`, without rescaling the basis.
    pub interval: [f64; 2],
    /// Explicit behavior and complete root-search policy.
    pub options: BezierRegularityOptions,
}
impl<'a> BezierRegularityQuery<'a> {
    /// Request a polynomial in its native frame; example: `BezierRegularityQuery::new(7, &poles, options)`.
    pub fn new(curve_id: u64, poles: &'a [[f64; 3]], options: BezierRegularityOptions) -> Self {
        Self {
            curve_id,
            poles,
            weights: None,
            interval: [0., 1.],
            options,
            placement: MetricEmbedding {
                linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
                origin: [0.; 3],
                scale: 1.,
            },
        }
    }
}
/// Owned input coefficients, exact placement and work policy for replay.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierRegularitySource {
    pub(super) curve_id: u64,
    pub(super) poles: Vec<[f64; 3]>,
    pub(super) weights: Option<Vec<f64>>,
    pub(super) linear: [[f64; 3]; 3],
    pub(super) origin: [f64; 3],
    pub(super) scale: f64,
    pub(super) interval: [f64; 2],
    pub(super) options: BezierRegularityOptions,
}
impl BezierRegularitySource {
    pub(super) fn retain(query: BezierRegularityQuery<'_>) -> Self {
        Self {
            curve_id: query.curve_id,
            poles: query.poles.to_vec(),
            weights: query.weights.map(<[f64]>::to_vec),
            linear: query.placement.linear,
            origin: query.placement.origin,
            scale: query.placement.scale,
            interval: query.interval,
            options: query.options,
        }
    }
}
/// Complete classification of the derivative, not whole-curve injectivity or body validity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BezierRegularityDecision {
    /// The nonnegative squared-speed numerator has no roots on the closed interval.
    Regular {
        /// Complete empty isolation of the squared-speed numerator.
        zero_speed_isolation: RootIsolation,
    },
    /// Every stationary parameter is retained; the parameterization is not regular.
    Stationary {
        /// All zero-speed parameters, with squared-numerator multiplicities.
        zero_speed_isolation: RootIsolation,
    },
    /// The derivative is identically zero; this is never an empty root list.
    EverywhereStationary,
}
/// Complete denominator and derivative evidence bound to owned input coefficients.
#[derive(Clone, Debug, PartialEq)]
pub struct BezierRegularityReport {
    pub(super) source: BezierRegularitySource,
    pub(super) denominator_isolation: RootIsolation,
    pub(super) denominator_degree: usize,
    pub(super) speed_degree: Option<usize>,
    pub(super) decision: BezierRegularityDecision,
    pub(super) bisections: u16,
}
/// Operation completion is distinct from whether the classified parameterization is regular.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierRegularityCompletion {
    /// A complete regular or stationary classification.
    Complete,
    /// Invalid coefficient, interval or policy.
    Invalid,
    /// Undefined rational curve or degree outside the explicit admission policy.
    Declined,
    /// Caller stopped the operation before complete certification.
    Cancelled,
    /// Exhausted or inconclusive required root proof.
    Incomplete,
}
