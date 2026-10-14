// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn scalar_receipt(report: &PrismMeasureInspection) -> String {
    format!(
        "{{\"operation\":\"martensite.prism-measure-subset\",\"behavior\":1,\"first_cap_bits\":{:?},\"last_cap_bits\":{:?},\"sweep_bits\":{:?},\"winding\":{},\"reversed\":{},\"charts\":{},\"pending\":\"side,total\",\"body\":\"incomplete\"}}",
        report.measures.first_cap.enclosure().map(f64::to_bits),
        report.measures.last_cap.enclosure().map(f64::to_bits),
        report.measures.signed_volume.enclosure().map(f64::to_bits),
        report.region.selected_winding,
        report.region.boundary_reversed,
        report.charts.len()
    )
}
fn portable_receipt() -> String {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    scalar_receipt(&measured(&bridge, &shapes, &receipt))
}
#[test]
fn portable_prism_measure_process_receipt() {
    let receipt = portable_receipt();
    assert!(receipt.contains("\"body\":\"incomplete\""));
    println!("{receipt}");
}
#[test]
fn independent_measurement_workers_match_serial_owned_results() {
    let baseline = portable_receipt();
    let workers: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..4).map(|_| portable_receipt()).collect::<Vec<_>>()))
        .collect();
    for worker in workers {
        for actual in worker.join().unwrap() {
            assert_eq!(actual, baseline);
        }
    }
}
#[test]
fn every_observed_measurement_cancellation_poll_is_incomplete_and_retryable() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut polls = 0;
    let baseline = measurement_attempt(&bridge, &shapes, &receipt, options(), || {
        polls += 1;
        false
    })
    .ok()
    .unwrap();
    assert_each_cancel_poll(&bridge, &shapes, &receipt, polls);
    assert_eq!(
        scalar_receipt(&measured(&bridge, &shapes, &receipt)),
        scalar_receipt(&baseline)
    );
}

fn assert_each_cancel_poll(
    bridge: &PairedContactTransaction,
    shapes: &[rustycad_tkbrep::topods::Shape],
    receipt: &PrismReceipt,
    polls: usize,
) {
    for cutoff in 1..=polls {
        let mut poll = 0;
        let result = measurement_attempt(bridge, shapes, receipt, options(), || {
            poll += 1;
            poll == cutoff
        });
        assert!(
            result.is_err(),
            "poll {cutoff}; expected incomplete cancellation"
        );
    }
}
