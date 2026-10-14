// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn portable_receipt() -> String {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = oriented(&bridge, &shapes, &receipt);
    encoded(&evidence)
}
pub(super) fn encoded(evidence: &PrismOrientationEvidence) -> String {
    let rows: Vec<_> = evidence.faces().iter().map(face_row).collect();
    let source = evidence.embedding().regularity().source();
    let poles: Vec<_> = source.poles().iter().map(|p| p.map(f64::to_bits)).collect();
    let (f, r, w, q) = evidence.winding().factors();
    format!(
        "{{\"operation\":\"martensite.prism-orientation\",\"version\":1,\"provenance\":\"DerivedFromFreshFacedWinding\",\"factors\":{:?},\"jacobian_sign\":{},\"poles\":{poles:?},\"rows\":{rows:?},\"identity_scope\":\"operation-local-not-serialized\",\"body\":\"incomplete\"}}",
        [i64::from(f), i64::from(r), w, i64::from(q)],
        evidence.jacobian_sign()
    )
}
fn face_row(row: &PrismFaceOrientation) -> String {
    let mut trims: Vec<_> = row.trims().iter().map(trim_row).collect();
    trims.sort();
    format!(
        "{:?}:{:?}:{}:{trims:?}",
        row.domain().role,
        row.orientations(),
        row.domain().parents.len()
    )
}
fn trim_row(trim: &PrismTrimOrientation) -> String {
    format!(
        "{:?}:{:?}:{:?}:{}",
        trim.authored().role,
        trim.orientations(),
        trim.authored().interval.map(f64::to_bits),
        trim.authored().parents.len()
    )
}
#[test]
fn prism_orientation_process_receipt() {
    let receipt = portable_receipt();
    assert_eq!(receipt, include_str!("orientation-v1.json").trim());
    println!("{receipt}");
}
#[test]
fn independent_orientation_threads_match_serial_owned_receipt() {
    let expected = portable_receipt();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..4).map(|_| portable_receipt()).collect::<Vec<_>>()))
        .collect();
    for worker in workers {
        for receipt in worker.join().unwrap() {
            assert_eq!(receipt, expected);
        }
    }
}
#[test]
fn two_orientation_processes_have_identical_portable_receipts() {
    let expected = portable_receipt();
    assert_eq!(child_receipt(), expected);
    assert_eq!(child_receipt(), expected);
}
fn child_receipt() -> String {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","sketch_region_prism::region_certificate::orientation::tests::replay_tests::prism_orientation_process_receipt","--nocapture"])
        .output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<_> = text
        .lines()
        .filter(|line| line.starts_with("{\"operation\":\"martensite.prism-orientation\""))
        .collect();
    assert_eq!(rows.len(), 1);
    rows[0].to_owned()
}
