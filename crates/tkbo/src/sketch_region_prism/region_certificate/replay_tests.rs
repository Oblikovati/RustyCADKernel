// SPDX-License-Identifier: GPL-2.0-only
//! Replay serializes scalar evidence and semantic roles; process-local addresses are not persistent names.
use super::{tests::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;

fn portable_receipt() -> String {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = proof(&receipt);
    let poles = receipt_poles(evidence);
    let count = evidence.source_candidate.boundaries.len();
    let orientation = evidence.source_candidate.face.orientation;
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    format!(
        "{{\"operation\":\"martensite.region-certificate\",\"version\":1,\"winding\":{},\"reversed\":{},\"orientation\":\"{:?}\",\"boundary_count\":{},\"report\":{},\"root\":{},\"poles\":{:?},\"body\":\"incomplete\"}}",
        evidence.selected_winding,
        evidence.boundary_reversed,
        orientation,
        count,
        evidence.contact.report,
        evidence.contact.candidate_root,
        poles
    )
}
#[test]
fn portable_region_certificate_process_receipt() {
    let receipt = portable_receipt();
    assert!(receipt.contains("\"winding\":1"));
    assert!(receipt.contains("[[0.0, 0.0], [4.0, 0.0], [0.0, 4.0], [0.0, 0.0]]"));
    println!("{receipt}");
}
#[test]
fn independent_operation_workers_match_single_thread_receipt() {
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

fn receipt_poles(evidence: &RegionCertificate) -> &[[f64; 2]] {
    let CapturedMetricCurve::Bezier { poles, .. } = &evidence.source_candidate.boundaries[0].pcurve
    else {
        panic!("expected certified Bézier boundary");
    };
    poles
}
