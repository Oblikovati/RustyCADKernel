// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region_prism::exact_tolerance::StoredToleranceCap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OrderedPrismValidationOptions {
    pub version: u32,
    pub stored_cap: StoredToleranceCap,
}
impl OrderedPrismValidationOptions {
    /// Select exact validation with an explicit metadata policy; example: `Options::v1(cap)`.
    pub(crate) fn v1(stored_cap: StoredToleranceCap) -> Self {
        Self {
            version: 1,
            stored_cap,
        }
    }
}

/// Exact proof for the currently admitted domain; no sampled or generic body mode is implied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderedPrismValidationMode {
    ExactSingleJordanPolynomialPrism,
}
