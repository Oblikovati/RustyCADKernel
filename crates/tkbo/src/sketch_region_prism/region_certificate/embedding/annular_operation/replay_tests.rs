// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::annular_binding::tests::with_actual;
use super::tests::{binding, options};
use super::*;

fn actual_evidence() -> AnnularEmbeddingEvidence {
    with_actual(|request, region, receipt| {
        let bound = binding(request, region, receipt);
        inspect_bound_annular_embedding(request, receipt, bound, options(), || false)
            .unwrap_or_else(|_| panic!("actual annular embedding"))
    })
}
fn encoded(evidence: &AnnularEmbeddingEvidence) -> Vec<u8> {
    replay::encode(evidence).unwrap_or_else(|_| panic!("bound annular replay"))
}
#[test]
fn owned_embedding_replay_survives_every_local_owner_and_has_fresh_read_equality() {
    let evidence = actual_evidence();
    assert_eq!(evidence.replay_bytes(), encoded(&evidence));
    assert!(evidence.replay_bytes().len() > evidence.binding().replay_bytes().len());
    let bytes = evidence.replay_bytes().to_vec();
    drop(evidence);
    assert_eq!(bytes, actual_evidence().replay_bytes());
}
#[test]
fn replay_qualifies_policies_domains_and_complete_pair_claims() {
    let mut evidence = actual_evidence();
    let baseline = encoded(&evidence);
    evidence.requested.regularity.total_bisections += 1;
    assert_ne!(baseline, encoded(&evidence));
    evidence.requested = options();
    evidence.regularity.total_bisections += 1;
    assert_ne!(baseline, encoded(&evidence));
    evidence.regularity.total_bisections -= 1;
    evidence.domains.first.identity.output.child_slots.push(77);
    assert_ne!(baseline, encoded(&evidence));
    evidence.domains.first.identity.output.child_slots.pop();
    evidence.incidences[0].conclusion = AnnularIncidenceConclusion::DisjointCaps;
    assert_ne!(baseline, encoded(&evidence));
}
#[test]
fn foreign_trim_parents_and_incidence_operands_cannot_be_encoded_as_bound() {
    let mut evidence = actual_evidence();
    evidence.domains.first.trims[0].operand.parents.clear();
    assert!(replay::encode(&evidence).is_err());
    let mut evidence = actual_evidence();
    let AnnularIncidenceConclusion::TrimIncidence { first, .. } =
        &mut evidence.incidences[2].conclusion
    else {
        panic!("side seam");
    };
    first.operand.interval[1] += 1.;
    assert!(replay::encode(&evidence).is_err());
}
#[test]
fn independent_thread_operations_and_immutable_shared_reads_match_embedding_bytes() {
    let expected = actual_evidence().replay_bytes().to_vec();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(|| actual_evidence().replay_bytes().to_vec()))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    let shared = std::sync::Arc::new(actual_evidence());
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let shared = shared.clone();
            std::thread::spawn(move || encoded(&shared))
        })
        .collect();
    for reader in readers {
        assert_eq!(reader.join().unwrap(), expected);
    }
}
#[test]
fn actual_annular_embedding_process_receipt() {
    let evidence = actual_evidence();
    println!(
        "annular-prism-embedding-v2={}",
        evidence
            .replay_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}
