// SPDX-License-Identifier: GPL-2.0-only
//! Explicit requests cannot silently inherit an unlimited metadata budget.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToleranceQuantity {
    StoredMetadataCap,
    WorldSpatialBudget,
    RawStoredMetadata,
    EffectiveAccessor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ToleranceScalarError {
    pub quantity: ToleranceQuantity,
    pub actual: f64,
}

impl std::fmt::Display for ToleranceScalarError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:?} {:?}; expected finite nonnegative scalar",
            self.quantity, self.actual
        )
    }
}

impl std::error::Error for ToleranceScalarError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StoredToleranceLimit(f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum StoredToleranceCap {
    NoLimit,
    Report(StoredToleranceLimit),
    Refuse(StoredToleranceLimit),
}

impl StoredToleranceCap {
    /// Record exceedances without suppressing them; example: `StoredToleranceCap::report(0.125)`.
    pub(crate) fn report(maximum: f64) -> Result<Self, ToleranceScalarError> {
        finite_nonnegative(ToleranceQuantity::StoredMetadataCap, maximum)?;
        Ok(Self::Report(StoredToleranceLimit(maximum)))
    }

    /// Refuse metadata above an explicit cap; example: `StoredToleranceCap::refuse(0.125)`.
    pub(crate) fn refuse(maximum: f64) -> Result<Self, ToleranceScalarError> {
        finite_nonnegative(ToleranceQuantity::StoredMetadataCap, maximum)?;
        Ok(Self::Refuse(StoredToleranceLimit(maximum)))
    }

    /// Preserve the exact metadata cap for reports; example: `cap.maximum()`.
    pub(crate) fn maximum(self) -> Option<f64> {
        match self {
            Self::NoLimit => None,
            Self::Report(StoredToleranceLimit(cap)) | Self::Refuse(StoredToleranceLimit(cap)) => {
                Some(cap)
            }
        }
    }

    fn decision(self, raw: f64) -> StoredCapDecision {
        match self {
            Self::NoLimit => StoredCapDecision::ExplicitNoLimit,
            Self::Report(StoredToleranceLimit(cap)) if raw > cap => StoredCapDecision::Reported,
            Self::Refuse(StoredToleranceLimit(cap)) if raw > cap => StoredCapDecision::Refused,
            _ => StoredCapDecision::Within,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoredCapDecision {
    Within,
    Reported,
    Refused,
    ExplicitNoLimit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismToleranceRequest {
    pub stored_cap: StoredToleranceCap,
    world_spatial_budget: f64,
}

impl PrismToleranceRequest {
    /// Keep budgets independent; example: `PrismToleranceRequest::new(cap, 1e-8)`.
    pub(crate) fn new(
        stored_cap: StoredToleranceCap,
        world_spatial_budget: f64,
    ) -> Result<Self, ToleranceScalarError> {
        finite_nonnegative(ToleranceQuantity::WorldSpatialBudget, world_spatial_budget)?;
        Ok(Self {
            stored_cap,
            world_spatial_budget,
        })
    }

    /// Return the unchanged metric request; example: `request.world_spatial_budget()`.
    pub(crate) fn world_spatial_budget(self) -> f64 {
        self.world_spatial_budget
    }

    /// Inspect metadata without interpreting it as world error; example: `request.observe(0., 1e-7)`.
    pub(crate) fn observe(
        self,
        raw: f64,
        effective: f64,
    ) -> Result<StoredToleranceObservation, ToleranceScalarError> {
        finite_nonnegative(ToleranceQuantity::RawStoredMetadata, raw)?;
        finite_nonnegative(ToleranceQuantity::EffectiveAccessor, effective)?;
        Ok(StoredToleranceObservation {
            raw,
            effective,
            policy: self.stored_cap,
            decision: self.stored_cap.decision(raw),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StoredToleranceObservation {
    pub raw: f64,
    pub effective: f64,
    pub policy: StoredToleranceCap,
    pub decision: StoredCapDecision,
}

fn finite_nonnegative(
    quantity: ToleranceQuantity,
    actual: f64,
) -> Result<(), ToleranceScalarError> {
    if !actual.is_finite() || actual < 0. {
        return Err(ToleranceScalarError { quantity, actual });
    }
    Ok(())
}
