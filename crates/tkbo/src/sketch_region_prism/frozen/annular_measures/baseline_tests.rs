// SPDX-License-Identifier: GPL-2.0-only
//! Pin initial V2 owned bytes before separating arithmetic orchestration helpers.
use super::*;
#[test]
fn annular_measure_process_receipt() {
    let report = measured(&joined());
    let hex: String = report
        .replay_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("annular-measures-v2={hex}");
    println!("ANNULAR_SIDE_SUBDIVISIONS={}", report.side_subdivisions());
}
