// SPDX-License-Identifier: GPL-2.0-only
//! Unhandled sketch spline construction and independent analytic oracles.
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::sketch_spline::*;
use rustycad_tkmath::gp::Point2;
mod handle_parity;
mod handles;
mod oracle;
fn source_points() -> Vec<Point2> {
    [[0.0, 0.0], [0.2, 1.0], [2.0, 1.5], [3.0, -0.5], [1.0, -2.0]]
        .into_iter()
        .map(|[x, y]| Point2::new(x, y))
        .collect()
}
fn options() -> SketchSplineOptionsV1 {
    SketchSplineOptionsV1 {
        minimum_pivot: 1e-12,
        maximum_fit_residual: 1e-10,
        maximum_seam_residuals: [1e-10, 1e-9, 1e-8],
    }
}
fn spacing(index: usize) -> SketchFitSpacing {
    [
        SketchFitSpacing::Centripetal,
        SketchFitSpacing::ChordLength,
        SketchFitSpacing::Uniform,
    ][index]
}
fn assert_close(actual: f64, expected: f64) {
    let tolerance = 1024.0 * f64::EPSILON * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual}, expected {expected} +/- {tolerance}"
    );
}
#[test]
fn all_go_fit_recipes_and_135_analytic_samples_are_reconstructed() {
    let reports: Vec<SketchSplineReport> = (0..6)
        .map(|case| {
            build_unhandled_spline(
                &source_points(),
                SketchSplineIntent::Fit(spacing(case % 3)),
                case >= 3,
                options(),
            )
            .unwrap()
        })
        .collect();
    let mut sample_count = 0;
    for row in include_str!("go-fit-oracle.csv").lines() {
        sample_count += oracle::assert_row(row, &reports);
    }
    assert_eq!(sample_count, 135);
}
#[test]
fn small_open_fits_lower_degree_and_pass_through_every_point() {
    for count in 2..=4 {
        for method in 0..3 {
            let points = &source_points()[..count];
            let report = build_unhandled_spline(
                points,
                SketchSplineIntent::Fit(spacing(method)),
                false,
                options(),
            )
            .unwrap();
            assert_eq!(report.definition.degree(), count as i32 - 1);
            assert_eq!(report.definition.poles().len(), count);
            assert_eq!(report.stations.len(), count);
            assert!(report.fit_residual.unwrap() <= options().maximum_fit_residual);
            assert!(report.seam_residuals.is_none());
        }
    }
}
#[test]
fn control_intent_retains_its_polygon_instead_of_fitting_it() {
    let points = source_points();
    for closed in [false, true] {
        let report =
            build_unhandled_spline(&points, SketchSplineIntent::Control, closed, options())
                .unwrap();
        assert_eq!(&report.definition.poles()[..points.len()], points);
        assert!(report.stations.is_empty() && report.fit_residual.is_none());
        assert_eq!(report.seam_residuals.is_some(), closed);
        assert_eq!(report.definition.curve().unwrap().first_parameter(), 0.0);
        assert_eq!(report.definition.curve().unwrap().last_parameter(), 1.0);
        if closed {
            assert_eq!(&report.definition.poles()[points.len()..], &points[..3]);
        }
    }
}
#[test]
fn invalid_counts_coordinates_options_and_collapsed_stations_fail() {
    for points in [
        vec![],
        vec![Point2::default()],
        vec![Point2::new(f64::NAN, 0.0); 3],
    ] {
        assert!(
            build_unhandled_spline(&points, SketchSplineIntent::Control, false, options()).is_err()
        );
    }
    let repeated = vec![Point2::default(); 4];
    for method in [SketchFitSpacing::Centripetal, SketchFitSpacing::ChordLength] {
        let failure =
            build_unhandled_spline(&repeated, SketchSplineIntent::Fit(method), false, options())
                .unwrap_err();
        assert!(failure.to_string().contains("station"));
    }
    let mut invalid = options();
    invalid.minimum_pivot = 0.0;
    assert!(
        build_unhandled_spline(
            &source_points(),
            SketchSplineIntent::Control,
            false,
            invalid
        )
        .is_err()
    );
    assert!(
        build_unhandled_spline(
            &source_points()[..2],
            SketchSplineIntent::Control,
            true,
            options()
        )
        .is_err()
    );
}
#[test]
fn independent_results_survive_failure_and_source_mutation() {
    let mut points = source_points();
    let report = build_unhandled_spline(
        &points,
        SketchSplineIntent::Fit(SketchFitSpacing::Uniform),
        true,
        options(),
    )
    .unwrap();
    let before = report.clone();
    points[0] = Point2::new(100.0, 200.0);
    assert!(
        build_unhandled_spline(&points[..1], SketchSplineIntent::Control, false, options())
            .is_err()
    );
    assert_eq!(report, before);
    assert_eq!(
        build_unhandled_spline(
            &source_points(),
            SketchSplineIntent::Fit(SketchFitSpacing::Uniform),
            true,
            options()
        )
        .unwrap(),
        before
    );
}
#[test]
fn fresh_native_curves_do_not_alias_each_other_or_the_accepted_recipe() {
    let report = build_unhandled_spline(
        &source_points(),
        SketchSplineIntent::Control,
        false,
        options(),
    )
    .unwrap();
    let before = report.clone();
    let first = report.definition.curve().unwrap();
    let second = report.definition.curve().unwrap();
    first.set_pole(1, Point2::new(80.0, 90.0)).unwrap();
    assert_eq!(second.pole(1).unwrap(), Point2::default());
    assert_eq!(
        report.definition.curve().unwrap().pole(1).unwrap(),
        Point2::default()
    );
    assert_eq!(report, before);
}
#[test]
fn generated_scales_counts_and_near_collinear_curves_keep_their_limits() {
    for count in 3..=12 {
        for scale in [1e-9, 1.0, 1e9] {
            assert_generated_case(count, scale);
        }
    }
}
fn assert_generated_case(count: usize, scale: f64) {
    let points: Vec<Point2> = (0..count)
        .map(|i| {
            Point2::new(
                i as f64 * scale,
                ((i * i + 3 * i) % 11) as f64 * scale * 1e-6,
            )
        })
        .collect();
    let mut limits = options();
    limits.maximum_fit_residual *= scale;
    limits.maximum_seam_residuals = limits.maximum_seam_residuals.map(|limit| limit * scale);
    for closed in [false, true] {
        for intent in [
            SketchSplineIntent::Control,
            SketchSplineIntent::Fit(spacing(0)),
            SketchSplineIntent::Fit(spacing(1)),
            SketchSplineIntent::Fit(spacing(2)),
        ] {
            let report = build_unhandled_spline(&points, intent, closed, limits).unwrap();
            assert_eq!(report.options, limits);
            assert!(
                report
                    .fit_residual
                    .is_none_or(|error| error <= limits.maximum_fit_residual)
            );
            assert_seam_limits(&report);
        }
    }
}
fn assert_seam_limits(report: &SketchSplineReport) {
    if let Some(errors) = report.seam_residuals {
        for (actual, bound) in errors
            .into_iter()
            .zip(report.options.maximum_seam_residuals)
        {
            assert!(actual <= bound, "seam error {actual}: expected <= {bound}");
        }
    }
}
#[test]
fn stricter_requested_fit_limit_fails_instead_of_widening_it() {
    let intent = SketchSplineIntent::Fit(SketchFitSpacing::Centripetal);
    let baseline = build_unhandled_spline(&source_points(), intent, false, options()).unwrap();
    let measured = baseline.fit_residual.unwrap();
    assert!(measured > 0.0);
    let mut strict = options();
    strict.maximum_fit_residual = measured * 0.5;
    let failure = build_unhandled_spline(&source_points(), intent, false, strict).unwrap_err();
    assert!(failure.to_string().contains("fit station"));
    assert_eq!(
        build_unhandled_spline(&source_points(), intent, false, options()).unwrap(),
        baseline
    );
}
#[test]
fn pivot_refusal_and_nonrepresentable_chords_do_not_return_curves() {
    let mut strict = options();
    strict.minimum_pivot = 10.0;
    let failure = build_unhandled_spline(
        &source_points(),
        SketchSplineIntent::Fit(spacing(1)),
        false,
        strict,
    )
    .unwrap_err();
    assert!(failure.to_string().contains("factorization"));
    for scale in [1e-250, 1e250] {
        let points = [Point2::default(), Point2::new(scale, scale)];
        let failure = build_unhandled_spline(
            &points,
            SketchSplineIntent::Fit(spacing(1)),
            false,
            options(),
        )
        .unwrap_err();
        assert!(failure.to_string().contains("station"));
    }
}
#[test]
fn uniform_coincident_fit_is_constant_without_a_polygon_fallback() {
    let points = vec![Point2::new(2.0, 3.0); 5];
    for closed in [false, true] {
        let report = build_unhandled_spline(
            &points,
            SketchSplineIntent::Fit(SketchFitSpacing::Uniform),
            closed,
            options(),
        )
        .unwrap();
        let sample = report.definition.curve().unwrap().eval_d1(0.37).unwrap();
        assert_close(sample.point.x(), 2.0);
        assert_close(sample.point.y(), 3.0);
        assert_close(sample.d1.x(), 0.0);
        assert_close(sample.d1.y(), 0.0);
    }
}

#[test]
fn reconstructed_recipes_match_occt_positions_and_two_derivative_orders() {
    let reports: Vec<SketchSplineReport> = (0..6)
        .map(|case| {
            build_unhandled_spline(
                &source_points(),
                SketchSplineIntent::Fit(spacing(case % 3)),
                case >= 3,
                options(),
            )
            .unwrap()
        })
        .collect();
    let rows: Vec<&str> = include_str!("occt-8.0.0.csv").lines().collect();
    assert_eq!(rows.len(), 135);
    for row in rows {
        oracle::assert_occt_row(row, &reports);
    }
}
#[test]
fn control_recipes_match_go_and_independent_occt_derivatives() {
    let reports: Vec<SketchSplineReport> = [false, true]
        .into_iter()
        .map(|closed| {
            build_unhandled_spline(
                &source_points(),
                SketchSplineIntent::Control,
                closed,
                options(),
            )
            .unwrap()
        })
        .collect();
    let samples: usize = include_str!("go-control-oracle.csv")
        .lines()
        .map(|row| oracle::assert_row(row, &reports))
        .sum();
    assert_eq!(samples, 34);
    let rows: Vec<&str> = include_str!("occt-control-8.0.0.csv").lines().collect();
    assert_eq!(rows.len(), 34);
    for row in rows {
        oracle::assert_occt_row(row, &reports);
    }
}
#[test]
fn stricter_seam_limit_refuses_the_measured_gap() {
    let intent = SketchSplineIntent::Fit(SketchFitSpacing::Centripetal);
    let baseline = build_unhandled_spline(&source_points(), intent, true, options()).unwrap();
    let errors = baseline.seam_residuals.unwrap();
    let order = errors.iter().position(|error| *error > 0.0).unwrap();
    let mut strict = options();
    strict.maximum_seam_residuals[order] = errors[order] * 0.5;
    let failure = build_unhandled_spline(&source_points(), intent, true, strict).unwrap_err();
    assert!(failure.to_string().contains("closed derivative order"));
}
#[test]
fn every_numerical_limit_requires_a_finite_positive_value() {
    for value in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
        for index in 0..5 {
            assert_invalid_limit(index, value);
        }
    }
}
fn assert_invalid_limit(index: usize, value: f64) {
    let mut limits = options();
    match index {
        0 => limits.minimum_pivot = value,
        1 => limits.maximum_fit_residual = value,
        _ => limits.maximum_seam_residuals[index - 2] = value,
    }
    let failure =
        build_unhandled_spline(&source_points(), SketchSplineIntent::Control, false, limits)
            .unwrap_err();
    assert!(failure.to_string().contains("options"));
}

#[test]
fn version_one_defaults_build_both_intents_and_retain_effective_limits() {
    let limits = SketchSplineOptionsV1::default();
    assert_eq!(limits.maximum_fit_residual, 1e-7);
    assert_eq!(limits.maximum_seam_residuals, [1e-7; 3]);
    for intent in [
        SketchSplineIntent::Control,
        SketchSplineIntent::Fit(SketchFitSpacing::Centripetal),
    ] {
        let report = build_unhandled_spline(&source_points(), intent, true, limits).unwrap();
        assert_eq!(report.options, limits);
        assert_seam_limits(&report);
    }
}
