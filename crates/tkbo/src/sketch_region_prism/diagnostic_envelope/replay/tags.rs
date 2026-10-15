// SPDX-License-Identifier: GPL-2.0-only
//! Explicit tags preserve classification and missing-evidence categories independently.
use super::super::*;
pub(in crate::sketch_region_prism::diagnostic_envelope) fn class(
    value: PrismDiagnosticClass,
) -> u8 {
    match value {
        PrismDiagnosticClass::Declined => 0,
        PrismDiagnosticClass::Incomplete => 1,
        PrismDiagnosticClass::Failed => 2,
        PrismDiagnosticClass::Cancelled => 3,
    }
}
pub(super) fn local_class(value: OwnedPreparationClass) -> u8 {
    match value {
        OwnedPreparationClass::Declined => 0,
        OwnedPreparationClass::Incomplete => 1,
        OwnedPreparationClass::Failed => 2,
        OwnedPreparationClass::Cancelled => 3,
    }
}
pub(super) fn payload(value: UnprojectedPreparationPayload) -> u8 {
    match value {
        UnprojectedPreparationPayload::Census => 0,
        UnprojectedPreparationPayload::ContactTransport => 1,
        UnprojectedPreparationPayload::RegionPrism => 2,
        UnprojectedPreparationPayload::StageCompletion => 3,
    }
}
pub(super) fn owner(value: RetainedPreparationStage) -> u8 {
    match value {
        RetainedPreparationStage::ConstructedSource => 0,
        RetainedPreparationStage::BoundaryInputs => 1,
        RetainedPreparationStage::Census => 2,
        RetainedPreparationStage::Partition => 3,
        RetainedPreparationStage::ContactTransport => 4,
        RetainedPreparationStage::Prism => 5,
        RetainedPreparationStage::RegionIdentities => 6,
    }
}
