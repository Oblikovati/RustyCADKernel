// SPDX-License-Identifier: GPL-2.0-only
//! The constructed point bounds the proven joint root, rather than a residual.
use super::super::{
    RootBracket, RootIsolation,
    certificate_arithmetic::{
        ExactRatio,
        world::{EnvelopeFailure, HomogeneousCurve, WorldEnvelope},
    },
};
use super::{
    graph_enclosure::{self, PairedParameterEnvelope},
    operation::{GraphTask, PairContact, PairOperationFailure},
    polynomial::ExactCoefficientPolynomial,
};
impl GraphTask<'_> {
    pub(super) fn construct(
        &self,
        polynomial: &ExactCoefficientPolynomial,
        roots: &RootIsolation,
        accepted: Vec<bool>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Vec<PairContact>, PairOperationFailure> {
        let source = &self.sources[0];
        let curve = HomogeneousCurve::from_controls(&source.controls, source.weights.as_deref());
        roots
            .roots()
            .iter()
            .zip(accepted)
            .enumerate()
            .filter(|(_, (_, flag))| *flag)
            .map(|(index, (root, _))| {
                self.construct_one(polynomial, &curve, (index, root.clone()), cancelled)
            })
            .collect()
    }
    fn construct_one(
        &self,
        polynomial: &ExactCoefficientPolynomial,
        curve: &HomogeneousCurve,
        candidate: (usize, RootBracket),
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<PairContact, PairOperationFailure> {
        let (candidate_index, mut root) = candidate;
        loop {
            let world = construction_attempt(curve, &root, cancelled)?;
            if let Some(u_bounds) = self.accepted_bounds(&root, &world, cancelled)? {
                return Ok(owned_contact(
                    (candidate_index, root),
                    u_bounds,
                    world.unwrap(),
                ));
            }
            root = self.refine_or_limit(polynomial, root, world, cancelled)?;
        }
    }
    fn refine_or_limit(
        &self,
        polynomial: &ExactCoefficientPolynomial,
        root: RootBracket,
        world: Option<WorldEnvelope>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<RootBracket, PairOperationFailure> {
        if root.bounds().0 == root.bounds().1 {
            return Err(resolution_limit(self.requested_error, root, world));
        }
        self.refine_contact_root(polynomial, &root, cancelled)
    }
    fn accepted_bounds(
        &self,
        root: &RootBracket,
        world: &Option<WorldEnvelope>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Option<PairedParameterEnvelope>, PairOperationFailure> {
        let Some(_world) = world.as_ref().filter(|world| self.certified_world(world)) else {
            return Ok(None);
        };
        graph_attempt(self, root, cancelled)
    }
    fn certified_world(&self, world: &WorldEnvelope) -> bool {
        let limit = ExactRatio::from_finite(self.requested_error);
        // The exact radius proves acceptance; the outward reported radius must also meet the request.
        world.error_squared <= limit.multiply(&limit)
            && world.achieved_error <= self.requested_error
    }
}
fn construction_attempt(
    curve: &HomogeneousCurve,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<WorldEnvelope>, PairOperationFailure> {
    match curve.enclose(root, cancelled) {
        Ok(world) => Ok(Some(world)),
        Err(EnvelopeFailure::DenominatorUnresolved | EnvelopeFailure::Binary64Overflow) => Ok(None),
        Err(cause) => Err(PairOperationFailure::World(cause)),
    }
}
fn graph_attempt(
    task: &GraphTask<'_>,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<PairedParameterEnvelope>, PairOperationFailure> {
    use graph_enclosure::GraphEnvelopeFailure;
    match graph_enclosure::enclose_u(task.projection, root, cancelled) {
        Ok(bounds) => Ok(Some(bounds)),
        Err(
            GraphEnvelopeFailure::UnresolvedLeadingInterval
            | GraphEnvelopeFailure::Binary64Overflow,
        ) => Ok(None),
        Err(cause) => Err(PairOperationFailure::Graph(cause)),
    }
}
fn resolution_limit(
    requested_error: f64,
    root: RootBracket,
    world: Option<WorldEnvelope>,
) -> PairOperationFailure {
    let enclosure = world.map(|world| (world.bounds, world.point, world.achieved_error));
    PairOperationFailure::WorldResolution {
        requested_error,
        last_root: Box::new(root),
        enclosure,
    }
}

fn owned_contact(
    candidate: (usize, RootBracket),
    u_bounds: PairedParameterEnvelope,
    world: WorldEnvelope,
) -> PairContact {
    let (candidate_index, root) = candidate;
    PairContact {
        candidate_index,
        root,
        u_bounds: u_bounds.rounded,
        exact_u: u_bounds.exact,
        world,
    }
}
