// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed native projection inputs separate shared scalar checks from V1 orchestration.
use super::*;
pub(in crate::sketch_region_prism::frozen) struct NativeHistoryInputs<'a> {
    pub bridge: &'a crate::sketch_region_contact_transport::PairedContactTransaction,
    pub receipt: &'a crate::sketch_region_prism::PrismReceipt,
    pub publication: &'a PrismPublicationBindings,
}
impl PrismHistoryRequest<'_, '_> {
    pub(super) fn native_inputs(&self) -> NativeHistoryInputs<'_> {
        NativeHistoryInputs {
            bridge: self.bridge,
            receipt: self.receipt,
            publication: self.publication,
        }
    }
}
