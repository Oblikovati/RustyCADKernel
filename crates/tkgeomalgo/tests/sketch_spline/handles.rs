// SPDX-License-Identifier: GPL-2.0-only
use super::{assert_close, options, source_points, spacing};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::sketch_spline::*;
use rustycad_tkmath::gp::Point2;
pub(super) fn limits() -> SketchHandleOptionsV1 {
    SketchHandleOptionsV1 {
        natural: options(),
        maximum_position_residual: 1e-10,
        maximum_derivative_residual: 1e-9,
    }
}
pub(super) fn handles() -> Vec<SketchSplineHandle> {
    vec![
        SketchSplineHandle {
            fit_index: 0,
            end: Point2::new(0.5, 0.25),
        },
        SketchSplineHandle {
            fit_index: 2,
            end: Point2::new(1.75, 2.0),
        },
        SketchSplineHandle {
            fit_index: 4,
            end: Point2::new(1.0, -2.0),
        },
    ]
}
#[test]
fn both_station_sides_obey_handles_for_all_spacing_and_closure_choices() {
    for closed in [false, true] {
        for method in 0..3 {
            let report = build_handled_spline(
                &source_points(),
                spacing(method),
                closed,
                &handles(),
                limits(),
            )
            .unwrap();
            assert_eq!(report.ignored_coincident_handles, [4]);
            assert!(report.station_residuals[0] < limits().maximum_position_residual);
            assert!(report.station_residuals[1] < limits().maximum_derivative_residual);
            assert_eq!(report.seam_residuals.is_some(), closed);
            assert_station_sides(&report);
        }
    }
}
fn assert_station_sides(report: &SketchHandledSplineReport) {
    let curve = report.definition.curve().unwrap();
    for (span, pair) in report.natural.stations.windows(2).enumerate() {
        for (side, &parameter) in pair.iter().enumerate() {
            let (point, derivative) = curve
                .local_d1(parameter, span as i32 + 1, span as i32 + 2)
                .unwrap();
            let index = (span + side) % report.derivatives.len();
            assert_close(point.x(), source_points()[index].x());
            assert_close(point.y(), source_points()[index].y());
            assert_close(derivative.x(), report.derivatives[index].x());
            assert_close(derivative.y(), report.derivatives[index].y());
        }
    }
}
#[test]
fn handle_order_and_fresh_native_mutation_do_not_change_owned_results() {
    let mut authored = handles();
    let first =
        build_handled_spline(&source_points(), spacing(0), true, &authored, limits()).unwrap();
    authored.reverse();
    let second =
        build_handled_spline(&source_points(), spacing(0), true, &authored, limits()).unwrap();
    assert_eq!(first, second);
    first
        .definition
        .curve()
        .unwrap()
        .set_pole(1, Point2::new(99.0, 98.0))
        .unwrap();
    assert_eq!(
        first.definition.curve().unwrap().value(0.0).unwrap(),
        source_points()[0]
    );
}
#[test]
fn closed_handles_have_checked_first_order_seams_without_a_false_c2_claim() {
    let report =
        build_handled_spline(&source_points(), spacing(0), true, &handles(), limits()).unwrap();
    let curve = report.definition.curve().unwrap();
    let first = curve.local_d2(0.0, 1, 2).unwrap();
    let last = curve.local_d2(1.0, 5, 6).unwrap();
    assert!((first.2.x() - last.2.x()).hypot(first.2.y() - last.2.y()) > 1.0);
    assert!(report.seam_residuals.unwrap()[1] < limits().maximum_derivative_residual);
    assert!(!curve.is_cn(1).unwrap());
}
#[test]
fn coincident_handle_still_reconstructs_every_span_and_reports_its_declined_override() {
    let authored = [SketchSplineHandle {
        fit_index: 0,
        end: source_points()[0],
    }];
    let report =
        build_handled_spline(&source_points(), spacing(0), false, &authored, limits()).unwrap();
    assert_eq!(report.ignored_coincident_handles, [0]);
    assert_eq!(report.definition.poles().len(), 13);
    assert_eq!(report.natural.definition.poles().len(), 5);
    let natural = report.natural.definition.curve().unwrap();
    let handled = report.definition.curve().unwrap();
    assert!((natural.value(0.5).unwrap().x() - handled.value(0.5).unwrap().x()).abs() > 1e-5);
}
#[test]
fn malformed_handles_and_limits_are_refused_without_mutation() {
    let source = source_points();
    let before = source.clone();
    for authored in [
        vec![],
        vec![handles()[0]; 2],
        vec![SketchSplineHandle {
            fit_index: 5,
            end: source[0],
        }],
        vec![SketchSplineHandle {
            fit_index: 0,
            end: Point2::new(f64::NAN, 0.0),
        }],
    ] {
        assert!(build_handled_spline(&source, spacing(0), false, &authored, limits()).is_err());
    }
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        for field in 0..2 {
            let mut policy = limits();
            if field == 0 {
                policy.maximum_position_residual = value;
            } else {
                policy.maximum_derivative_residual = value;
            }
            assert!(build_handled_spline(&source, spacing(0), false, &handles(), policy).is_err());
        }
    }
    assert_eq!(source, before);
}
#[test]
fn nonrepresentable_handle_lengths_and_strict_derivative_limits_are_named_failures() {
    for end in [Point2::new(1e200, 0.0), Point2::new(1e-200, 0.0)] {
        let authored = [SketchSplineHandle { fit_index: 0, end }];
        let failure =
            build_handled_spline(&source_points(), spacing(0), false, &authored, limits())
                .unwrap_err();
        assert!(failure.to_string().contains("representable finite length"));
    }
    let policy = SketchHandleOptionsV1 {
        maximum_derivative_residual: f64::MIN_POSITIVE,
        ..limits()
    };
    assert!(
        build_handled_spline(&source_points(), spacing(0), false, &handles(), policy)
            .unwrap_err()
            .to_string()
            .contains("residual")
    );
}
#[test]
fn two_point_fit_and_zero_neighbor_chords_have_defined_handle_semantics() {
    let source = [Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)];
    let authored = [SketchSplineHandle {
        fit_index: 0,
        end: Point2::new(0.0, 2.0),
    }];
    let report = build_handled_spline(
        &source,
        spacing(2),
        false,
        &authored,
        SketchHandleOptionsV1::default(),
    )
    .unwrap();
    assert_close(report.derivatives[0].x(), 0.0);
    assert_close(report.derivatives[0].y(), 6.0);
    let constant = [Point2::new(0.0, 0.0); 3];
    let report = build_handled_spline(&constant, spacing(2), true, &authored, limits()).unwrap();
    assert!(
        report
            .derivatives
            .iter()
            .all(|v| v.x() == 0.0 && v.y() == 0.0)
    );
}
#[test]
fn generated_near_collinear_handles_cover_counts_scales_and_all_station_policies() {
    for count in 3..=9 {
        for scale in [1e-8, 1.0, 1e8] {
            for closed in [false, true] {
                for method in 0..3 {
                    check_generated(count, scale, closed, method);
                }
            }
        }
    }
}
fn check_generated(count: usize, scale: f64, closed: bool, method: usize) {
    let points: Vec<Point2> = (0..count)
        .map(|i| Point2::new(i as f64 * scale, (i * i) as f64 * scale * 1e-5))
        .collect();
    let authored: Vec<SketchSplineHandle> = points
        .iter()
        .enumerate()
        .map(|(fit_index, point)| SketchSplineHandle {
            fit_index,
            end: Point2::new(point.x() + scale * 0.2, point.y() - scale * 0.1),
        })
        .collect();
    let mut policy = limits();
    policy.natural.maximum_fit_residual *= scale;
    policy.natural.maximum_seam_residuals =
        policy.natural.maximum_seam_residuals.map(|v| v * scale);
    policy.maximum_position_residual *= scale;
    policy.maximum_derivative_residual *= scale;
    let report = build_handled_spline(&points, spacing(method), closed, &authored, policy).unwrap();
    assert!(report.ignored_coincident_handles.is_empty());
    assert_eq!(
        report.definition.poles().len(),
        3 * (count - usize::from(!closed)) + 1
    );
    assert_eq!(report.options, policy);
}
#[test]
fn coincident_neighbors_keep_unit_weight_for_a_nonzero_natural_derivative() {
    let points = [
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 0.0),
    ];
    let authored = [SketchSplineHandle {
        fit_index: 1,
        end: Point2::new(1.0, 4.0),
    }];
    let report = build_handled_spline(&points, spacing(2), false, &authored, limits()).unwrap();
    let natural = report
        .natural
        .definition
        .curve()
        .unwrap()
        .eval_d1(report.natural.stations[1])
        .unwrap()
        .d1;
    assert!(natural.x().hypot(natural.y()) > 0.0);
    assert_close(report.derivatives[1].x(), 0.0);
    assert_close(report.derivatives[1].y(), natural.x().hypot(natural.y()));
}
#[test]
fn nonfinite_effective_tangent_is_refused_after_successful_natural_fit() {
    let points = [
        Point2::new(0.0, 0.0),
        Point2::new(1e-150, 0.0),
        Point2::new(1e150, 1e150),
    ];
    let authored = [SketchSplineHandle {
        fit_index: 0,
        end: Point2::new(1e150, 0.0),
    }];
    let policy = SketchHandleOptionsV1 {
        natural: SketchSplineOptionsV1 {
            maximum_fit_residual: 1e140,
            ..options()
        },
        ..limits()
    };
    let failure = build_handled_spline(&points, spacing(2), false, &authored, policy).unwrap_err();
    assert!(
        failure.to_string().contains("effective derivative"),
        "{failure}"
    );
}
