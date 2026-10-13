// SPDX-License-Identifier: GPL-2.0-only
use super::{assert_close, source_points, spacing};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeomalgo::sketch_spline::{SketchHandledSplineReport, build_handled_spline};
fn reports() -> Vec<SketchHandledSplineReport> {
    (0..6)
        .map(|index| {
            build_handled_spline(
                &source_points(),
                spacing(index % 3),
                index >= 3,
                &super::handles::handles(),
                super::handles::limits(),
            )
            .unwrap()
        })
        .collect()
}
#[test]
fn actual_go_handle_methods_and_all_219_display_samples_match_analytic_curves() {
    let cases = reports();
    let mut samples = 0;
    for row in include_str!("go-handles.csv").lines() {
        let fields: Vec<&str> = row.split(',').collect();
        let case: usize = fields[0].parse().unwrap();
        let slot: usize = fields[2].parse().unwrap();
        let values: Vec<f64> = fields[3..]
            .iter()
            .map(|field| field.parse().unwrap())
            .collect();
        let report = &cases[case];
        if fields[1] == "station" {
            assert_station(report, slot, &values);
            continue;
        }
        let point = report.definition.curve().unwrap().value(values[0]).unwrap();
        assert_close(point.x(), values[1]);
        assert_close(point.y(), values[2]);
        samples += 1;
    }
    assert_eq!(samples, 219);
}
fn assert_station(report: &SketchHandledSplineReport, slot: usize, values: &[f64]) {
    assert_close(report.natural.stations[slot], values[0]);
    let tangent = report.derivatives[slot % report.derivatives.len()];
    assert_close(tangent.x(), values[3]);
    assert_close(tangent.y(), values[4]);
}
#[test]
fn independent_occt_bezier_spans_match_both_derivative_orders_including_both_join_sides() {
    let cases = reports();
    let mut samples = 0;
    for row in include_str!("occt-handles-8.0.0.csv").lines() {
        let fields: Vec<&str> = row.split(',').collect();
        let case: usize = fields[0].parse().unwrap();
        let span: i32 = fields[1].parse().unwrap();
        let values: Vec<f64> = fields[2..]
            .iter()
            .map(|field| field.parse().unwrap())
            .collect();
        let curve = cases[case].definition.curve().unwrap();
        let (point, first, second) = curve.local_d2(values[0], span + 1, span + 2).unwrap();
        let actual = [
            point.x(),
            point.y(),
            first.x(),
            first.y(),
            second.x(),
            second.y(),
        ];
        for (&actual, &expected) in actual.iter().zip(&values[1..]) {
            assert_close(actual, expected);
        }
        samples += 1;
    }
    assert_eq!(samples, 243);
}
