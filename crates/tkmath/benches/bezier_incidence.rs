// SPDX-License-Identifier: GPL-2.0-only
//! All-target execution verifies accepted inputs; only --measure requests timing.
#[path = "bezier_incidence/cases.rs"]
mod bezier_incidence_cases;
use bezier_incidence_cases::{IncidenceWorkload, accepted_workloads};
use std::time::Instant;

fn main() {
    let workloads = selected_workloads();
    for workload in &workloads {
        workload.verify();
    }
    if !std::env::args().any(|argument| argument == "--measure") {
        println!(
            "{{\"verified_workloads\":{},\"timings_requested\":false}}",
            workloads.len()
        );
        return;
    }
    for workload in &workloads {
        measure_workload(workload);
    }
}
fn selected_workloads() -> Vec<IncidenceWorkload> {
    let selector = std::env::args()
        .find_map(|argument| argument.strip_prefix("--workload=").map(str::to_owned));
    let workloads: Vec<IncidenceWorkload> = accepted_workloads()
        .into_iter()
        .filter(|workload| selector.as_ref().is_none_or(|name| name == &workload.name))
        .collect();
    assert!(
        !workloads.is_empty(),
        "workload {selector:?}; expected accepted corpus name"
    );
    workloads
}
fn measure_workload(workload: &IncidenceWorkload) {
    for _ in 0..20 {
        workload.verify();
    }
    let samples: Vec<u128> = (0..100).map(|_| measured_request(workload)).collect();
    println!(
        "{{\"workload\":\"{}\",\"warmup\":20,\"samples\":100,\"accepted_contacts\":{},\"parameter_width\":0.001,\"world_error_limit\":1e-8,\"baseline_accepted\":false,\"samples_ns\":{samples:?}}}",
        workload.name, workload.expected
    );
}
fn measured_request(workload: &IncidenceWorkload) -> u128 {
    let start = Instant::now();
    workload.verify();
    start.elapsed().as_nanos()
}
