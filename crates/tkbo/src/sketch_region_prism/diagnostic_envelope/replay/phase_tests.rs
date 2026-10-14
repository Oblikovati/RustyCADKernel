// SPDX-License-Identifier: GPL-2.0-only
//! Every currently supported outer and child phase has an independently declared format tag.
use super::{super::*, phases};
use crate::sketch_region::ArrangementPhase;
use crate::sketch_region_candidates::CandidatePhase;
use crate::sketch_region_contact_transport::ContactTransportPhase;
use crate::sketch_region_prism::{PrismPhase, publication::PrismPublicationPhase};
#[test]
fn preparation_boundaries_and_partition_phases_retain_exact_tags() {
    use OwnedPrismPhase::*;
    check(&[
        (Admission, &[0]),
        (Construction, &[1]),
        (SourceCapture, &[2]),
        (Census, &[3]),
    ]);
    check(&[
        (Partition(ArrangementPhase::Admission), &[4, 0]),
        (Partition(ArrangementPhase::Copy), &[4, 1]),
        (Partition(ArrangementPhase::Interference), &[4, 2]),
        (Partition(ArrangementPhase::History), &[4, 3]),
    ]);
}
#[test]
fn candidate_and_contact_phases_retain_exact_tags() {
    use OwnedPrismPhase::*;
    check(&[
        (Candidate(CandidatePhase::Copy), &[5, 0]),
        (Candidate(CandidatePhase::Generation), &[5, 1]),
        (Candidate(CandidatePhase::Capture), &[5, 2]),
    ]);
    check(&[
        (Contact(ContactTransportPhase::SourceBinding), &[6, 0]),
        (Contact(ContactTransportPhase::CanonicalContacts), &[6, 1]),
        (Contact(ContactTransportPhase::TopologyBinding), &[6, 2]),
        (
            Contact(ContactTransportPhase::ParameterConstruction),
            &[6, 3],
        ),
        (Contact(ContactTransportPhase::WorldConstruction), &[6, 4]),
        (Contact(ContactTransportPhase::GeneratedBinding), &[6, 5]),
    ]);
}
#[test]
fn prism_child_phases_retain_exact_tags() {
    use OwnedPrismPhase::Prism;
    check(&[
        (Prism(PrismPhase::Admission), &[7, 0]),
        (Prism(PrismPhase::Selection), &[7, 1]),
        (Prism(PrismPhase::Copy), &[7, 2]),
        (Prism(PrismPhase::Sweep), &[7, 3]),
        (Prism(PrismPhase::History), &[7, 4]),
        (Prism(PrismPhase::Validation), &[7, 5]),
    ]);
}
#[test]
fn publication_phase_vocabulary_has_explicit_tags_without_an_attribution_constructor() {
    use PrismPublicationPhase::*;
    for (phase, tag) in [
        (Admission, 0),
        (Join, 1),
        (Validation, 2),
        (History, 3),
        (Measures, 4),
        (Commit, 5),
    ] {
        let mut bytes = Vec::new();
        phases::append(PrismDiagnosticPhase::Publication(phase), &mut bytes);
        assert_eq!(bytes, [1, tag]);
    }
}
fn check(cases: &[(OwnedPrismPhase, &[u8])]) {
    for (phase, expected) in cases {
        let mut bytes = Vec::new();
        phases::append(
            PrismDiagnosticPhase::PreparationBoundary(*phase),
            &mut bytes,
        );
        let mut complete = vec![0];
        complete.extend(*expected);
        assert_eq!(bytes, complete);
    }
}
