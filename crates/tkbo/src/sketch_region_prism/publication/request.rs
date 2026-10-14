// SPDX-License-Identifier: GPL-2.0-only
//! Callers supply semantic identities; publication never manufactures names from lookup order.
use super::source::PrismPublicationSource;
use crate::sketch_region_prism::frozen::{
    complete_history::*, joined_measures::*, lineage::PrismCopyBindings, ordered_validation::*,
    proof_join::*,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismPublicationOptions {
    pub version: u32,
    pub join: PrismProofJoinOptions,
    pub validation: OrderedPrismValidationOptions,
    pub history: PrismHistoryOptions,
    pub measures: JoinedPrismMeasureOptions,
}
pub(crate) struct PrismPublicationRequest<'a> {
    pub source: PrismPublicationSource<'a, 'a>,
    pub copied: &'a PrismCopyBindings,
    pub historical: &'a PrismHistoryBindings,
    pub requested: PrismPublicationOptions,
}
impl<'a> PrismPublicationRequest<'a> {
    pub(super) fn history_request<'join>(
        &self,
        joined: &'join JoinedPrismProof,
    ) -> PrismHistoryRequest<'a, 'join> {
        PrismHistoryRequest {
            bridge: self.source.bridge,
            sources: self.source.sources,
            receipt: self.source.receipt,
            publication: self.source.bindings,
            copied: self.copied,
            historical: self.historical,
            joined,
            requested: self.requested.history,
        }
    }
}
