// SPDX-License-Identifier: GPL-2.0-only
//! Test-only semantic authorship delegates to the existing reviewed history fixture.
use super::{progress::*, request::*, result::*, source::*};
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap,
    frozen::{
        complete_history::*,
        joined_measures::*,
        lineage::PrismCopyBindings,
        ordered_validation::*,
        proof_join::{operation_tests::JoinFixture, *},
    },
    region_certificate::{CompletePrismMeasureOptions, PrismMeasureOptions},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    EnclosureSumOptions, PolynomialSideOptions,
};

pub(super) struct PublicationFixture {
    pub operation: JoinFixture,
    pub copied: PrismCopyBindings,
    pub historical: PrismHistoryBindings,
    pub requested: PrismPublicationOptions,
}
impl PublicationFixture {
    pub fn new() -> Self {
        Self::from_operation(JoinFixture::new())
    }
    pub fn from_operation(operation: JoinFixture) -> Self {
        let lookup = operation.compose(|| false).ok().unwrap();
        let (copied, historical) = fixture::authored_history_inputs(&operation, &lookup);
        let requested = options(lookup.requested());
        Self {
            operation,
            copied,
            historical,
            requested,
        }
    }
    pub fn request<'a>(
        &'a self,
        sources: &'a [ArrangementSource<'a>],
    ) -> PrismPublicationRequest<'a> {
        PrismPublicationRequest {
            source: PrismPublicationSource {
                bridge: &self.operation.bridge,
                sources,
                receipt: &self.operation.receipt,
                bindings: &self.operation.authored,
                contact: &self.operation.contact,
            },
            copied: &self.copied,
            historical: &self.historical,
            requested: self.requested,
        }
    }
    pub fn publish(&self, progress: &mut dyn PrismPublicationProgress) -> PublishedNamedPrism {
        let sources =
            crate::sketch_region_partition_bindings::binding_sources(&self.operation.shapes);
        match super::publish_named_prism(&self.request(&sources), progress) {
            Ok(result) => result,
            Err(failure) => panic!(
                "expected complete publication, got {:?} at {:?}",
                failure.class, failure.phase
            ),
        }
    }
}
fn options(join: PrismProofJoinOptions) -> PrismPublicationOptions {
    PrismPublicationOptions {
        version: 1,
        join,
        validation: OrderedPrismValidationOptions::v1(StoredToleranceCap::refuse(1e-6).unwrap()),
        history: PrismHistoryOptions::v1(),
        measures: measures(),
    }
}
fn measures() -> JoinedPrismMeasureOptions {
    JoinedPrismMeasureOptions::v1(CompletePrismMeasureOptions {
        subset: PrismMeasureOptions {
            cap_area: Default::default(),
            signed_volume: Default::default(),
        },
        side_area: PolynomialSideOptions {
            absolute_error_limit: 0.01,
            ..Default::default()
        },
        total_area: EnclosureSumOptions {
            absolute_error_limit: 0.02,
            ..Default::default()
        },
    })
}
#[derive(Default)]
pub(super) struct PublicationObserver {
    pub seen: Vec<PrismPublicationCheckpoint>,
    pub stop: Option<PrismPublicationPhase>,
}
impl PrismPublicationProgress for PublicationObserver {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        Ok(self.stop == Some(point.phase))
    }
}

pub(super) fn publication_phases() -> [PrismPublicationPhase; 5] {
    [
        PrismPublicationPhase::Join,
        PrismPublicationPhase::Validation,
        PrismPublicationPhase::History,
        PrismPublicationPhase::Measures,
        PrismPublicationPhase::Commit,
    ]
}
