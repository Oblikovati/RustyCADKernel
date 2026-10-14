// SPDX-License-Identifier: GPL-2.0-only
//! One authored source binding feeds the join, validation, history and measures of one annulus.
use super::*;

/// Explicit child policies; `AnnularPublicationOptions { version: 1, .. }` never widens a child.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularPublicationOptions {
    pub version: u32,
    pub join: AnnularFrozenOptions,
    pub validation: AnnularValidationOptions,
    pub history: PrismHistoryOptions,
    pub measures: AnnularMeasureOptions,
}
/// Borrowed operation-local inputs; all names are authored before lookup.
/// Example: `AnnularPublicationRequest { region, receipt, bindings, copied, historical, requested }`.
pub(in crate::sketch_region_prism) struct AnnularPublicationRequest<'a, 'r, 's> {
    pub region: &'a TwoLoopRegionRequest<'r, 's>,
    pub receipt: &'a PrismReceipt,
    pub bindings: &'a PrismPublicationBindings,
    pub copied: &'a PrismCopyBindings,
    pub historical: &'a AnnularHistoryBindings,
    pub requested: AnnularPublicationOptions,
}
impl<'a, 'r, 's> AnnularPublicationRequest<'a, 'r, 's> {
    /// The history request over exactly this source binding and the given join.
    pub(super) fn history<'join>(
        &self,
        joined: &'join AnnularFrozenDomain,
    ) -> AnnularHistoryRequest<'a, 'r, 's, 'join> {
        AnnularHistoryRequest {
            region: self.region,
            receipt: self.receipt,
            publication: self.bindings,
            copied: self.copied,
            historical: self.historical,
            joined,
            requested: self.requested.history,
        }
    }
}
