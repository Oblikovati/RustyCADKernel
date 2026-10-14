// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;

/// Independent unchanged scalar policies; no child budget is allocated from the total cap.
/// Example: `CompletePrismMeasureOptions { subset, side_area, total_area }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CompletePrismMeasureOptions {
    pub subset: PrismMeasureOptions,
    pub side_area: PolynomialSideOptions,
    pub total_area: EnclosureSumOptions,
}
/// Owned scalar inspection, with complete current coefficient/trim/history operands.
/// Example: `let total = inspection.measures.total.enclosure();`.
pub(crate) struct CompletePrismMeasureInspection {
    pub binding: CurrentPrismBinding,
    pub measures: CompletePrismMeasures,
}
/// Original component reports are retained unchanged; total adds their enclosure endpoints.
/// Example: `let side_error = measures.side.absolute_error_bound();`.
pub(crate) struct CompletePrismMeasures {
    pub subset: PrismMeasureSubset,
    pub side: CertifiedPolynomialSideIntegral,
    pub total: CertifiedEnclosureSum,
    pub requested: CompletePrismMeasureOptions,
}
// Private failures may retain legacy mutable diagnostics; no partial successful body is returned.
pub(crate) enum CompletePrismMeasureFailure {
    Binding(Box<PrismBindingFailure>),
    Subset(Box<PrismMeasureFailure>),
    Side {
        source: PartitionOccurrenceIdentity,
        output: PartitionOccurrenceIdentity,
        cause: Box<PolynomialSideError>,
    },
    Total(EnclosureSumError),
}
impl From<PrismBindingFailure> for CompletePrismMeasureFailure {
    fn from(cause: PrismBindingFailure) -> Self {
        // Keep the shared binding refusal without reconstructing its provenance.
        Self::Binding(Box::new(cause))
    }
}
impl From<PrismMeasureFailure> for CompletePrismMeasureFailure {
    fn from(cause: PrismMeasureFailure) -> Self {
        // Subset arithmetic keeps its original typed quantity and policy.
        Self::Subset(Box::new(cause))
    }
}
