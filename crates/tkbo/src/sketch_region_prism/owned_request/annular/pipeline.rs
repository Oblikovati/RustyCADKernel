// SPDX-License-Identifier: GPL-2.0-only
//! New private orchestration reuses existing constructors and stages; it cannot publish a body.
use super::super::classification::{callback_class, preparation_class};
use super::stages::enter;
use super::*;
use crate::sketch_region_prism::region_certificate::{borrowed_batch, temporary_view};
use crate::sketch_region_prism::{
    PrismCompletion, PrismPhase, PrismRequest, retain_selected_prism,
};

/// Prepare actual annular diagnostics from owned inputs; `prepare_owned_annular_prism(request, progress)`.
/// Region certification, binding, join and publication remain separate downstream gates.
pub(in crate::sketch_region_prism) fn prepare_owned_annular_prism(
    request: OwnedAnnularPrismRequest,
    progress: &mut dyn OwnedPrismProgress,
) -> Result<PreparedAnnularPrism, Box<AnnularPreparationFailure>> {
    let mut retained = AnnularPreparationRetained::new(request);
    let mut callbacks = OwnedPrismCallbacks::new(progress);
    let outcome = execute(&mut retained, &mut callbacks);
    finish(retained, outcome, callbacks.stop.take())
}
fn execute(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> PreparationCheck<()> {
    super::admission::admit(&retained.request).map_err(AnnularPreparationCause::Admission)?;
    enter(retained, callbacks, OwnedPrismPhase::Admission)?;
    enter(retained, callbacks, OwnedPrismPhase::Construction)?;
    super::stages::construct(retained)?;
    super::stages::census(retained, callbacks)?;
    super::stages::partition(retained, callbacks)?;
    super::stages::contacts(retained, callbacks)?;
    let bridge = local(retained.bridge.as_ref(), "contact transaction")?;
    retained.identities = Some(super::roles::identities(&retained.request, bridge)?);
    prism(retained, callbacks)
}
/// Sweep the actual batch through the existing retained prism builder.
fn prism(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> PreparationCheck<()> {
    let phase = OwnedPrismPhase::Prism(PrismPhase::Admission);
    enter(retained, callbacks, phase)?;
    let bridge = local(retained.bridge.as_ref(), "contact transaction")?;
    let batch = local(borrowed_batch(bridge), "actual completed candidate batch")?;
    let requested = prism_request(&retained.request);
    let receipt = retain_selected_prism(temporary_view(batch), requested, callbacks);
    let class = refused_class(&receipt.completion);
    retained.prism = Some(receipt);
    match class {
        None => Ok(()),
        Some(class) => Err(OwnedPreparationCause::StageUnavailable { class }.into()),
    }
}
/// The incomplete diagnostic body is the expected pre-certification state.
fn refused_class(completion: &PrismCompletion) -> Option<OwnedPreparationClass> {
    match completion {
        PrismCompletion::Incomplete => None,
        PrismCompletion::Failed(_) => Some(OwnedPreparationClass::Failed),
        PrismCompletion::Cancelled(_) => Some(OwnedPreparationClass::Cancelled),
        PrismCompletion::NoBody(_) => Some(OwnedPreparationClass::Incomplete),
    }
}
fn prism_request(request: &OwnedAnnularPrismRequest) -> PrismRequest {
    let [x, y, z] = request.extrusion;
    PrismRequest {
        seed: request.material_seed,
        extrusion: rustycad_tkmath::gp::Vector3::new(x, y, z),
        roots: request.policy.classification_roots,
        spatial_limit: request.policy.spatial_limit,
        parameter_limit: request.policy.parameter_limit,
    }
}
fn finish(
    retained: AnnularPreparationRetained,
    outcome: PreparationCheck<()>,
    callback: Option<OwnedPrismCallbackStop>,
) -> Result<PreparedAnnularPrism, Box<AnnularPreparationFailure>> {
    let outcome = outcome.and_then(|_| match callback {
        Some(_) => Err(OwnedPreparationCause::Callback.into()),
        None => Ok(()),
    });
    if let Err(cause) = outcome {
        return Err(failed(retained, cause, callback));
    }
    let complete = retained.bridge.is_some() && retained.identities.is_some();
    if !complete || retained.prism.is_none() {
        let missing = OwnedPreparationCause::MissingLocalState {
            expected: "prepared annular state",
        };
        return Err(failed(retained, missing.into(), None));
    }
    Ok(prepared(retained))
}
fn prepared(mut retained: AnnularPreparationRetained) -> PreparedAnnularPrism {
    let bridge = retained.bridge.take().expect("checked prepared bridge");
    let identities = retained
        .identities
        .take()
        .expect("checked prepared identities");
    let receipt = retained.prism.take().expect("checked prepared receipt");
    PreparedAnnularPrism {
        retained,
        bridge,
        identities,
        receipt,
    }
}
fn failed(
    retained: AnnularPreparationRetained,
    cause: AnnularPreparationCause,
    callback: Option<OwnedPrismCallbackStop>,
) -> Box<AnnularPreparationFailure> {
    let class = match (&callback, &cause) {
        (Some(stop), _) => callback_class(&stop.cause),
        (None, AnnularPreparationCause::Stage(cause)) => preparation_class(cause),
        (None, AnnularPreparationCause::Admission(AnnularAdmissionCause::Owned(cause))) => {
            preparation_class(&OwnedPreparationCause::Admission(cause.clone()))
        }
        (None, AnnularPreparationCause::Admission(_)) => OwnedPreparationClass::Failed,
        (None, AnnularPreparationCause::Role { .. }) => OwnedPreparationClass::Incomplete,
    };
    let phase = retained.phase;
    Box::new(AnnularPreparationFailure {
        class,
        cause,
        callback,
        phase,
        retained,
    })
}
