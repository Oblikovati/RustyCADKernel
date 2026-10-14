// SPDX-License-Identifier: GPL-2.0-only
//! Default execution verifies every accepted workload; only --measure emits timings.
#[path = "certified_bezier_pairs/cases.rs"]
mod cases;
use cases::PairWorkload;
use std::time::Instant;
fn main() {
    let workloads = cases::accepted_cases();
    for workload in &workloads {
        cases::accepted_operation(workload);
    }
    let selected = selected_case(&workloads);
    if !std::env::args().any(|arg| arg == "--measure") {
        println!("{{\"verified_workloads\":6,\"timings_requested\":false}}");
        return;
    }
    for workload in workloads
        .iter()
        .filter(|case| selected.as_deref().is_none_or(|label| label == case.label))
    {
        measure(workload);
    }
}
fn selected_case(workloads: &[PairWorkload]) -> Option<String> {
    let selected = std::env::args().find_map(|arg| arg.strip_prefix("--case=").map(str::to_owned));
    assert!(
        selected
            .as_deref()
            .is_none_or(|label| workloads.iter().any(|case| case.label == label)),
        "case {selected:?}; expected an accepted workload label"
    );
    selected
}
fn measure(workload: &PairWorkload) {
    for _ in 0..20 {
        cases::accepted_operation(workload);
    }
    let expected_work = cases::accepted_operation(workload);
    let mut samples = Vec::with_capacity(100);
    for _ in 0..100 {
        let start = Instant::now();
        let work = cases::accepted_operation(workload);
        samples.push(start.elapsed().as_nanos());
        assert_eq!(work, expected_work);
    }
    emit_measurement(workload, expected_work, &samples);
}
fn emit_measurement(workload: &PairWorkload, completed: u16, samples: &[u128]) {
    println!(
        "{{\"case\":\"{}\",\"operation\":\"capture+certify+accept+drop\",\"warmup\":20,\"samples\":100,\"completed_splits\":{},\"contacts\":{},\"requested_world_error\":{},\"baseline_accepted\":false,\"samples_ns\":{:?}}}",
        workload.label, completed, workload.expected_contacts, workload.world_error, samples
    );
}
