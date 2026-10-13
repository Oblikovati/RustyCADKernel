// SPDX-License-Identifier: GPL-2.0-only
//! Face receipts are scalar; opt-in diagnostics retain actual alert payloads.
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::SurfaceType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelfCheckPhase {
    Initialization,
    FacePolicy,
    FaceSelf,
    SolidPairs,
    PostTreat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaceTermination {
    PolicyExcluded,
    NotAttempted,
    NotDone,
    Terminated { has_contacts: bool },
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct FaceCompletion {
    pub(crate) source_index: i32,
    pub(crate) kind: SurfaceType,
    pub(crate) termination: FaceTermination,
}

pub(crate) struct SelfCheckCompletion {
    pub(crate) level: i32,
    pub(crate) phase: SelfCheckPhase,
    pub(crate) active_source: Option<i32>,
    pub(crate) faces: Vec<FaceCompletion>,
    pub(crate) face_phase_scheduled: bool,
    pub(crate) returned: bool,
    pub(crate) rejected_arguments: Option<i32>,
    pub(crate) native_alerts: bool,
    pub(crate) alerts: Vec<NativeAlertObservation>,
    pub(crate) failure: Option<Failure>,
}

impl SelfCheckCompletion {
    pub(super) fn new(level: i32) -> Self {
        Self {
            level,
            phase: SelfCheckPhase::Initialization,
            active_source: None,
            faces: Vec::new(),
            face_phase_scheduled: false,
            returned: false,
            rejected_arguments: None,
            native_alerts: false,
            alerts: Vec::new(),
            failure: None,
        }
    }
}

pub(super) fn enter_phase(trace: Option<&mut SelfCheckCompletion>, phase: SelfCheckPhase) {
    if let Some(trace) = trace {
        trace.phase = phase;
        trace.active_source = None;
    }
}

pub(super) fn enter_face(trace: Option<&mut SelfCheckCompletion>, index: i32) {
    if let Some(trace) = trace {
        trace.active_source = Some(index);
    }
}

pub(super) fn record_policy(
    trace: Option<&mut SelfCheckCompletion>,
    index: i32,
    kind: SurfaceType,
    candidate: bool,
) {
    if let Some(trace) = trace {
        let termination = if candidate {
            FaceTermination::NotAttempted
        } else {
            FaceTermination::PolicyExcluded
        };
        trace.faces.push(FaceCompletion {
            source_index: index,
            kind,
            termination,
        });
        trace.active_source = None;
    }
}

pub(super) fn record_termination(
    trace: Option<&mut SelfCheckCompletion>,
    index: i32,
    done: bool,
    contacts: bool,
) {
    if let Some(trace) = trace {
        trace.finish_face(index, termination(done, contacts));
        trace.active_source = None;
    }
}

pub(super) fn capture_failure(trace: Option<&mut SelfCheckCompletion>, failure: Failure) {
    if let Some(trace) = trace {
        trace.failure = Some(failure);
    }
    // Without an opt-in receipt the original error is dropped before AddError.
}

impl SelfCheckCompletion {
    fn finish_face(&mut self, index: i32, termination: FaceTermination) {
        if let Some(face) = self
            .faces
            .iter_mut()
            .find(|face| face.source_index == index)
        {
            face.termination = termination;
        }
    }
}

fn termination(done: bool, contacts: bool) -> FaceTermination {
    if !done {
        return FaceTermination::NotDone;
    }
    FaceTermination::Terminated {
        has_contacts: contacts,
    }
}

pub(super) fn schedule_face_phase(trace: Option<&mut SelfCheckCompletion>) {
    if let Some(trace) = trace {
        trace.face_phase_scheduled = true;
    }
}

pub(crate) struct NativeAlertObservation {
    pub(crate) original: AlertHandle,
}
impl std::fmt::Debug for NativeAlertObservation {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.original.dynamic_type().name();
        output
            .debug_tuple("NativeAlertObservation")
            .field(&kind)
            .finish()
    }
}
