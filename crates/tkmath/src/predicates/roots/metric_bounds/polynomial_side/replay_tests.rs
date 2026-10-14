// SPDX-License-Identifier: GPL-2.0-only
use super::tests::*;
use super::*;

fn reversing(
    cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialSideIntegral, PolynomialSideError> {
    let poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    let options = PolynomialSideOptions {
        absolute_error_limit: 0.1,
        ..Default::default()
    };
    certify_polynomial_side_integral(
        curve(&poles),
        identity(),
        identity(),
        [0., 0., 1.],
        options,
        cancelled,
    )
}
#[test]
fn every_observed_cancellation_checkpoint_preserves_reuse_and_completed_work() {
    let mut polls = 0;
    let expected = reversing(|| {
        polls += 1;
        false
    })
    .unwrap();
    let mut observed_partial_child = false;
    for stop in 1..=polls {
        let error = cancelled_at(stop);
        assert!(matches!(*error.cause, PolynomialSideFailure::Cancelled));
        assert!(error.work.subdivisions <= expected.work().subdivisions);
        observed_partial_child |= error.work.interval_bounds > 1 + 2 * error.work.subdivisions;
    }
    assert!(observed_partial_child);
    assert_eq!(reversing(|| false).unwrap(), expected);
}
fn cancelled_at(stop: usize) -> PolynomialSideError {
    let mut polls = 0;
    reversing(|| {
        polls += 1;
        polls == stop
    })
    .unwrap_err()
}
#[test]
fn caller_inputs_and_published_results_are_isolated_after_all_terminal_states() {
    let mut poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    let original = poles;
    let report = integrate(&poles, Default::default()).unwrap();
    let saved = report.clone();
    assert_eq!(poles, original);
    assert_input_refusal_and_cancellation(&poles);
    assert_eq!(poles, original);
    poles.fill([0.; 3]);
    assert_eq!(report, saved);
    assert_eq!(
        integrate(&poles, Default::default()).unwrap().enclosure(),
        [0.; 2]
    );
}
fn assert_input_refusal_and_cancellation(poles: &[[f64; 3]]) {
    let refusal = PolynomialSideOptions {
        max_degree: 0,
        ..Default::default()
    };
    assert!(integrate(poles, refusal).is_err());
    let result = certify_polynomial_side_integral(
        curve(poles),
        identity(),
        identity(),
        [0.; 3],
        Default::default(),
        || true,
    );
    assert!(result.is_err());
}

#[test]
fn concurrent_operations_and_immutable_reads_match_serial_bytes() {
    let expected = reversing(|| false).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let expected = &expected;
            scope.spawn(move || {
                for _ in 0..8 {
                    assert_eq!(&reversing(|| false).unwrap(), expected);
                }
            });
        }
    });
}
#[test]
fn side_integral_process_probe() {
    let Ok(mode) = std::env::var("MARTENSITE_SIDE_INTEGRAL_PROBE") else {
        return;
    };
    let reports = if mode == "parallel" {
        [0, 1]
            .map(|_| std::thread::spawn(|| reversing(|| false).unwrap()))
            .map(|thread| thread.join().unwrap())
    } else {
        [reversing(|| false).unwrap(), reversing(|| false).unwrap()]
    };
    for report in reports {
        print_bits(&report);
    }
}
fn print_bits(report: &CertifiedPolynomialSideIntegral) {
    println!(
        "{{\"kind\":\"side_integral_bits\",\"enclosure_bits\":{:?},\"error_bits\":{},\"subdivisions\":{},\"interval_bounds\":{}}}",
        report.enclosure().map(f64::to_bits),
        report.absolute_error_bound().to_bits(),
        report.work().subdivisions,
        report.work().interval_bounds
    );
}

#[test]
fn independent_processes_and_threaded_process_match_byte_for_byte() {
    let expected = process_output("serial");
    assert_eq!(expected, process_output("serial"));
    assert_eq!(expected, process_output("parallel"));
}
fn process_output(mode: &str) -> Vec<String> {
    let probe = "predicates::roots::metric_bounds::polynomial_side::replay_tests::side_integral_process_probe";
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", probe, "--nocapture", "--test-threads=1"])
        .env("MARTENSITE_SIDE_INTEGRAL_PROBE", mode)
        .output()
        .unwrap();
    assert!(output.status.success(), "process failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<_> = stdout
        .lines()
        .filter_map(|line| {
            line.find("{\"kind\":\"side_integral_bits\"")
                .map(|start| line[start..].to_owned())
        })
        .collect();
    assert_eq!(rows.len(), 2, "probe stdout: {stdout}");
    rows
}
