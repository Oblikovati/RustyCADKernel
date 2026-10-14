// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn portable_receipt() -> String {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = embedded(&bridge, &shapes, &receipt);
    let source = report.regularity().source();
    let poles: Vec<_> = source
        .poles()
        .iter()
        .map(|pole| pole.map(f64::to_bits))
        .collect();
    let rows: Vec<_> = report.incidences().iter().map(row_receipt).collect();
    format!(
        "{{\"operation\":\"martensite.prism-embedding\",\"behavior\":1,\"poles\":{poles:?},\"interval\":{:?},\"jacobian\":{:?},\"regularity\":{:?},\"rows\":{rows:?},\"identity_scope\":\"operation-local-not-serialized\",\"body\":\"incomplete\"}}",
        source.interval().map(f64::to_bits),
        format!("{:?}", report.binding().signed_jacobian),
        format!("{:?}", report.regularity())
    )
}
fn row_receipt(row: &PrismFaceIncidence) -> String {
    let kind = incidence_tag(&row.conclusion);
    let trims: Vec<_> = row
        .first
        .trims
        .iter()
        .chain(&row.second.trims)
        .map(trim_receipt)
        .collect();
    format!(
        "{:?}/{:?}:{kind}:{trims:?}",
        row.first.role, row.second.role
    )
}
fn trim_receipt(trim: &PrismAuthoredTrimUse) -> String {
    format!(
        "{:?}:{:?}:{}",
        trim.role,
        trim.interval.map(f64::to_bits),
        trim.parents.len()
    )
}
fn incidence_tag(conclusion: &PrismIncidenceConclusion) -> String {
    match conclusion {
        PrismIncidenceConclusion::InjectivePlanarDomain => "planar".to_owned(),
        PrismIncidenceConclusion::Disjoint => "disjoint".to_owned(),
        PrismIncidenceConclusion::InjectiveModuloSeam {
            native_endpoints,
            normalized_sweep,
            ..
        } => format!("seam:{native_endpoints:?}:{normalized_sweep:?}"),
        PrismIncidenceConclusion::ExactlyRim {
            native,
            normalized_sweep,
            ..
        } => format!("rim:{native:?}:{normalized_sweep:?}"),
    }
}
#[test]
fn prism_embedding_process_receipt() {
    println!("{}", portable_receipt());
}
#[test]
fn independent_embedding_threads_match_serial_receipt() {
    let expected = portable_receipt();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..4).map(|_| portable_receipt()).collect::<Vec<_>>()))
        .collect();
    for worker in workers {
        for row in worker.join().unwrap() {
            assert_eq!(row, expected);
        }
    }
}
#[test]
fn two_embedding_processes_replay_identical_owned_math_bytes() {
    let expected = portable_receipt();
    assert_eq!(child_receipt(), expected);
    assert_eq!(child_receipt(), expected);
}
fn child_receipt() -> String {
    let output=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","sketch_region_prism::region_certificate::embedding::tests::replay_tests::prism_embedding_process_receipt","--nocapture"])
        .output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("{\"operation\":\"martensite.prism-embedding\""))
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0].to_owned()
}
#[test]
fn every_embedding_callback_cancellation_refuses_and_retry_stays_identical() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut count = 0;
    let before = attempt(&bridge, &shapes, &receipt, options(), || {
        count += 1;
        false
    })
    .ok()
    .unwrap();
    for cutoff in 1..=count {
        let mut polls = 0;
        assert!(
            attempt(&bridge, &shapes, &receipt, options(), || {
                polls += 1;
                polls == cutoff
            })
            .is_err(),
            "poll {cutoff}; expected refusal"
        );
    }
    let after = embedded(&bridge, &shapes, &receipt);
    assert_eq!(before.incidences(), after.incidences());
    assert_eq!(before.regularity(), after.regularity());
}
