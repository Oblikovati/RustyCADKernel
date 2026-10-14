// SPDX-License-Identifier: GPL-2.0-only
//! Functional all-target execution; optimized measurement requires explicit --measure.
#[path = "../tests/support/metric_residual_cases.rs"]
mod metric_residual_cases;
use metric_residual_cases::{AcceptedMetricWorkload, accepted_metric_workloads};
use rustycad_tkmath::predicates::roots as metric_api;
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
        measure(workload);
    }
}
fn selected_workloads() -> Vec<AcceptedMetricWorkload> {
    let selected =
        std::env::args().find_map(|arg| arg.strip_prefix("--workload=").map(str::to_owned));
    let workloads: Vec<_> = accepted_metric_workloads()
        .into_iter()
        .filter(|w| selected.as_ref().is_none_or(|name| name == w.name))
        .collect();
    assert!(
        !workloads.is_empty(),
        "metric workload {selected:?}; expected an accepted corpus name"
    );
    workloads
}
fn measure(workload: &AcceptedMetricWorkload) {
    for _ in 0..20 {
        workload.verify();
    }
    let samples: Vec<u128> = (0..100).map(|_| sample(workload)).collect();
    println!(
        "{{\"workload\":\"{}\",\"warmup\":20,\"samples\":100,\"baseline_accepted\":false,\"samples_ns\":{samples:?}}}",
        workload.name
    );
}
fn sample(workload: &AcceptedMetricWorkload) -> u128 {
    let start = Instant::now();
    workload.verify();
    start.elapsed().as_nanos()
}
