// SPDX-License-Identifier: GPL-2.0-only
//! V2 is authored before any geometry; legacy V1 admission remains the common domain gate.
use super::super::{OwnedPolynomialPrismRequest, OwnedPrismAdmissionCause, names::*};
pub(super) fn admit_complete_names(
    request: &OwnedPolynomialPrismRequest,
) -> Result<SingleBoundaryNamePlan, OwnedPrismAdmissionCause> {
    let admitted = super::super::admission::admit_owned_prism(request)?;
    SingleBoundaryNamePlan::v2(admitted.identities(), admitted.selection())
        .map_err(OwnedPrismAdmissionCause::Names)
}
