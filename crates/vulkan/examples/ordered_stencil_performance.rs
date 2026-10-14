//! Accepted original87 retained workloads; `ordered_stencil_performance` records steady redraw/readback.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ordered_stencil_replay.rs"]
mod replay;

fn main() {
    let mut options = device_options::validation_options();
    options.validation = false;
    for case in replay::native_cases() {
        let samples = replay::replay_case(&case, options, true);
        println!(
            "MARTENSITE_PERFORMANCE={{\"mode\":\"{}\",\"method\":\"{}\",\"seed\":{},\"warmup\":20,\"validation\":false,\"baseline_accepted\":false,\"samples_ms\":{samples:?}}}",
            case.mode, case.method, case.seed
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn performance_inputs_verify_every_accepted_workload_without_timing() {
        for case in super::replay::native_cases() {
            assert!(
                super::replay::replay_case(
                    &case,
                    super::device_options::validation_options(),
                    false,
                )
                .is_empty()
            );
        }
    }
}
