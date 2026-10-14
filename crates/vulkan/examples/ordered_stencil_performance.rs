//! Accepted original87 retained workloads; `ordered_stencil_performance` records steady redraw/readback.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ordered_stencil_replay.rs"]
mod replay;

fn main() {
    let measure = stencil_performance_mode(std::env::args().skip(1).collect());
    let mut options = device_options::validation_options();
    options.validation = false;
    for case in replay::native_cases() {
        let samples = replay::replay_case(&case, options, measure);
        if !measure {
            continue;
        }
        println!(
            "MARTENSITE_PERFORMANCE={{\"mode\":\"{}\",\"method\":\"{}\",\"seed\":{},\"warmup\":20,\"validation\":false,\"baseline_accepted\":false,\"samples_ms\":{samples:?}}}",
            case.mode, case.method, case.seed
        );
    }
}

fn stencil_performance_mode(arguments: Vec<String>) -> bool {
    match arguments.as_slice() {
        [] => true,
        [flag] if flag == "--verify-only" => false,
        _ => panic!("received arguments={arguments:?}; expected no arguments or --verify-only"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn explicit_verification_never_requests_measurement() {
        assert!(super::stencil_performance_mode(vec![]));
        assert!(!super::stencil_performance_mode(vec![
            "--verify-only".into()
        ]));
        assert!(
            std::panic::catch_unwind(|| super::stencil_performance_mode(vec!["bad".into()]))
                .is_err()
        );
    }
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
