// SPDX-License-Identifier: GPL-2.0-only
pub(crate) use super::super::current_binding::BoundPrismChart as MeasuredPrismChart;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismMeasureOptions {
    pub cap_area: PolynomialIntegralOptions,
    pub signed_volume: PolynomialIntegralOptions,
}
// This owns captured coefficients and operation-local identities only, never Shape/Handle aliases.
pub(crate) struct PrismMeasureInspection {
    pub signed_jacobian: rustycad_tkmath::predicates::roots::DyadicBound,
    pub region: RegionCertificate,
    pub charts: Vec<MeasuredPrismChart>,
    pub measures: PrismMeasureSubset,
    pub pending: [PendingMeasure; 2],
}
pub(crate) struct PrismMeasureSubset {
    pub first_cap: CertifiedPolynomialPlaneMeasure,
    pub last_cap: CertifiedPolynomialPlaneMeasure,
    pub signed_volume: CertifiedPolynomialMeasure,
    pub extrusion: [f64; 3],
    pub requested: PrismMeasureOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingMeasure {
    SideArea,
    TotalArea,
}
// Private refusal preserves legacy typed causes; these may carry mutable diagnostic Shapes.
// No failure here is an immutable published operation result or a partially accepted body.
pub(crate) enum PrismMeasureFailure {
    Region(RegionCertificateCause),
    Classification(Box<CandidateCause>),
    Chart(PrismSurfaceCause),
    Body(PrismCompletion),
    Cap {
        role: PrismImageRole,
        cause: PolynomialPlaneError,
    },
    Sweep(PolynomialIntegralError),
    Cancelled,
}
impl From<RegionCertificateCause> for PrismMeasureFailure {
    fn from(cause: RegionCertificateCause) -> Self {
        // Preserve the exact failed source hypothesis for this private refusal.
        Self::Region(cause)
    }
}
impl From<PrismSurfaceCause> for PrismMeasureFailure {
    fn from(cause: PrismSurfaceCause) -> Self {
        // Keep representation and proof-stage context instead of a generic rejection.
        Self::Chart(cause)
    }
}
impl From<PrismCompletion> for PrismMeasureFailure {
    fn from(cause: PrismCompletion) -> Self {
        // Legacy body diagnostics remain separate from immutable scalar evidence.
        Self::Body(cause)
    }
}
pub(super) type MeasureCheck<T = ()> = Result<T, PrismMeasureFailure>;

impl From<super::super::current_binding::PrismBindingFailure> for PrismMeasureFailure {
    fn from(cause: super::super::current_binding::PrismBindingFailure) -> Self {
        // Structural extraction preserves every previous measurement failure variant.
        use super::super::current_binding::PrismBindingFailure as Binding;
        match cause {
            Binding::Region(cause) => Self::Region(cause),
            Binding::Classification(cause) => Self::Classification(cause),
            Binding::Chart(cause) => Self::Chart(cause),
            Binding::Body(cause) => Self::Body(cause),
            Binding::Cancelled => Self::Cancelled,
        }
    }
}

#[cfg(test)]
mod binding_failure_tests {
    use super::super::super::current_binding::PrismBindingFailure as Binding;
    use super::*;

    #[test]
    fn shared_region_failure_keeps_exact_old_payload() {
        let region = RegionCertificateCause::MissingEvidence {
            hypothesis: "owned missing",
        };
        assert!(matches!(
            PrismMeasureFailure::from(Binding::from(region)),
            PrismMeasureFailure::Region(RegionCertificateCause::MissingEvidence {
                hypothesis: "owned missing"
            })
        ));
    }

    #[test]
    fn shared_chart_failure_keeps_exact_old_payload() {
        let chart = PrismSurfaceCause::Count {
            entity: "owned trim",
            actual: 3,
            expected: 4,
        };
        assert!(matches!(
            PrismMeasureFailure::from(Binding::from(chart)),
            PrismMeasureFailure::Chart(PrismSurfaceCause::Count {
                entity: "owned trim",
                actual: 3,
                expected: 4
            })
        ));
    }

    #[test]
    fn shared_body_and_cancellation_failures_keep_old_variants() {
        assert!(matches!(
            PrismMeasureFailure::from(Binding::from(PrismCompletion::Incomplete)),
            PrismMeasureFailure::Body(PrismCompletion::Incomplete)
        ));
        assert!(matches!(
            PrismMeasureFailure::from(Binding::Cancelled),
            PrismMeasureFailure::Cancelled
        ));
    }

    #[test]
    fn shared_classification_failure_keeps_original_phase() {
        use crate::sketch_region_candidates::CandidatePhase;
        let cause = Box::new(CandidateCause::Cancelled(CandidatePhase::Capture));
        let PrismMeasureFailure::Classification(cause) =
            PrismMeasureFailure::from(Binding::Classification(cause))
        else {
            panic!("expected retained classification failure")
        };
        assert!(matches!(
            *cause,
            CandidateCause::Cancelled(CandidatePhase::Capture)
        ));
    }
}
