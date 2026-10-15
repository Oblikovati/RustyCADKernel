// SPDX-License-Identifier: GPL-2.0-only
//! Stage composition over one prepared region; each child keeps its own admission and binding
//! checks, and a refusal carries the child's own class.
use super::*;
use crate::sketch_region_prism::publication::PrismPublicationClass;
use crate::sketch_region_prism::publication::annular::{
    AnnularPublicationRequest, PublishedAnnularPrism, publish_annular_prism,
};
use crate::sketch_region_prism::region_certificate::annular_binding::{
    AnnularPrismBinding, bind_annular_prism_v2,
};
use crate::sketch_region_prism::region_certificate::embedding::{
    AnnularEmbeddingEvidence, inspect_bound_annular_embedding,
};
use crate::sketch_region_prism::region_certificate::two_loop::{
    TwoLoopRefusalClass, TwoLoopRegionCertificate, TwoLoopRegionOutcome, inspect_two_loop_region,
};

type StageCheck<T> = Result<T, StageRefusal>;

/// Region proof, binding, names, embedding, publication, volume and owned body, in order.
pub(super) fn construct(
    prepared: &PreparedAnnularPrism,
    region: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    stages: &mut StageProgress<'_>,
) -> StageCheck<Box<OwnedAnnularPrism>> {
    let certificate = certify_region(region, stages)?;
    let binding = bind(region, certificate, receipt, stages)?;
    stages.stage = AnnularOperationStage::Naming;
    let names = issue_annular_names(prepared.request(), region, receipt, &binding)
        .map_err(|cause| naming_refusal(&cause))?;
    let embedding = embed(prepared, region, receipt, binding, stages)?;
    stages.stage = AnnularOperationStage::Publication;
    let request = AnnularPublicationRequest {
        region,
        receipt,
        bindings: &names.publication,
        copied: &names.copied,
        historical: &names.historical,
        requested: prepared.request().policy.publication,
    };
    let published =
        publish_annular_prism(&request, embedding, stages).map_err(publication_refusal)?;
    let volume = volume(prepared, &published, stages)?;
    stages.stage = AnnularOperationStage::Body;
    body::project(prepared.request(), &published, &volume, names.dictionary)
}
fn class(value: TwoLoopRefusalClass) -> PrismDiagnosticClass {
    match value {
        TwoLoopRefusalClass::Incomplete => PrismDiagnosticClass::Incomplete,
        TwoLoopRefusalClass::Failed => PrismDiagnosticClass::Failed,
        TwoLoopRefusalClass::Cancelled => PrismDiagnosticClass::Cancelled,
    }
}
fn certify_region(
    region: &TwoLoopRegionRequest<'_, '_>,
    stages: &mut StageProgress<'_>,
) -> StageCheck<Box<TwoLoopRegionCertificate>> {
    stages.stage = AnnularOperationStage::Region;
    let refused = |class| StageRefusal::new(AnnularOperationStage::Region, class);
    match inspect_two_loop_region(region, stages) {
        TwoLoopRegionOutcome::Inspected(certificate) => Ok(certificate),
        TwoLoopRegionOutcome::Incomplete(_) => Err(refused(PrismDiagnosticClass::Incomplete)),
        TwoLoopRegionOutcome::Failed(_) => Err(refused(PrismDiagnosticClass::Failed)),
        TwoLoopRegionOutcome::Cancelled(_) => Err(refused(PrismDiagnosticClass::Cancelled)),
    }
}
fn bind(
    region: &TwoLoopRegionRequest<'_, '_>,
    certificate: Box<TwoLoopRegionCertificate>,
    receipt: &PrismReceipt,
    stages: &mut StageProgress<'_>,
) -> StageCheck<AnnularPrismBinding> {
    stages.stage = AnnularOperationStage::Binding;
    bind_annular_prism_v2(region, certificate, receipt, stages).map_err(|failure| {
        StageRefusal::new(
            AnnularOperationStage::Binding,
            class(failure.classification()),
        )
    })
}
fn naming_refusal(cause: &super::super::names::AnnularNameCause<'_>) -> StageRefusal {
    let class = match cause {
        super::super::names::AnnularNameCause::Native(_) => PrismDiagnosticClass::Failed,
        _ => PrismDiagnosticClass::Incomplete,
    };
    StageRefusal::new(AnnularOperationStage::Naming, class)
}
fn embed(
    prepared: &PreparedAnnularPrism,
    region: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    binding: AnnularPrismBinding,
    stages: &mut StageProgress<'_>,
) -> StageCheck<AnnularEmbeddingEvidence> {
    stages.stage = AnnularOperationStage::Embedding;
    let options = prepared.request().policy.embedding;
    inspect_bound_annular_embedding(region, receipt, binding, options, || stages.poll()).map_err(
        |failure| {
            StageRefusal::new(
                AnnularOperationStage::Embedding,
                class(failure.classification()),
            )
        },
    )
}
fn publication_refusal(
    failure: Box<crate::sketch_region_prism::publication::annular::AnnularPublicationFailure<'_>>,
) -> StageRefusal {
    let projection = project_annular_publication_refusal(failure);
    let diagnostic = projection.into_diagnostic();
    let class = match diagnostic.class() {
        PrismPublicationClass::Failed => PrismDiagnosticClass::Failed,
        PrismPublicationClass::Incomplete => PrismDiagnosticClass::Incomplete,
        PrismPublicationClass::Cancelled => PrismDiagnosticClass::Cancelled,
    };
    let complete = diagnostic.gaps().is_empty();
    let stage = AnnularOperationStage::Publication;
    StageRefusal::owned(
        stage,
        class,
        AnnularOperationDiagnostic::Publication(diagnostic),
        complete,
    )
}
fn volume(
    prepared: &PreparedAnnularPrism,
    published: &PublishedAnnularPrism,
    stages: &mut StageProgress<'_>,
) -> StageCheck<AnnularPrismVolume> {
    stages.stage = AnnularOperationStage::Volume;
    let orientation = published.validation().orientation();
    let version = prepared.request().policy.volume_version;
    let (join, measures) = (published.join(), published.measures());
    certify_annular_prism_volume(join, orientation, measures, version, || stages.poll()).map_err(
        |failure| {
            let class = match failure.cancelled() {
                true => PrismDiagnosticClass::Cancelled,
                false => PrismDiagnosticClass::Incomplete,
            };
            let stage = AnnularOperationStage::Volume;
            StageRefusal::owned(
                stage,
                class,
                AnnularOperationDiagnostic::Volume(failure),
                true,
            )
        },
    )
}
