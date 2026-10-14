// SPDX-License-Identifier: GPL-2.0-only
//! Context constructors accept actual retained invocation state, never an arbitrary caller header.
use super::super::{
    frozen::proof_join::encode_joined_block,
    owned_request::request_replay::encode_owned_prism_request,
};
use super::*;
impl OwnedPrismDiagnosticContext {
    pub(super) fn from_preparation(failure: &OwnedPrismPreparationFailure) -> Self {
        let view = failure.diagnostic_context();
        let request = view.request().clone();
        let plan = view.plan().cloned();
        let replay = encode_context(&request, plan.as_ref());
        Self {
            request,
            plan,
            replay,
        }
    }
}
fn encode_context(
    request: &OwnedPolynomialPrismRequest,
    plan: Option<&SingleBoundaryNamePlan>,
) -> Vec<u8> {
    let mut bytes = b"Martensite.OwnedDiagnosticContext/v1\0".to_vec();
    encode_joined_block(&encode_owned_prism_request(request), &mut bytes);
    bytes.push(u8::from(plan.is_some()));
    if let Some(plan) = plan {
        let envelope = SemanticReplayEnvelope::new(plan.clone(), &[]);
        encode_joined_block(envelope.replay_bytes(), &mut bytes);
    }
    bytes
}
