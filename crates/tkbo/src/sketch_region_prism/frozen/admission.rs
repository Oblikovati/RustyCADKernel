// SPDX-License-Identifier: GPL-2.0-only
//! Required compatibility diagnostics must be imported or explicitly refused, never erased.
use super::super::{PrismCompletion, PrismReceipt};
use super::{bindings::missing, types::*};

pub(super) fn admit_receipt(receipt: &PrismReceipt) -> Result<(), PrismFreezeCause> {
    if !matches!(receipt.completion, PrismCompletion::Incomplete) {
        return Err(unsupported(
            "non-incomplete source completion carries unprojected cause",
        ));
    }
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .ok_or(missing("diagnostic body"))?;
    if !diagnostic.transport_gaps.is_empty() || receipt.cancelled_query.is_some() {
        return Err(unsupported(
            "present history transport or cancelled-query diagnostic",
        ));
    }
    admit_scalar_observations(receipt)?;
    admit_report(receipt)
}
fn admit_scalar_observations(receipt: &PrismReceipt) -> Result<(), PrismFreezeCause> {
    if receipt
        .region
        .as_ref()
        .is_some_and(|region| region.decision.is_err())
    {
        return Err(unsupported("present RegionCertificateInspection error"));
    }
    if receipt
        .affine_chart
        .as_ref()
        .is_some_and(|chart| chart.decision.is_err())
    {
        return Err(unsupported("present PrismAffineChartObservation error"));
    }
    // Successful historical selection/affine observations remain explicitly unbound limitations.
    Ok(())
}
fn admit_report(receipt: &PrismReceipt) -> Result<(), PrismFreezeCause> {
    for gravity in rustycad_tkernel::message::Gravity::ALL {
        if !receipt.input.report.get_alerts(gravity).is_empty() {
            return Err(unsupported("present owner-bearing input Report alert"));
        }
    }
    Ok(())
}
pub(super) fn unsupported(actual: &'static str) -> PrismFreezeCause {
    PrismFreezeCause::UnsupportedDiagnostic {
        stage: None,
        actual,
    }
}
