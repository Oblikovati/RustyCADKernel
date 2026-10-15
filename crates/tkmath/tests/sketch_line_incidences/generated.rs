// SPDX-License-Identifier: GPL-2.0-only
//! Generated near-degenerate cases against the exact Fraction oracle (`oracle.py`).
use super::*;

struct OracleContact {
    target: usize,
    multiplicity: [Option<usize>; 2],
    line: SketchLinePosition,
    target_position: SketchLinePosition,
    coordinates: [[f64; 2]; 2],
}
fn bits(field: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(field, 16).unwrap_or_else(|_| panic!("hex {field}")))
}
fn values(fields: &str) -> Vec<f64> {
    fields.split(',').map(bits).collect()
}
fn pairs(values: &[f64]) -> Vec<[f64; 2]> {
    values.chunks(2).map(|pair| [pair[0], pair[1]]).collect()
}
fn position(code: &str) -> SketchLinePosition {
    match code {
        "B" => SketchLinePosition::BeforeStart,
        "S" => SketchLinePosition::AtStart,
        "I" => SketchLinePosition::Interior,
        "E" => SketchLinePosition::AtEnd,
        "A" => SketchLinePosition::AfterEnd,
        other => panic!("position code {other}; expected B, S, I, E or A"),
    }
}
fn side(field: &str) -> Option<usize> {
    (field != "-").then(|| field.parse().unwrap())
}
fn conic_sector(fields: &[&str]) -> Option<SketchConicSector> {
    let numbers: Vec<f64> = fields.iter().take(4).map(|field| bits(field)).collect();
    (fields.len() == 5).then(|| SketchConicSector {
        start: [numbers[0], numbers[1]],
        end: [numbers[2], numbers[3]],
        counterclockwise: fields[4] == "1",
    })
}
fn parse_target(id: u64, spec: &str) -> SketchIncidenceTarget {
    let (kind, rest) = spec.split_once(':').unwrap();
    let curve = match kind {
        "circle" => {
            let fields: Vec<&str> = rest.split(',').collect();
            let numbers: Vec<f64> = fields[..3].iter().map(|field| bits(field)).collect();
            let sector = conic_sector(&fields[3..]);
            SketchIncidenceCurve::Circle {
                center: [numbers[0], numbers[1]],
                radius: numbers[2],
                sector: sector.map(Into::into),
            }
        }
        "ellipse" => {
            let numbers = values(rest);
            ellipse(
                [numbers[0], numbers[1]],
                [numbers[2], numbers[3]],
                [numbers[4], numbers[5]],
            )
        }
        "segment" => {
            let points = pairs(&values(rest));
            segment(points[0], points[1])
        }
        "spline" => parse_spline(rest),
        other => panic!("target kind {other}; expected circle, ellipse, segment or spline"),
    };
    target(id, curve)
}
fn parse_spline(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(':').collect();
    let bounds = values(fields[3]);
    spline(
        fields[0].parse().unwrap(),
        &pairs(&values(fields[1])),
        &values(fields[2]),
        [bounds[0], bounds[1]],
    )
}
fn parse_contact(text: &str) -> OracleContact {
    let fields: Vec<&str> = text.split(',').collect();
    let numbers: Vec<f64> = fields[5..].iter().map(|field| bits(field)).collect();
    OracleContact {
        target: fields[0].parse().unwrap(),
        multiplicity: [side(fields[1]), side(fields[2])],
        line: position(fields[3]),
        target_position: position(fields[4]),
        coordinates: [[numbers[0], numbers[1]], [numbers[2], numbers[3]]],
    }
}
fn check_contact(case: &str, actual: &SketchLineContact, expected: &OracleContact, limit: f64) {
    assert_eq!(actual.target_index(), expected.target, "case {case}");
    assert_eq!(actual.multiplicity(), expected.multiplicity, "case {case}");
    assert_eq!(actual.source_position(), expected.line, "case {case}");
    assert_eq!(
        actual.target_position(),
        expected.target_position,
        "case {case}"
    );
    assert!(actual.achieved_world_error() <= limit, "case {case}");
    for axis in 0..2 {
        let [lower, upper] = actual.world_bounds()[axis];
        let [exact_lower, exact_upper] = expected.coordinates[axis];
        // The exact coordinate lies in [exact_lower, exact_upper] and in the certified bounds.
        assert!(
            lower <= exact_upper && exact_lower <= upper,
            "case {case} axis {axis}: {actual:?}"
        );
    }
}
fn check_row(row: &str) {
    let fields: Vec<&str> = row.split(';').collect();
    let case = fields[0];
    let source_points = pairs(&values(fields[3]));
    let targets: Vec<SketchIncidenceTarget> = fields[4]
        .split('|')
        .enumerate()
        .map(|(index, spec)| parse_target(index as u64, spec))
        .collect();
    let line = source(
        [source_points[0], source_points[1]],
        SketchLineDomain::Support,
    );
    // Request a relative 1e-12 bound; an absolute one is below binary64 resolution at 1e9.
    let magnitude = source_points
        .iter()
        .flatten()
        .fold(1f64, |size, value| size.max(value.abs()));
    let limit = magnitude * ERROR;
    let report = certify_with(line, &targets, SketchIncidenceOptions::v1(limit));
    let expected: Vec<OracleContact> = if fields[5].is_empty() {
        Vec::new()
    } else {
        fields[5].split('|').map(parse_contact).collect()
    };
    let actual = ordered(&report);
    assert_eq!(
        actual.len(),
        expected.len(),
        "case {case} ({}): {actual:?}",
        fields[1]
    );
    assert_eq!(
        report.ordered().len(),
        expected.len(),
        "case {case}: generated cases have no ties"
    );
    for (actual, expected) in actual.iter().zip(&expected) {
        check_contact(case, actual, expected, limit);
    }
}
#[test]
fn generated_near_degenerate_cases_match_the_exact_oracle() {
    let rows: Vec<&str> = include_str!("oracle.csv").lines().collect();
    assert_eq!(rows.len(), 195);
    for row in rows {
        check_row(row);
    }
}
