// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn assert_row(row: &str, reports: &[SketchSplineReport]) -> usize {
    let fields: Vec<&str> = row.split(',').collect();
    let report = &reports[fields[0].parse::<usize>().unwrap()];
    let index = fields[2].parse::<usize>().unwrap();
    let values: Vec<f64> = fields[3..]
        .iter()
        .map(|field| field.parse().unwrap())
        .collect();
    match fields[1] {
        "shape" => assert_shape(report, &values),
        "pole" => assert_point(report.definition.poles()[index], &values),
        "knot" => assert_close(report.definition.flat_knots()[index], values[0]),
        "station" => assert_close(report.stations[index], values[0]),
        "sample" => {
            assert_sample(report, &values);
            return 1;
        }
        kind => panic!("oracle kind {kind:?}: expected shape, pole, knot, station or sample"),
    }
    0
}
fn assert_shape(report: &SketchSplineReport, values: &[f64]) {
    assert_eq!(report.definition.degree(), values[0] as i32);
    assert_eq!(report.definition.poles().len(), values[1] as usize);
    assert_eq!(report.definition.flat_knots().len(), values[2] as usize);
    assert_eq!(report.stations.len(), values[3] as usize);
}
fn assert_point(point: Point2, values: &[f64]) {
    assert_close(point.x(), values[0]);
    assert_close(point.y(), values[1]);
}
fn assert_sample(report: &SketchSplineReport, values: &[f64]) {
    let sample = report
        .definition
        .curve()
        .unwrap()
        .eval_d1(values[0])
        .unwrap();
    assert_point(sample.point, &values[1..3]);
    assert_close(sample.d1.x(), values[3]);
    assert_close(sample.d1.y(), values[4]);
}

pub(super) fn assert_occt_row(row: &str, reports: &[SketchSplineReport]) {
    let values: Vec<f64> = row.split(',').map(|value| value.parse().unwrap()).collect();
    let sample = reports[values[0] as usize]
        .definition
        .curve()
        .unwrap()
        .eval_d2(values[1])
        .unwrap();
    let actual = [
        sample.point.x(),
        sample.point.y(),
        sample.d1.x(),
        sample.d1.y(),
        sample.d2.x(),
        sample.d2.y(),
    ];
    for (actual, expected) in actual.into_iter().zip(&values[2..]) {
        assert_close(actual, *expected);
    }
}
