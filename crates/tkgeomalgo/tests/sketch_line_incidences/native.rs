// SPDX-License-Identifier: GPL-2.0-only
//! Certified contacts against native OCCT 8.0.0 and the mapped Rust intersector.
use super::corpus::{CorpusCase, mapped_line, mapped_target, parse_case, values};
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

/// Comparison slack for native points: OCCT reports rounded solver output, not roots.
const NATIVE_SLACK: f64 = 1e-8;
const ERROR: f64 = 1e-12;

fn corpus() -> Vec<CorpusCase> {
    include_str!("native-cases.csv")
        .lines()
        .map(parse_case)
        .collect()
}
fn native_points() -> Vec<Vec<[f64; 2]>> {
    include_str!("occt-8.0.0.csv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split(';').collect();
            assert_eq!(
                fields[3], "0",
                "native corpus has no overlap segments: {row}"
            );
            let points: Vec<[f64; 2]> = fields[2]
                .split('|')
                .filter(|text| !text.is_empty())
                .map(|text| {
                    let v = values(text);
                    [v[0], v[1]]
                })
                .collect();
            assert_eq!(points.len(), fields[1].parse::<usize>().unwrap());
            points
        })
        .collect()
}
pub fn certify_case(case: &CorpusCase) -> SketchLineIncidenceReport {
    let source = SketchIncidenceSource::line(0, case.source, SketchLineDomain::Support);
    let target = SketchIncidenceTarget {
        id: case.id as u64,
        curve: case.target.clone(),
    };
    certify_sketch_line_incidences(source, &[target], SketchIncidenceOptions::v1(ERROR), || {
        false
    })
    .unwrap_or_else(|failure| panic!("case {} ({}): {failure}", case.id, case.kind))
}
/// Each approximate point lies near a distinct certified contact, and counts agree.
pub fn assert_covers(case: &CorpusCase, report: &SketchLineIncidenceReport, points: &[[f64; 2]]) {
    let contacts = report.targets()[0].contacts();
    assert!(
        report.targets()[0].overlaps().is_empty(),
        "case {}",
        case.id
    );
    assert_eq!(
        contacts.len(),
        points.len(),
        "case {} ({}): {contacts:?} vs {points:?}",
        case.id,
        case.kind
    );
    let mut used = vec![false; contacts.len()];
    for point in points {
        let (index, distance) = nearest(contacts, *point, &used);
        let slack = contacts[index].achieved_world_error() + NATIVE_SLACK;
        assert!(
            distance <= slack,
            "case {} ({}): {point:?} is {distance} from {:?}",
            case.id,
            case.kind,
            contacts[index]
        );
        used[index] = true;
    }
}
fn nearest(contacts: &[SketchLineContact], point: [f64; 2], used: &[bool]) -> (usize, f64) {
    contacts
        .iter()
        .enumerate()
        .filter(|(index, _)| !used[*index])
        .map(|(index, contact)| {
            (
                index,
                (contact.point()[0] - point[0]).hypot(contact.point()[1] - point[1]),
            )
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .expect("an unused certified contact for every native point")
}
#[test]
fn certified_contacts_match_every_native_occt_point() {
    let cases = corpus();
    let native = native_points();
    assert_eq!((cases.len(), native.len()), (38, 38));
    for (case, points) in cases.iter().zip(&native) {
        assert_covers(case, &certify_case(case), points);
    }
}
#[test]
fn certified_contacts_match_every_mapped_rust_intersector_point() {
    for case in corpus() {
        let mapped = InterCurveCurve::between(
            &mapped_line(case.source),
            &mapped_target(&case.target),
            1e-9,
        )
        .unwrap();
        assert_eq!(mapped.nb_segments(), 0, "case {}", case.id);
        let points: Vec<[f64; 2]> = (1..=mapped.nb_points())
            .map(|index| {
                let point = mapped.point(index).unwrap();
                [point.x(), point.y()]
            })
            .collect();
        assert_covers(&case, &certify_case(&case), &points);
    }
}
