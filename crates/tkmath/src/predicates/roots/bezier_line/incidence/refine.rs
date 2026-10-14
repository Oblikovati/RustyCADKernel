// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{
    DyadicBound, RootBracket, RootIsolation, RootIsolationOptions, polynomial::IntegerPolynomial,
};
use super::super::{ray, solve};
use super::types::*;
use crate::predicates::roots::certificate_arithmetic::{
    ExactRatio,
    world::{EnvelopeFailure, HomogeneousCurve, WorldEnvelope},
};

pub(super) struct ConstructedRoot {
    pub root: RootBracket,
    pub envelope: WorldEnvelope,
}
pub(super) struct RootConstruction<'a> {
    curve: &'a HomogeneousCurve,
    normal: &'a IntegerPolynomial,
    options: RootIsolationOptions,
    squared_limit: ExactRatio,
    requested: f64,
}
enum ConstructionStep {
    Complete(WorldEnvelope),
    Refine(RootBracket),
}
impl<'a> RootConstruction<'a> {
    pub(super) fn new(
        curve: &'a HomogeneousCurve,
        normal: &'a IntegerPolynomial,
        options: RootIsolationOptions,
        requested: f64,
    ) -> Self {
        let limit = ExactRatio::from_finite(requested);
        Self {
            curve,
            normal,
            options,
            squared_limit: limit.multiply(&limit),
            requested,
        }
    }
    pub(super) fn construct(
        &self,
        mut root: RootBracket,
        used: &mut u16,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ConstructedRoot, (BezierIncidenceCause, Box<RootBracket>)> {
        loop {
            match self.advance(&root, used, cancelled) {
                Ok(ConstructionStep::Complete(envelope)) => {
                    return Ok(ConstructedRoot { root, envelope });
                }
                Ok(ConstructionStep::Refine(next)) => root = next,
                Err(cause) => return Err((cause, Box::new(root))),
            }
        }
    }
    fn certified(&self, envelope: &Option<WorldEnvelope>) -> bool {
        envelope
            .as_ref()
            .is_some_and(|candidate| candidate.error_squared <= self.squared_limit)
    }
    fn advance(
        &self,
        root: &RootBracket,
        used: &mut u16,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<ConstructionStep, BezierIncidenceCause> {
        if cancelled() {
            return Err(BezierIncidenceCause::Cancelled);
        }
        let envelope = construction_attempt(self.curve, root, cancelled)?;
        if self.certified(&envelope) {
            return Ok(ConstructionStep::Complete(envelope.unwrap()));
        }
        if root.bounds().0 == root.bounds().1 {
            return Err(resolution_limit(self.requested, envelope));
        }

        refine_one(self.normal, root, self.options, used, cancelled).map(ConstructionStep::Refine)
    }
}
fn construction_attempt(
    curve: &HomogeneousCurve,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<WorldEnvelope>, BezierIncidenceCause> {
    match curve.enclose(root, cancelled) {
        Ok(envelope) => Ok(Some(envelope)),
        Err(EnvelopeFailure::DenominatorUnresolved | EnvelopeFailure::Binary64Overflow) => Ok(None),
        Err(EnvelopeFailure::Cancelled) => Err(BezierIncidenceCause::Cancelled),
    }
}
fn resolution_limit(requested: f64, envelope: Option<WorldEnvelope>) -> BezierIncidenceCause {
    let last_enclosure = envelope.map(|envelope| BezierConstructionEnclosure {
        world_bounds: envelope.bounds,
        point: envelope.point,
        achieved_world_error: envelope.achieved_error,
    });
    BezierIncidenceCause::ConstructionResolutionLimit {
        requested_error: requested,
        effective_error_limit: requested,
        last_enclosure,
    }
}
fn refine_one(
    normal: &IntegerPolynomial,
    root: &RootBracket,
    options: RootIsolationOptions,
    used: &mut u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootBracket, BezierIncidenceCause> {
    let remaining = remaining_policy(root, options, *used)?;
    let bounds = [root.bounds().0.clone(), root.bounds().1.clone()];
    let width = root.width().midpoint(&DyadicBound::from_f64(0.).unwrap());
    let refined = solve::solve_polynomial(normal.clone(), bounds, width, remaining, cancelled)
        .map_err(|cause| BezierIncidenceCause::Classification(ray::normal_error(cause)))?;
    *used += refined.bisections();
    unique_refined_root(refined)
}
fn remaining_policy(
    _root: &RootBracket,
    mut options: RootIsolationOptions,
    used: u16,
) -> Result<RootIsolationOptions, BezierIncidenceCause> {
    if used == options.max_bisections {
        return Err(BezierIncidenceCause::RefinementExhausted {
            limit: options.max_bisections,
        });
    }
    options.max_bisections -= used;
    Ok(options)
}
fn unique_refined_root(refined: RootIsolation) -> Result<RootBracket, BezierIncidenceCause> {
    if refined.roots().len() != 1 {
        return Err(BezierIncidenceCause::RefinementInvariant);
    }
    Ok(refined.roots()[0].clone())
}
