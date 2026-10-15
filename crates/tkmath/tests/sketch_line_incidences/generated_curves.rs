// SPDX-License-Identifier: GPL-2.0-only
//! Generated near-degenerate conic- and spline-source cases against the exact
//! Fraction oracle (`oracle_curves.py`).
use super::*;

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
fn transition(code: &str) -> SketchContactTransition {
    match code {
        "C" => SketchContactTransition::Crossing,
        "T" => SketchContactTransition::Touching,
        "E" => SketchContactTransition::TargetEnd,
        "O" => SketchContactTransition::SourceEnd,
        other => panic!("transition code {other}; expected C, T, E or O"),
    }
}
fn side(field: &str) -> Option<usize> {
    (field != "-").then(|| field.parse().unwrap())
}
/// Curve spec: circle, ellipse, segment, line (unbounded), spline, offset or periodic.
pub(super) fn parse_curve(spec: &str) -> SketchIncidenceCurve {
    let (kind, rest) = spec.split_once(':').unwrap();
    match kind {
        "circle" => parse_circle(rest),
        "circlearc" | "ellipsearc" => parse_angle_arc(kind, rest),
        "ellipse" => {
            let v = values(rest);
            ellipse([v[0], v[1]], [v[2], v[3]], [v[4], v[5]])
        }
        "segment" | "line" => {
            let points = pairs(&values(rest));
            let bounded = kind == "segment";
            SketchIncidenceCurve::Line {
                points: [points[0], points[1]],
                bounded,
            }
        }
        "spline" => {
            let fields: Vec<&str> = rest.split(':').collect();
            let bounds = values(fields[3]);
            spline(
                fields[0].parse().unwrap(),
                &pairs(&values(fields[1])),
                &values(fields[2]),
                [bounds[0], bounds[1]],
            )
        }
        "offset" => {
            let (distance, basis) = rest.split_once(':').unwrap();
            SketchIncidenceCurve::Offset(SketchIncidenceOffset {
                basis: Box::new(parse_curve(basis)),
                distance: bits(distance),
            })
        }
        "periodic" => parse_periodic(rest),
        other => panic!("curve kind {other}; expected circle, ellipse, segment, line or spline"),
    }
}
// periodic:degree:poles:knots:multiplicities:bounds, multiplicities as decimal integers.
fn parse_periodic(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(':').collect();
    let bounds = values(fields[4]);
    SketchIncidenceCurve::PeriodicSpline(SketchPeriodicSpline {
        degree: fields[0].parse().unwrap(),
        poles: pairs(&values(fields[1])),
        weights: None,
        knots: values(fields[2]),
        multiplicities: fields[3].split(',').map(|m| m.parse().unwrap()).collect(),
        bounds: [bounds[0], bounds[1]],
    })
}
// circlearc:cx,cy,r,ax,ay,t0,t1,ccw or ellipsearc:cx,cy,mx,my,ra,rb,ax,ay,t0,t1,ccw.
fn parse_angle_arc(kind: &str, rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(',').collect();
    let v: Vec<f64> = fields[..fields.len() - 1]
        .iter()
        .map(|field| bits(field))
        .collect();
    let counterclockwise = fields[fields.len() - 1] == "1";
    let angles = |offset: usize| SketchConicAngles {
        axis: [v[offset], v[offset + 1]],
        counterclockwise,
        bounds: [v[offset + 2], v[offset + 3]],
    };
    if kind == "circlearc" {
        return SketchIncidenceCurve::Circle {
            center: [v[0], v[1]],
            radius: v[2],
            sector: Some(angles(3).into()),
        };
    }
    SketchIncidenceCurve::Ellipse {
        center: [v[0], v[1]],
        major_axis: [v[2], v[3]],
        radii: [v[4], v[5]],
        sector: Some(angles(6).into()),
    }
}
fn parse_circle(rest: &str) -> SketchIncidenceCurve {
    let fields: Vec<&str> = rest.split(',').collect();
    let v: Vec<f64> = fields.iter().take(7).map(|field| bits(field)).collect();
    if fields.len() == 3 {
        return circle([v[0], v[1]], v[2]);
    }
    arc(
        [v[0], v[1]],
        v[2],
        [v[3], v[4]],
        [v[5], v[6]],
        fields[7] == "1",
    )
}
struct OracleContact {
    target: usize,
    multiplicity: [Option<usize>; 2],
    source: SketchLinePosition,
    target_position: SketchLinePosition,
    transition: SketchContactTransition,
    coordinates: [[f64; 2]; 2],
}
fn parse_contact(text: &str) -> OracleContact {
    let fields: Vec<&str> = text.split(',').collect();
    let numbers: Vec<f64> = fields[6..].iter().map(|field| bits(field)).collect();
    OracleContact {
        target: fields[0].parse().unwrap(),
        multiplicity: [side(fields[1]), side(fields[2])],
        source: position(fields[3]),
        target_position: position(fields[4]),
        transition: transition(fields[5]),
        coordinates: [[numbers[0], numbers[1]], [numbers[2], numbers[3]]],
    }
}
fn check_contact(case: &str, actual: &SketchLineContact, expected: &OracleContact, limit: f64) {
    let facts = (
        actual.target_index(),
        actual.multiplicity(),
        actual.source_position(),
        actual.target_position(),
        actual.transition(),
    );
    let wanted = (
        expected.target,
        expected.multiplicity,
        expected.source,
        expected.target_position,
        expected.transition,
    );
    assert_eq!(facts, wanted, "case {case}: {actual:?}");
    assert!(actual.achieved_world_error() <= limit, "case {case}");
    for axis in 0..2 {
        let [lower, upper] = actual.world_bounds()[axis];
        let [exact_lower, exact_upper] = expected.coordinates[axis];
        assert!(
            lower <= exact_upper && exact_lower <= upper,
            "case {case} axis {axis}: {actual:?}"
        );
    }
}
fn domain(code: &str) -> SketchLineDomain {
    match code {
        "segment" => SketchLineDomain::Segment,
        "support" => SketchLineDomain::Support,
        other => panic!("domain {other}; expected segment or support"),
    }
}
fn magnitude(curves: &[&SketchIncidenceCurve]) -> f64 {
    curves.iter().map(|curve| scale(curve)).fold(1f64, f64::max)
}
fn scale(curve: &SketchIncidenceCurve) -> f64 {
    match curve {
        SketchIncidenceCurve::Circle { center, radius, .. } => {
            center[0].abs().max(center[1].abs()).max(*radius)
        }
        SketchIncidenceCurve::Ellipse { center, radii, .. } => {
            center[0].abs().max(center[1].abs()).max(radii[0])
        }
        SketchIncidenceCurve::Line { points, .. } => largest(points),
        SketchIncidenceCurve::Spline(spline) => largest(&spline.poles),
        SketchIncidenceCurve::PeriodicSpline(spline) => largest(&spline.poles),
        SketchIncidenceCurve::Offset(offset) => scale(&offset.basis) + offset.distance.abs(),
    }
}
fn largest(poles: &[[f64; 2]]) -> f64 {
    poles
        .iter()
        .flatten()
        .fold(0f64, |size, value| size.max(value.abs()))
}
pub(super) fn check_row(row: &str) {
    let fields: Vec<&str> = row.split(';').collect();
    let case = fields[0];
    let source_curve = parse_curve(fields[3]);
    let targets: Vec<SketchIncidenceTarget> = fields[4]
        .split('|')
        .enumerate()
        .map(|(index, spec)| target(index as u64, parse_curve(spec)))
        .collect();
    let mut curves = vec![&source_curve];
    curves.extend(targets.iter().map(|target| &target.curve));
    // A relative 1e-12 bound; an absolute one is below binary64 resolution at large scales.
    let limit = magnitude(&curves) * ERROR;
    let source = curve_source(source_curve.clone(), domain(fields[2]));
    let report = certify_with(source, &targets, SketchIncidenceOptions::v1(limit));
    let expected: Vec<OracleContact> = fields[5]
        .split('|')
        .filter(|text| !text.is_empty())
        .map(parse_contact)
        .collect();
    let actual = ordered(&report);
    assert_eq!(
        actual.len(),
        expected.len(),
        "case {case} ({}): {report:#?}",
        fields[1]
    );
    for (actual, expected) in actual.iter().zip(&expected) {
        check_contact(case, actual, expected, limit);
    }
}
#[test]
fn generated_curve_source_cases_match_the_exact_oracle() {
    let rows: Vec<&str> = include_str!("oracle-curves.csv").lines().collect();
    assert_eq!(rows.len(), 244);
    for row in rows {
        check_row(row);
    }
}
