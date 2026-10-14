// SPDX-License-Identifier: GPL-2.0-only
//! Census and prism continue through the existing exact/general path; no new classifiers.
use super::{diagnostics::*, names::SemanticSelectionIntent, pipeline::enter, progress::*};
use crate::sketch_region_certification::{capture_boundary_inputs, enumerate_source_bound_census};
use crate::sketch_region_contact_transport::{ContactTransportPhase, transport_partition_contacts};
use crate::sketch_region_prism::{
    PrismCompletion, PrismPhase, PrismReceipt, PrismRequest, retain_source_bound_prism,
};
use rustycad_tkmath::gp::Vector3;
pub(super) fn census(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    enter(retained, callbacks, OwnedPrismPhase::SourceCapture)?;
    retained.inputs = Some(
        capture_boundary_inputs(
            &[retained.source()?.source()],
            retained.request.declared_plane,
        )
        .map_err(OwnedPreparationCause::Census)?,
    );
    enumerate(retained, callbacks)
}
fn enumerate(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    enter(retained, callbacks, OwnedPrismPhase::Census)?;
    let inputs = local(retained.inputs.take(), "captured boundary inputs")?;
    let result = enumerate_source_bound_census(
        inputs,
        retained.request.policy.census,
        retained.request.policy.source,
        || callbacks.poll(OwnedPrismPhase::Census),
    );
    retained.census = Some(result.map_err(OwnedPreparationCause::Census)?);
    Ok(())
}
pub(super) fn contacts(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    let phase = OwnedPrismPhase::Contact(ContactTransportPhase::SourceBinding);
    enter(retained, callbacks, phase)?;
    let census = local(retained.census.take(), "census")?;
    let partition = local(retained.partition.take(), "partition")?;
    let result = transport_partition_contacts(
        census,
        partition,
        retained.request.policy.contacts,
        &[retained.source()?.source()],
        |phase| callbacks.poll(OwnedPrismPhase::Contact(phase)),
    );
    retained.bridge = Some(result.map_err(OwnedPreparationCause::Contact)?);
    Ok(())
}
pub(super) fn prism(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    enter(
        retained,
        callbacks,
        OwnedPrismPhase::Prism(PrismPhase::Admission),
    )?;
    let receipt = observe_prism(retained, callbacks)?;
    let class = prism_class(&receipt.completion);
    let certified = receipt.region.as_ref().is_some_and(|r| r.decision.is_ok());
    retained.prism = Some(receipt);
    if !certified {
        return Err(OwnedPreparationCause::StageUnavailable { class });
    }
    Ok(())
}
fn observe_prism(
    retained: &PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<PrismReceipt, OwnedPreparationCause> {
    let bridge = local(retained.bridge.as_ref(), "contact transaction")?;
    retain_source_bound_prism(
        bridge,
        &[retained.source()?.source()],
        prism_request(&retained.request),
        callbacks,
    )
    .map_err(OwnedPreparationCause::PrismUnavailable)
}
fn prism_request(request: &super::request::OwnedPolynomialPrismRequest) -> PrismRequest {
    let SemanticSelectionIntent::BoundedRegionContaining { seed } = request.selection;
    let [x, y, z] = request.extrusion;
    PrismRequest {
        seed,
        extrusion: Vector3::new(x, y, z),
        roots: request.policy.classification_roots,
        spatial_limit: request.policy.spatial_limit,
        parameter_limit: request.policy.parameter_limit,
    }
}
fn prism_class(completion: &PrismCompletion) -> OwnedPreparationClass {
    match completion {
        PrismCompletion::Failed(_) => OwnedPreparationClass::Failed,
        PrismCompletion::Cancelled(_) => OwnedPreparationClass::Cancelled,
        PrismCompletion::Incomplete | PrismCompletion::NoBody(_) => {
            OwnedPreparationClass::Incomplete
        }
    }
}
