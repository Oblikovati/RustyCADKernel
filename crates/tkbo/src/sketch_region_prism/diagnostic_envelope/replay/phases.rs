// SPDX-License-Identifier: GPL-2.0-only
//! Explicit format tags do not depend on Rust enum layout or debug formatting.
use super::super::*;
use crate::sketch_region::ArrangementPhase;
use crate::sketch_region_candidates::CandidatePhase;
use crate::sketch_region_contact_transport::ContactTransportPhase;
use crate::sketch_region_prism::{PrismPhase, publication::PrismPublicationPhase};
pub(super) fn append(value: PrismDiagnosticPhase, bytes: &mut Vec<u8>) {
    match value {
        PrismDiagnosticPhase::PreparationBoundary(value) => {
            bytes.push(0);
            preparation(value, bytes);
        }
        PrismDiagnosticPhase::Publication(value) => {
            bytes.push(1);
            bytes.push(publication(value));
        }
    }
}
pub(in crate::sketch_region_prism) fn preparation(value: OwnedPrismPhase, bytes: &mut Vec<u8>) {
    match value {
        OwnedPrismPhase::Admission => bytes.push(0),
        OwnedPrismPhase::Construction => bytes.push(1),
        OwnedPrismPhase::SourceCapture => bytes.push(2),
        OwnedPrismPhase::Census => bytes.push(3),
        OwnedPrismPhase::Partition(value) => bytes.extend([4, arrangement(value)]),
        OwnedPrismPhase::Candidate(value) => bytes.extend([5, candidate(value)]),
        OwnedPrismPhase::Contact(value) => bytes.extend([6, contact(value)]),
        OwnedPrismPhase::Prism(value) => bytes.extend([7, prism(value)]),
    }
}
fn arrangement(value: ArrangementPhase) -> u8 {
    match value {
        ArrangementPhase::Admission => 0,
        ArrangementPhase::Copy => 1,
        ArrangementPhase::Interference => 2,
        ArrangementPhase::History => 3,
    }
}
fn candidate(value: CandidatePhase) -> u8 {
    match value {
        CandidatePhase::Copy => 0,
        CandidatePhase::Generation => 1,
        CandidatePhase::Capture => 2,
    }
}
fn contact(value: ContactTransportPhase) -> u8 {
    match value {
        ContactTransportPhase::SourceBinding => 0,
        ContactTransportPhase::CanonicalContacts => 1,
        ContactTransportPhase::TopologyBinding => 2,
        ContactTransportPhase::ParameterConstruction => 3,
        ContactTransportPhase::WorldConstruction => 4,
        ContactTransportPhase::GeneratedBinding => 5,
    }
}
fn prism(value: PrismPhase) -> u8 {
    match value {
        PrismPhase::Admission => 0,
        PrismPhase::Selection => 1,
        PrismPhase::Copy => 2,
        PrismPhase::Sweep => 3,
        PrismPhase::History => 4,
        PrismPhase::Validation => 5,
    }
}
pub(in crate::sketch_region_prism) fn publication(value: PrismPublicationPhase) -> u8 {
    match value {
        PrismPublicationPhase::Admission => 0,
        PrismPublicationPhase::Join => 1,
        PrismPublicationPhase::Validation => 2,
        PrismPublicationPhase::History => 3,
        PrismPublicationPhase::Measures => 4,
        PrismPublicationPhase::Commit => 5,
    }
}
