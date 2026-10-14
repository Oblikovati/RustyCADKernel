// SPDX-License-Identifier: GPL-2.0-only
//! Only this actual invocation can pair caller context with its publication failure or result.
use super::super::request::OwnedPolynomialPrismRequest;
use super::*;
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use crate::sketch_region_prism::frozen::PrismPublicationBindings;
use crate::sketch_region_prism::{PrismReceipt, publication::*};
pub(in crate::sketch_region_prism::owned_request) struct ResolvedPrismInvocation<'a> {
    prepared: &'a PreparedPolynomialPrism,
    plan: SingleBoundaryNamePlan,
    bridge: &'a PairedContactTransaction,
    receipt: &'a PrismReceipt,
    sources: Vec<ArrangementSource<'a>>,
    bindings: bindings::InvocationBindings,
}
pub(in crate::sketch_region_prism) struct InvokedPrismPublicationFailure<'a> {
    request: OwnedPolynomialPrismRequest,
    plan: SingleBoundaryNamePlan,
    bindings: &'a PrismPublicationBindings,
    failure: Box<PrismPublicationFailure<'a>>,
}
pub(in crate::sketch_region_prism::owned_request) struct ResolvedPrismPublication {
    request: OwnedPolynomialPrismRequest,
    plan: SingleBoundaryNamePlan,
    result: PublishedNamedPrism,
}
pub(in crate::sketch_region_prism::owned_request) fn resolve_invocation<'a>(
    prepared: &'a PreparedPolynomialPrism,
    plan: SingleBoundaryNamePlan,
    cancelled: &mut impl FnMut() -> bool,
) -> ResolveCheck<ResolvedPrismInvocation<'a>> {
    let roles = roster::resolve_stages(prepared, &plan, cancelled)?;
    let bindings = bindings::assemble(prepared, &plan, &roles)?;
    invocation_operands(prepared, plan, bindings)
}
fn invocation_operands<'a>(
    prepared: &'a PreparedPolynomialPrism,
    plan: SingleBoundaryNamePlan,
    bindings: bindings::InvocationBindings,
) -> ResolveCheck<ResolvedPrismInvocation<'a>> {
    let source = current::constructed_source(prepared)?;
    Ok(ResolvedPrismInvocation {
        prepared,
        plan,
        bridge: require_local(
            prepared.retained.bridge.as_ref(),
            "actual contact transaction",
        )?,
        receipt: current::receipt(prepared)?,
        sources: vec![source.source()],
        bindings,
    })
}

impl ResolvedPrismInvocation<'_> {
    pub(in crate::sketch_region_prism::owned_request) fn publish<'a>(
        &'a self,
        progress: &mut dyn PrismPublicationProgress,
    ) -> Result<ResolvedPrismPublication, Box<InvokedPrismPublicationFailure<'a>>> {
        let requested = self.prepared.retained.request.clone();
        let request = self.publication_request();
        match publish_named_prism(&request, progress) {
            Ok(result) => Ok(ResolvedPrismPublication {
                request: requested,
                plan: self.plan.clone(),
                result,
            }),
            Err(failure) => Err(Box::new(InvokedPrismPublicationFailure {
                request: requested,
                plan: self.plan.clone(),
                bindings: &self.bindings.publication,
                failure,
            })),
        }
    }
    fn publication_request(&self) -> PrismPublicationRequest<'_> {
        PrismPublicationRequest {
            source: PrismPublicationSource {
                bridge: self.bridge,
                sources: &self.sources,
                receipt: self.receipt,
                bindings: &self.bindings.publication,
                contact: &self.bindings.contact,
            },
            copied: &self.bindings.copied,
            historical: &self.bindings.historical,
            requested: self.prepared.retained.request.policy.publication,
        }
    }
}
impl ResolvedPrismPublication {
    /// Transfer the sealed payload after projection; `invoked.into_parts()` retains the exact request.
    pub(in crate::sketch_region_prism::owned_request) fn into_parts(
        self,
    ) -> (OwnedPolynomialPrismRequest, PublishedNamedPrism) {
        (self.request, self.result)
    }

    /// Borrow only this invocation's accepted child; `invoked.result()`.
    pub(in crate::sketch_region_prism::owned_request) fn result(&self) -> &PublishedNamedPrism {
        &self.result
    }
    /// Read the predeclared dictionary actually used by this invocation; `invoked.plan()`.
    pub(in crate::sketch_region_prism::owned_request) fn plan(&self) -> &SingleBoundaryNamePlan {
        &self.plan
    }
    /// Read exact caller input captured by this invocation; `invoked.request()`.
    pub(in crate::sketch_region_prism::owned_request) fn request(
        &self,
    ) -> &OwnedPolynomialPrismRequest {
        &self.request
    }
}
impl<'a> InvokedPrismPublicationFailure<'a> {
    /// Read exact caller input captured by this invocation; `invoked.request()`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedPolynomialPrismRequest {
        &self.request
    }
    /// Read the predeclared dictionary actually used by this invocation; `invoked.plan()`.
    pub(in crate::sketch_region_prism) fn plan(&self) -> &SingleBoundaryNamePlan {
        &self.plan
    }
    /// Borrow names from the actual failed invocation; `packet.bindings()` outlives the packet.
    pub(in crate::sketch_region_prism) fn bindings(&self) -> &'a PrismPublicationBindings {
        self.bindings
    }
    /// Retain the original outer class and nested cause; `packet.failure()`.
    pub(in crate::sketch_region_prism) fn failure(&self) -> &PrismPublicationFailure<'a> {
        &self.failure
    }
    /// Consume this packet into its actual local child failure; `packet.into_failure()`.
    pub(in crate::sketch_region_prism) fn into_failure(self) -> Box<PrismPublicationFailure<'a>> {
        self.failure
    }
}
