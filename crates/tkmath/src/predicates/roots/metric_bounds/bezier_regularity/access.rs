// SPDX-License-Identifier: GPL-2.0-only
use super::super::{super::RootIsolation, MetricEmbedding};
use super::types::*;
impl BezierRegularitySource {
    /// Read the occurrence key; example: `source.curve_id()`.
    pub fn curve_id(&self) -> u64 {
        self.curve_id
    }
    /// Borrow retained poles; example: `source.poles()`.
    pub fn poles(&self) -> &[[f64; 3]] {
        &self.poles
    }
    /// Borrow retained signed weights; example: `source.weights()`.
    pub fn weights(&self) -> Option<&[f64]> {
        self.weights.as_deref()
    }
    /// Read the original separate affine coefficients; example: `source.placement()`.
    pub fn placement(&self) -> MetricEmbedding<3> {
        MetricEmbedding {
            linear: self.linear,
            origin: self.origin,
            scale: self.scale,
        }
    }
    /// Read the closed native interval; example: `source.interval()`.
    pub fn interval(&self) -> [f64; 2] {
        self.interval
    }
    /// Read requested version and unchanged root policy; example: `source.options()`.
    pub fn options(&self) -> BezierRegularityOptions {
        self.options
    }
}
impl BezierRegularityReport {
    /// Read complete classification status; example: `report.completion()`.
    pub fn completion(&self) -> BezierRegularityCompletion {
        BezierRegularityCompletion::Complete
    }
    /// Borrow owned source provenance; example: `report.source()`.
    pub fn source(&self) -> &BezierRegularitySource {
        &self.source
    }
    /// Borrow the empty complete denominator isolation; example: `report.denominator_isolation()`.
    pub fn denominator_isolation(&self) -> &RootIsolation {
        &self.denominator_isolation
    }
    /// Read admitted denominator degree; example: `report.denominator_degree()`.
    pub fn denominator_degree(&self) -> usize {
        self.denominator_degree
    }
    /// Read squared-speed numerator degree, None for the zero polynomial; example: `report.speed_degree()`.
    pub fn speed_degree(&self) -> Option<usize> {
        self.speed_degree
    }
    /// Distinguish regular, isolated stationary and identically stationary; example: `report.decision()`.
    pub fn decision(&self) -> &BezierRegularityDecision {
        &self.decision
    }
    /// Read total denominator and derivative subdivisions; example: `report.bisections()`.
    pub fn bisections(&self) -> u16 {
        self.bisections
    }
}
