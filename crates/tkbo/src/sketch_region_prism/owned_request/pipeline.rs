// SPDX-License-Identifier: GPL-2.0-only
//! New private orchestration uses existing constructors and operation stages without fixture helpers.
use super::{
    admission::*, classification::*, construction::*, diagnostics::*, progress::*, request::*,
};
/// Prepare actual private diagnostics from owned inputs; `prepare_owned_polynomial_prism(request, progress)`.
/// This cannot publish a body; ordered validation, complete naming/history and publication remain required.
pub(crate) fn prepare_owned_polynomial_prism(
    request: OwnedPolynomialPrismRequest,
    progress: &mut dyn OwnedPrismProgress,
) -> Result<PreparedPolynomialPrism, Box<OwnedPrismPreparationFailure>> {
    let mut retained = PreparationRetained::new(request);
    match admit_owned_prism(&retained.request) {
        Ok(plan) => retained.plan = Some(plan),
        Err(cause) => {
            return Err(failed(
                retained,
                OwnedPreparationCause::Admission(cause),
                None,
            ));
        }
    }
    let mut callbacks = OwnedPrismCallbacks::new(progress);
    let outcome = execute(&mut retained, &mut callbacks);
    finish_preparation(retained, outcome, callbacks.stop.take())
}
fn execute(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Result<(), OwnedPreparationCause> {
    enter(retained, callbacks, OwnedPrismPhase::Admission)?;
    enter(retained, callbacks, OwnedPrismPhase::Construction)?;
    let source = local(retained.request.sources.first(), "one admitted source")?;
    retained.source =
        Some(construct_owned_polynomial(source).map_err(OwnedPreparationCause::Native)?);
    super::stages::census(retained, callbacks)?;
    super::partition::partition(retained, callbacks)?;
    super::stages::contacts(retained, callbacks)?;
    super::stages::prism(retained, callbacks)
}
pub(super) fn enter(
    retained: &mut PreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
    phase: OwnedPrismPhase,
) -> Result<(), OwnedPreparationCause> {
    retained.phase = phase;
    if callbacks.poll(phase) {
        return Err(OwnedPreparationCause::Callback);
    }
    Ok(())
}
fn finish_preparation(
    retained: PreparationRetained,
    outcome: Result<(), OwnedPreparationCause>,
    callback: Option<OwnedPrismCallbackStop>,
) -> Result<PreparedPolynomialPrism, Box<OwnedPrismPreparationFailure>> {
    if let Err(cause) = outcome {
        return Err(failed(retained, cause, callback));
    }
    if callback.is_some() {
        return Err(failed(retained, OwnedPreparationCause::Callback, callback));
    }
    Ok(PreparedPolynomialPrism { retained })
}
fn failed(
    retained: PreparationRetained,
    cause: OwnedPreparationCause,
    callback: Option<OwnedPrismCallbackStop>,
) -> Box<OwnedPrismPreparationFailure> {
    let class = callback
        .as_ref()
        .map(|s| callback_class(&s.cause))
        .unwrap_or_else(|| preparation_class(&cause));
    Box::new(OwnedPrismPreparationFailure {
        class,
        cause,
        callback,
        retained,
    })
}
