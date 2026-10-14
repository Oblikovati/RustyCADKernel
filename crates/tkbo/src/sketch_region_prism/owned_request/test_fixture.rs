// SPDX-License-Identifier: GPL-2.0-only
//! Caller-owned polynomial and explicit policies; no geometry or naming fixture orchestration.
use super::{names::*, progress::*, request::*};
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap,
    frozen::{
        complete_history::PrismHistoryOptions, joined_measures::JoinedPrismMeasureOptions,
        ordered_validation::OrderedPrismValidationOptions, proof_join::PrismProofJoinOptions,
    },
    publication::PrismPublicationOptions,
    region_certificate::{
        CompletePrismMeasureOptions, PrismMeasureOptions, embedding::PrismEmbeddingOptions,
    },
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::Plane3,
    predicates::roots::{RootIsolationOptions, metric_bounds::*},
};
pub(super) fn request() -> OwnedPolynomialPrismRequest {
    OwnedPolynomialPrismRequest {
        version: 1,
        operation: CallerOperationId(257),
        sources: vec![boundary()],
        declared_plane: Plane3::default(),
        selection: SemanticSelectionIntent::BoundedRegionContaining { seed: [1., 1.] },
        extrusion: [0., 0., 2.],
        policy: policy(),
    }
}
fn boundary() -> OwnedPolynomialBoundary {
    OwnedPolynomialBoundary {
        source: CallerBoundaryId(0),
        revision: CallerSourceRevision(71),
        closure: CallerVertexId(7000),
        native_start: CallerDirectedUseId(901),
        native_end: CallerDirectedUseId(19),
        world_poles: vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]],
        reversed_traversal: false,
    }
}
fn policy() -> OwnedPrismPolicy {
    let roots = RootIsolationOptions::new(0.001);
    OwnedPrismPolicy {
        census: crate::sketch_region_certification::CensusPolicy {
            roots,
            max_world_error: 1e-8,
        },
        source: source_policy(roots),
        contacts: contact_policy(roots),
        classification_roots: roots,
        spatial_limit: 1e-8,
        parameter_limit: 1e-8,
        publication: publication_options(),
    }
}
fn publication_options() -> PrismPublicationOptions {
    PrismPublicationOptions {
        version: 1,
        join: PrismProofJoinOptions::v1(PrismEmbeddingOptions::v1(BezierRegularityOptions::v1(
            RootIsolationOptions::new(1e-6),
        ))),
        validation: OrderedPrismValidationOptions::v1(StoredToleranceCap::refuse(1e-6).unwrap()),
        history: PrismHistoryOptions::v1(),
        measures: measure_options(),
    }
}
fn measure_options() -> JoinedPrismMeasureOptions {
    JoinedPrismMeasureOptions::v1(CompletePrismMeasureOptions {
        subset: subset_options(),
        side_area: PolynomialSideOptions {
            behavior_version: 1,
            max_degree: 16,
            max_subdivisions: 4096,
            absolute_error_limit: 0.01,
        },
        total_area: EnclosureSumOptions {
            behavior_version: 1,
            max_components: 3,
            absolute_error_limit: 0.02,
        },
    })
}
#[derive(Default)]
pub(super) struct RequestObserver {
    pub seen: Vec<OwnedPrismCheckpoint>,
    pub stop: Option<usize>,
    pub fail: Option<usize>,
}
impl OwnedPrismProgress for RequestObserver {
    fn checkpoint(&mut self, checkpoint: OwnedPrismCheckpoint) -> Result<bool, Failure> {
        self.seen.push(checkpoint);
        if self.fail == Some(checkpoint.ordinal) {
            return Err(Failure::new("owned request injected callback failure"));
        }
        Ok(self.stop == Some(checkpoint.ordinal))
    }
}
pub(super) fn prepared(
    request: OwnedPolynomialPrismRequest,
) -> super::diagnostics::PreparedPolynomialPrism {
    super::pipeline::prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
        .unwrap_or_else(|failure| {
            panic!(
                "expected prepared diagnostic; actual class={:?} phase={:?}",
                failure.class, failure.retained.phase
            )
        })
}

fn source_policy(
    roots: RootIsolationOptions,
) -> crate::sketch_region_source_conformance::SourceConformancePolicy {
    crate::sketch_region_source_conformance::SourceConformancePolicy {
        roots,
        original_plane_limit: 0.,
        construction_limit: 0.,
    }
}
fn contact_policy(
    roots: RootIsolationOptions,
) -> crate::sketch_region_contact_transport::ContactTransportPolicy {
    crate::sketch_region_contact_transport::ContactTransportPolicy {
        roots,
        native_parameter_limits: [1e-8; 2],
        world_endpoint_limit: 1e-8,
    }
}
fn subset_options() -> PrismMeasureOptions {
    let integral = PolynomialIntegralOptions {
        behavior_version: 1,
        max_degree: 16,
        max_curves: 1,
        absolute_error_limit: 1e-12,
    };
    PrismMeasureOptions {
        cap_area: integral,
        signed_volume: integral,
    }
}
