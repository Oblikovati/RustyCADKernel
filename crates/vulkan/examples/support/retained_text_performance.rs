//! Measure complete retained and explicit pixel text redraw/readback under one sampling contract.
use martensite_vulkan::Graphic3dRenderView;
use std::time::Instant;
/// Record 100 complete samples after 20 warmups; `measure_text_view(&render, "subtitle-basic")`.
pub fn measure_text_view(render: &Graphic3dRenderView, mode: &str) {
    // One sampling contract prevents retained and pixel metrics from drifting.
    measure_text_operation(mode, &mut || render.try_redraw().unwrap());
}
/// Share complete-operation timings across retained/pixel frontends; `measure_text_operation("probe", &mut operation)`.
pub fn measure_text_operation(mode: &str, operation: &mut dyn FnMut()) {
    let samples = text_operation_samples(operation);
    print_text_samples(mode, &samples);
}

fn text_operation_samples(operation: &mut dyn FnMut()) -> Vec<f64> {
    for _ in 0..20 {
        operation();
    }
    (0..100)
        .map(|_| {
            let start = Instant::now();
            operation();
            start.elapsed().as_secs_f64() * 1000.
        })
        .collect()
}
fn print_text_samples(mode: &str, samples: &[f64]) {
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{mode}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_contract_warms_twenty_and_records_one_hundred_complete_operations() {
        let mut calls = 0;
        let samples = text_operation_samples(&mut || {
            calls += 1;
        });
        assert_eq!(calls, 120);
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|v| v.is_finite() && *v >= 0.));
        print_text_samples("named-counting-operation", &samples);
        measure_text_operation("named-counting-operation", &mut || {
            calls += 1;
        });
        assert_eq!(calls, 240);
    }
}
