// SPDX-License-Identifier: GPL-2.0-only
//! Offset and periodic-spline certified contacts against native OCCT 8.0.0 and the
//! mapped Rust intersector (ADR 0080). Native points are rounded solver output;
//! counts must agree and each native point must lie within the certified error
//! plus 1e-8 of a distinct contact.
use super::corpus::{Curve, mapped_target, parse_target, values};
use super::mapped_offsets::mapped_extension;
use super::native_curves::{Location, assert_covers_contacts};
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

const ERROR: f64 = 1e-12;

struct OffsetCase {
    id: usize,
    kind: String,
    source: SketchIncidenceCurve,
    target: SketchIncidenceCurve,
}
fn corpus() -> Vec<OffsetCase> {
    include_str!("native-offset-cases.csv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split(';').collect();
            OffsetCase {
                id: fields[0].parse().unwrap(),
                kind: fields[1].to_owned(),
                source: parse_target(fields[2]),
                target: parse_target(fields[3]),
            }
        })
        .collect()
}
fn native_points() -> Vec<Vec<Location>> {
    include_str!("occt-offsets-8.0.0.csv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split(';').collect();
            assert_eq!(fields[3], "0", "no native coincidence zones: {row}");
            fields[2]
                .split('|')
                .filter(|text| !text.is_empty())
                .map(|text| {
                    let v = values(text);
                    Location {
                        point: [v[0], v[1]],
                        radius: 0.,
                    }
                })
                .collect()
        })
        .collect()
}
fn certify_case(case: &OffsetCase) -> SketchLineIncidenceReport {
    let source = SketchIncidenceSource {
        id: 0,
        curve: case.source.clone(),
        domain: SketchLineDomain::Segment,
    };
    let target = SketchIncidenceTarget {
        id: case.id as u64,
        curve: case.target.clone(),
    };
    certify_sketch_line_incidences(source, &[target], SketchIncidenceOptions::v1(ERROR), || {
        false
    })
    .unwrap_or_else(|failure| panic!("case {} ({}): {failure}", case.id, case.kind))
}
fn label(case: &OffsetCase) -> String {
    format!("case {} ({})", case.id, case.kind)
}
fn mapped(curve: &SketchIncidenceCurve) -> Curve {
    match curve {
        SketchIncidenceCurve::Offset(_) | SketchIncidenceCurve::PeriodicSpline(_) => {
            mapped_extension(curve)
        }
        other => mapped_target(other),
    }
}
#[test]
fn certified_offset_contacts_match_every_native_occt_point() {
    let cases = corpus();
    let native = native_points();
    assert_eq!((cases.len(), native.len()), (108, 108));
    for (case, found) in cases.iter().zip(&native) {
        assert_covers_contacts(&label(case), &certify_case(case), found);
    }
}
#[test]
fn certified_offset_contacts_match_the_mapped_rust_intersector() {
    for case in corpus() {
        let mapped = InterCurveCurve::between(&mapped(&case.source), &mapped(&case.target), 1e-9)
            .unwrap();
        assert_eq!(mapped.nb_segments(), 0, "{}", label(&case));
        let found: Vec<Location> = (1..=mapped.nb_points())
            .map(|index| {
                let point = mapped.point(index).unwrap();
                Location {
                    point: [point.x(), point.y()],
                    radius: 0.,
                }
            })
            .collect();
        assert_covers_contacts(&label(&case), &certify_case(&case), &found);
    }
}
