// SPDX-License-Identifier: GPL-2.0-only
//! The constructed point bounds the proven joint root, rather than a residual.
use super::super::{RootBracket, RootIsolation, certificate_arithmetic::world::HomogeneousCurve};
use super::{
    graph_enclosure,
    operation::{GraphTask, PairContact, PairOperationFailure},
};
impl GraphTask<'_> {
    pub(super) fn construct(
        &self,
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
            .filter(|(_, flag)| *flag)
            .map(|(root, _)| self.construct_one(&curve, root, cancelled))
            .collect()
    }
    fn construct_one(
        &self,
        curve: &HomogeneousCurve,
        root: &RootBracket,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<PairContact, PairOperationFailure> {
        let world = curve
            .enclose(root, cancelled)
            .map_err(PairOperationFailure::World)?;
        if world.achieved_error > self.requested_error {
            return Err(PairOperationFailure::WorldResolution);
        }
        let u_bounds = graph_enclosure::enclose_u(self.projection, root, cancelled)
            .map_err(PairOperationFailure::Graph)?;
        Ok(PairContact {
            root: root.clone(),
            u_bounds,
            world,
        })
    }
}
