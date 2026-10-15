// SPDX-License-Identifier: GPL-2.0-only
//! Owned profile preparation through the existing general stages: capture, the ADR 0066 part B
//! carrier census, partition, candidate copies and contact transport.
use super::super::classification::{
    arrangement_class, callback_class, candidate_class, preparation_class,
};
use super::*;
use crate::sketch_region::{
    ArrangementCompletion, ArrangementPhase, PartitionOccurrenceIdentity, observe_partition_space,
};
use crate::sketch_region_candidates::observe_candidate_copies;
use crate::sketch_region_certification::{capture_boundary_inputs, enumerate_carrier_census};
use crate::sketch_region_contact_transport::{ContactTransportPhase, transport_partition_contacts};
use crate::sketch_region_partition_bindings::{PartitionBoundOutputs, bind_partition_outputs};
use crate::sketch_region_source_conformance::{capture_originals, capture_space_representation};

/// Actual stage cause; general child causes reuse the owned preparation vocabulary.
pub(in crate::sketch_region_prism) enum ProfilePreparationCause {
    Admission(ProfileAdmissionCause),
    Stage(OwnedPreparationCause),
}
/// Typed refusal; the request stays with it, no partial body exists.
pub(in crate::sketch_region_prism) struct ProfilePreparationFailure {
    pub class: OwnedPreparationClass,
    pub cause: ProfilePreparationCause,
    pub callback: Option<OwnedPrismCallbackStop>,
    pub phase: OwnedPrismPhase,
    pub request: OwnedProfileRequest,
}
/// Prepared profile: owned request, constructed edges and the contact transaction.
pub(in crate::sketch_region_prism) struct PreparedProfile {
    pub request: OwnedProfileRequest,
    pub edges: Vec<ConstructedProfileEdge>,
    pub bridge: PairedContactTransaction,
}
impl PreparedProfile {
    /// Borrow the constructed edges as arrangement sources in request order.
    pub(in crate::sketch_region_prism) fn sources(&self) -> Vec<ArrangementSource<'_>> {
        self.edges
            .iter()
            .map(ConstructedProfileEdge::source)
            .collect()
    }
}
type Check<T> = Result<T, ProfilePreparationCause>;
struct Retained<'a, 'p> {
    request: &'a OwnedProfileRequest,
    callbacks: OwnedPrismCallbacks<'p>,
    phase: OwnedPrismPhase,
}

/// Prepare an owned profile; `prepare_owned_profile(request, &mut progress)`.
/// The general loop region proof is a separate downstream gate.
pub(in crate::sketch_region_prism) fn prepare_owned_profile(
    request: OwnedProfileRequest,
    progress: &mut dyn OwnedPrismProgress,
) -> Result<PreparedProfile, Box<ProfilePreparationFailure>> {
    let mut retained = Retained {
        request: &request,
        callbacks: OwnedPrismCallbacks::new(progress),
        phase: OwnedPrismPhase::Admission,
    };
    let outcome = execute(&mut retained);
    let (phase, stop) = (retained.phase, retained.callbacks.stop.take());
    match (outcome, stop) {
        (Ok((edges, bridge)), None) => Ok(PreparedProfile {
            request,
            edges,
            bridge,
        }),
        (Ok(_), stop @ Some(_)) => Err(failed(request, cause_callback(), stop, phase)),
        (Err(cause), stop) => Err(failed(request, cause, stop, phase)),
    }
}
fn cause_callback() -> ProfilePreparationCause {
    ProfilePreparationCause::Stage(OwnedPreparationCause::Callback)
}
fn execute(
    retained: &mut Retained<'_, '_>,
) -> Check<(Vec<ConstructedProfileEdge>, PairedContactTransaction)> {
    super::admission::admit(retained.request).map_err(ProfilePreparationCause::Admission)?;
    retained.enter(OwnedPrismPhase::Admission)?;
    retained.enter(OwnedPrismPhase::Construction)?;
    let edges = super::construction::construct(retained.request).map_err(native)?;
    let sources: Vec<_> = edges.iter().map(ConstructedProfileEdge::source).collect();
    let census = retained.census(&sources)?;
    let partition = retained.partition(&sources)?;
    let bridge = retained.contacts(census, partition, &sources)?;
    Ok((edges, bridge))
}
fn native(cause: rustycad_tkernel::standard::Failure) -> ProfilePreparationCause {
    ProfilePreparationCause::Stage(OwnedPreparationCause::Native(cause))
}
fn stage(cause: OwnedPreparationCause) -> ProfilePreparationCause {
    ProfilePreparationCause::Stage(cause)
}
impl Retained<'_, '_> {
    fn enter(&mut self, phase: OwnedPrismPhase) -> Check<()> {
        self.phase = phase;
        if self.callbacks.poll(phase) {
            return Err(cause_callback());
        }
        Ok(())
    }
    fn census(&mut self, sources: &[ArrangementSource<'_>]) -> Check<BoundaryCensus> {
        self.enter(OwnedPrismPhase::SourceCapture)?;
        let inputs = capture_boundary_inputs(sources, self.request.declared_plane);
        let inputs = inputs.map_err(|cause| stage(cause.into()))?;
        self.enter(OwnedPrismPhase::Census)?;
        let policy = self.request.policy;
        let callbacks = &mut self.callbacks;
        let poll = || callbacks.poll(OwnedPrismPhase::Census);
        let census = enumerate_carrier_census(inputs, policy.census, policy.source, poll);
        census.map_err(|cause| stage(cause.into()))
    }
    fn partition(&mut self, sources: &[ArrangementSource<'_>]) -> Check<PartitionBoundOutputs> {
        self.enter(OwnedPrismPhase::Partition(ArrangementPhase::Admission))?;
        let originals = capture_originals(sources).map_err(native)?;
        let callbacks = &mut self.callbacks;
        let poll = &mut |p| callbacks.poll(OwnedPrismPhase::Partition(p));
        let observed = observe_partition_space(sources, poll, capture_space_representation);
        let copied = match &observed.completion {
            ArrangementCompletion::Arranged(owned) => {
                let poll = &mut |p| callbacks.poll(OwnedPrismPhase::Candidate(p));
                let plane = self.request.declared_plane;
                Some(observe_candidate_copies(
                    owned,
                    plane,
                    poll,
                    PartitionOccurrenceIdentity::capture,
                ))
            }
            _ => None,
        };
        let bound = bind_partition_outputs(observed, copied, originals, sources);
        require_candidates(&bound)?;
        Ok(bound)
    }
    fn contacts(
        &mut self,
        census: BoundaryCensus,
        partition: PartitionBoundOutputs,
        sources: &[ArrangementSource<'_>],
    ) -> Check<PairedContactTransaction> {
        self.enter(OwnedPrismPhase::Contact(
            ContactTransportPhase::SourceBinding,
        ))?;
        let policy = self.request.policy.contacts;
        let callbacks = &mut self.callbacks;
        let poll = |p| callbacks.poll(OwnedPrismPhase::Contact(p));
        let bridge = transport_partition_contacts(census, partition, policy, sources, poll);
        bridge.map_err(|cause| stage(OwnedPreparationCause::Contact(cause)))
    }
}
fn require_candidates(bound: &PartitionBoundOutputs) -> Check<()> {
    let completion = &bound.arrangement.completion;
    if !matches!(completion, ArrangementCompletion::Arranged(_)) {
        let class = arrangement_class(completion);
        return Err(stage(OwnedPreparationCause::StageUnavailable { class }));
    }
    let missing = OwnedPreparationCause::MissingLocalState {
        expected: "actual candidate copy observation",
    };
    let copied = bound.copied.as_ref().ok_or(stage(missing))?;
    copied.completion.as_ref().map(|_| ()).map_err(|cause| {
        let class = candidate_class(cause);
        stage(OwnedPreparationCause::StageUnavailable { class })
    })
}
fn failed(
    request: OwnedProfileRequest,
    cause: ProfilePreparationCause,
    callback: Option<OwnedPrismCallbackStop>,
    phase: OwnedPrismPhase,
) -> Box<ProfilePreparationFailure> {
    let class = match (&callback, &cause) {
        (Some(stop), _) => callback_class(&stop.cause),
        (None, ProfilePreparationCause::Stage(cause)) => preparation_class(cause),
        (None, ProfilePreparationCause::Admission(_)) => OwnedPreparationClass::Failed,
    };
    Box::new(ProfilePreparationFailure {
        class,
        cause,
        callback,
        phase,
        request,
    })
}
