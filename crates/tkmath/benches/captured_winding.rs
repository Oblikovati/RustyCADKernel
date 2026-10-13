// SPDX-License-Identifier: GPL-2.0-only
//! Default execution verifies accepted operations; only --measure requests timings.
#[path = "captured_winding/allocation_calls.rs"]
mod allocation_calls;
#[path = "captured_winding/cases.rs"]
mod cases;
#[path = "captured_winding/execute.rs"]
mod execute;
use cases::WindingReuseWorkload;
use execute::WindingWorkLedger;
use std::time::Instant;
#[global_allocator]
static ALLOCATOR: allocation_calls::WindingBenchmarkAllocator =
    allocation_calls::WindingBenchmarkAllocator;
fn main() {
    let mode = std::env::args().find_map(|arg| arg.strip_prefix("--mode=").map(str::to_owned));
    assert!(
        mode.as_deref()
            .is_none_or(|value| matches!(value, "baseline" | "captured")),
        "mode {mode:?}; expected baseline or captured"
    );
    let workloads = cases::accepted_workloads();
    for workload in &workloads {
        verify_pair(workload);
    }
    if !std::env::args().any(|argument| argument == "--measure") {
        println!("{{\"verified_workloads\":3,\"timings_requested\":false}}");
        return;
    }
    for workload in &workloads {
        measure_selected(workload, mode.as_deref());
    }
}
fn verify_pair(workload: &WindingReuseWorkload) {
    let baseline = execute::run_baseline(workload);
    let captured = execute::run_captured(workload);
    assert_eq!(
        baseline.historical_bisections,
        captured.historical_bisections
    );
    assert_eq!(captured.reused, workload.segments.len());
    assert_eq!(baseline.new_bisections, 2 * captured.new_bisections);
}
fn operation(workload: &WindingReuseWorkload, mode: &str) -> WindingWorkLedger {
    match mode {
        "baseline" => execute::run_baseline(workload),
        "captured" => execute::run_captured(workload),
        _ => panic!("mode {mode}; expected baseline or captured"),
    }
}
fn measure(workload: &WindingReuseWorkload, mode: &str) {
    for _ in 0..20 {
        operation(workload, mode);
    }
    let expected = operation(workload, mode);
    let mut times = Vec::with_capacity(100);
    let mut allocations = Vec::with_capacity(100);
    for _ in 0..100 {
        let (elapsed, calls, actual) = measured_operation(workload, mode);
        assert_eq!(actual, expected);
        times.push(elapsed);
        allocations.push(calls);
    }
    print_measurement(workload, mode, expected, &times, &allocations);
}

fn measured_operation(
    workload: &WindingReuseWorkload,
    mode: &str,
) -> (u128, u64, WindingWorkLedger) {
    let allocations = allocation_calls::current_calls();
    let start = Instant::now();
    let result = operation(workload, mode);
    let elapsed = start.elapsed().as_nanos();
    (
        elapsed,
        allocation_calls::current_calls() - allocations,
        result,
    )
}

fn measure_selected(workload: &WindingReuseWorkload, mode: Option<&str>) {
    for selected in ["baseline", "captured"] {
        if mode.is_none_or(|value| value == selected) {
            measure(workload, selected);
        }
    }
}

fn print_measurement(
    workload: &WindingReuseWorkload,
    mode: &str,
    expected: WindingWorkLedger,
    times: &[u128],
    allocations: &[u64],
) {
    println!(
        "{{\"workload\":\"{}\",\"mode\":\"{mode}\",\"warmup\":20,\"samples\":100,\"baseline_accepted\":false,\"requested_policy\":{{\"version\":1,\"max_width\":0.001,\"max_degree\":32,\"max_bisections\":4096}},\"effective_capture_budget_count\":{},\"effective_capture_budgets\":{:?},\"new_bisections\":{},\"reused_certificates\":{},\"historical_bisections\":{},\"samples_ns\":{times:?},\"allocation_calls\":{allocations:?}}}",
        workload.name,
        expected.capture_budget_count,
        expected.effective_budgets,
        expected.new_bisections,
        expected.reused,
        expected.historical_bisections
    );
}
