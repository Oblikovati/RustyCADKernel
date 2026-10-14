// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::ResidualError;
use super::*;

fn sweep(
    cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialMeasure, PolynomialIntegralError> {
    certify_polynomial_sweep_integral(
        &occurrences("cubic_ccw"),
        frame(),
        placement(),
        [0., 0., 2.],
        PolynomialIntegralOptions::default(),
        cancelled,
    )
}
#[test]
fn cancellation_at_every_polled_step_leaves_no_state_for_the_next_request() {
    let mut polls = 0;
    let expected = sweep(|| {
        polls += 1;
        false
    })
    .unwrap();
    for stop in 1..=polls {
        let error = cancel_at(stop);
        assert!(matches!(
            error.cause,
            PolynomialIntegralFailure::Metric(cause) if matches!(*cause, ResidualError::Roots(RootIsolationError::Cancelled))
        ));
    }
    assert_eq!(sweep(|| false).unwrap(), expected);
}
fn cancel_at(stop: usize) -> PolynomialIntegralError {
    let mut current = 0;
    sweep(|| {
        current += 1;
        current == stop
    })
    .unwrap_err()
}
#[test]
fn singular_sweep_and_zero_curves_report_zero_scalar_without_body_claims() {
    let report = certify_polynomial_sweep_integral(
        &occurrences("cubic_ccw"),
        frame(),
        placement(),
        [1., 0., 0.],
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap();
    assert_eq!(report.enclosure(), [0., 0.]);
    assert_eq!(report.absolute_error_bound(), 0.);
    let poles = [[2., 3.]; 17];
    assert_eq!(measure(&[polynomial(&poles)]).enclosure(), [0., 0.]);
}
#[test]
fn tiny_exact_integral_keeps_underflow_error_and_large_integral_is_incomplete() {
    let tiny = f64::from_bits(1);
    let curves = [line([tiny, 0.], [0., tiny])];
    let report = measure(&curves);
    assert_eq!(report.enclosure(), [0., tiny]);
    assert_eq!(report.absolute_error_bound(), tiny);
    let huge = [line([f64::MAX, 0.], [0., f64::MAX])];
    let error =
        certify_polynomial_boundary_integral(&huge, PolynomialIntegralOptions::default(), || false)
            .unwrap_err();
    assert!(matches!(
        error.cause,
        PolynomialIntegralFailure::Unrepresentable { .. }
    ));
}
#[test]
fn sweep_scales_exact_integral_before_any_binary64_rounding() {
    let huge = [line([f64::MAX, 0.], [0., f64::MAX])];
    let options = PolynomialIntegralOptions {
        absolute_error_limit: f64::MAX,
        ..Default::default()
    };
    let report = certify_polynomial_sweep_integral(
        &huge,
        frame(),
        placement(),
        [0., 0., f64::from_bits(1)],
        options,
        || false,
    )
    .unwrap();
    let maximum = ExactRatio::from_finite(f64::MAX);
    let factor = ExactRatio::from_finite(f64::from_bits(1)).divide(&ExactRatio::from_finite(2.));
    let expected = maximum.multiply(&maximum).multiply(&factor);
    assert_exact_enclosure(&report, &expected);
}
#[test]
fn generated_orientation_reversals_and_cancellation_pairs_are_exact() {
    for count in 1..=17 {
        let poles: Vec<[f64; 2]> = (0..count).map(|i| [i as f64, (i * i) as f64]).collect();
        let forward = polynomial(&poles);
        let reverse = OrientedMetricCurve {
            reversed: true,
            ..forward
        };
        let actual = measure(&[forward, reverse]);
        assert_eq!(actual.enclosure(), [0., 0.]);
        assert_eq!(actual.absolute_error_bound(), 0.);
    }
}
#[test]
fn concurrent_operations_and_shared_report_reads_match_serial_results() {
    let serial = sweep(|| false).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let serial = &serial;
            scope.spawn(move || {
                for _ in 0..8 {
                    assert_eq!(&sweep(|| false).unwrap(), serial);
                }
            });
        }
    });
}
#[test]
fn integral_process_probe() {
    let Ok(mode) = std::env::var("MARTENSITE_INTEGRAL_PROBE") else {
        return;
    };
    let reports = if mode == "parallel" {
        let handles = [0, 1].map(|_| std::thread::spawn(|| sweep(|| false).unwrap()));
        handles.map(|thread| thread.join().unwrap())
    } else {
        [sweep(|| false).unwrap(), sweep(|| false).unwrap()]
    };
    for report in reports {
        println!(
            "INTEGRAL_BITS {:?} {}",
            report.enclosure().map(f64::to_bits),
            report.absolute_error_bound().to_bits()
        );
    }
}
#[test]
fn two_processes_and_parallel_process_have_byte_identical_measurements() {
    let first = process_output("serial");
    assert_eq!(first, process_output("serial"));
    assert_eq!(first, process_output("parallel"));
}
fn process_output(mode: &str) -> Vec<String> {
    let probe = "predicates::roots::metric_bounds::polynomial_integrals::tests::stress_tests::integral_process_probe";
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", probe, "--nocapture", "--test-threads=1"])
        .env("MARTENSITE_INTEGRAL_PROBE", mode)
        .output()
        .unwrap();
    assert!(output.status.success(), "process failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            line.split_once("INTEGRAL_BITS ")
                .map(|(_, row)| row.to_owned())
        })
        .collect();
    assert_eq!(rows.len(), 2, "probe stdout: {stdout}");
    rows
}

#[test]
fn degree_ladder_matches_independent_area_between_diagonal_and_power_graph() {
    for degree in [1, 2, 4, 8, 16] {
        let mut poles: Vec<[f64; 2]> = (0..=degree)
            .map(|index| [index as f64 / degree as f64, 0.])
            .collect();
        poles[degree][1] = 1.;
        let curves = [polynomial(&poles), line([1., 1.], [-1., -1.])];
        // Dyadic controls give x=t and y=t^degree; area is integral_0^1(t-t^degree)dt.
        let expected = ExactRatio::new((degree - 1).into(), (2 * (degree + 1)).into());
        assert_boundary_fixture(&curves, expected);
    }
}
