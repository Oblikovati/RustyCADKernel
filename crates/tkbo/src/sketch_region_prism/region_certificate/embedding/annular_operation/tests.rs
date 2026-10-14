// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::annular_binding::tests::with_actual;
use super::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};
struct ContinueEmbeddingBinding;
impl AnnularBindingProgress for ContinueEmbeddingBinding {
    fn checkpoint(
        &mut self,
        _: AnnularBindingStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}

fn options() -> AnnularEmbeddingOptions {
    AnnularEmbeddingOptions {
        version: 2,
        regularity: AnnularRegularityOptions {
            boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12)),
            total_bisections: 0,
        },
    }
}
fn binding(
    request: &TwoLoopRegionRequest<'_, '_>,
    region: Box<super::super::super::two_loop::TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
) -> AnnularPrismBinding {
    bind_annular_prism_v2(request, region, receipt, &mut ContinueEmbeddingBinding)
        .unwrap_or_else(|_| panic!("actual annular binding"))
}
#[test]
fn actual_entry_composes_owned_domains_without_promoting_legacy_body_completion() {
    let evidence = with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        let evidence =
            inspect_bound_annular_embedding(request, receipt, bound, options(), || false)
                .unwrap_or_else(|_| panic!("actual annular embedding"));
        assert!(matches!(
            receipt.completion,
            crate::sketch_region_prism::PrismCompletion::Incomplete
        ));
        evidence
    });
    assert_eq!(evidence.requested, options());
    assert_eq!(evidence.regularity.boundaries.len(), 2);
    assert_eq!(evidence.incidences.len(), 10);
    assert_eq!(evidence.domains.first.trims.len(), 2);
    assert_eq!(evidence.binding.region().selected_candidate_id(), 301);
    assert!(
        evidence
            .binding
            .signed_jacobian()
            .compare_f64(2.)
            .unwrap()
            .is_eq()
    );
    static_assertions::assert_impl_all!(AnnularEmbeddingEvidence: Send, Sync);
    static_assertions::assert_impl_all!(AnnularEmbeddingOptions: Send, Sync);
}
#[test]
fn unsupported_version_refuses_before_cancellation_callback_and_keeps_request() {
    with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        let mut requested = options();
        requested.version = 9;
        let failure = inspect_bound_annular_embedding(request, receipt, bound, requested, || {
            panic!("version admission first")
        })
        .err()
        .unwrap();
        assert_eq!(failure.operation, request.identities.operation);
        assert_eq!(failure.selected_candidate, request.identities.selected);
        assert_eq!(failure.requested, requested);
        assert_eq!(failure.phase, AnnularEmbeddingPhase::Admission);
        assert!(matches!(
            failure.cause,
            AnnularEmbeddingCause::Version {
                actual: 9,
                expected: 2
            }
        ));
    });
}
