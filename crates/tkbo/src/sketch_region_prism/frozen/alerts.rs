// SPDX-License-Identifier: GPL-2.0-only
//! Only a concrete dataless alert can be projected without losing an owner-bearing payload.
use super::types::CaptureResult;
use super::types::PrismFreezeCause;
use crate::bop_algo::{AlertIntersectionFailed, NativeAlertObservation};
use rustycad_tkernel::message::downcast_alert;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FrozenNativeAlert {
    /// checker_si::observed_alerts captures only Fail gravity; this unit alert has no other payload.
    IntersectionFailedAtFailGravity,
}
pub(super) fn freeze_alert(alert: &NativeAlertObservation) -> CaptureResult<FrozenNativeAlert> {
    if downcast_alert::<AlertIntersectionFailed>(&alert.original).is_some() {
        return Ok(FrozenNativeAlert::IntersectionFailedAtFailGravity);
    }
    Err(PrismFreezeCause::UnsupportedDiagnostic {
        stage: Some(crate::sketch_region_validation::ValidationStage::SelfIntersection),
        actual: "unprojectable native alert type/payload",
    })
}
