// SPDX-License-Identifier: GPL-2.0-only
//! Conic- and spline-source certified contacts against native OCCT 8.0.0 and the
//! mapped Rust intersector (ADR 0061).
use super::corpus::{mapped_target, parse_target, values};
use rustycad_tkgeomalgo::geom2d_api::InterCurveCurve;
use rustycad_tkmath::predicates::roots::sketch_incidence::*;

/// Comparison slack for native points: OCCT reports rounded solver output, not roots.
const NATIVE_SLACK: f64 = 1e-8;
const ERROR: f64 = 1e-12;

struct CurveCase {
    id: usize,
    kind: String,
    source: SketchIncidenceCurve,
    target: SketchIncidenceCurve,
}
/// A native intersection location: a point, or the middle of a tiny coincidence zone.
pub struct Location {
    pub point: [f64; 2],
    pub radius: f64,
}
fn corpus() -> Vec<CurveCase> {
    include_str!("native-curve-cases.csv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split(';').collect();
            CurveCase {
                id: fields[0].parse().unwrap(),
                kind: fields[1].to_owned(),
                source: parse_target(fields[2]),
                target: parse_target(fields[3]),
            }
        })
        .collect()
}
fn pair(text: &str) -> [f64; 2] {
    let v = values(text);
    [v[0], v[1]]
}
// OCCT 8.0.0 reports some circle/circle crossings as coincidence zones about one
// tolerance long instead of points; each zone stands for one crossing at its middle.
fn zone(text: &str) -> Location {
    let v = values(text);
    let point = [(v[0] + v[2]) / 2., (v[1] + v[3]) / 2.];
    Location {
        point,
        radius: (v[2] - v[0]).hypot(v[3] - v[1]) / 2.,
    }
}
fn native_locations() -> Vec<Vec<Location>> {
    include_str!("occt-curves-8.0.0.csv")
        .lines()
        .map(|row| {
            let fields: Vec<&str> = row.split(';').collect();
            let mut found: Vec<Location> = fields[2]
                .split('|')
                .filter(|text| !text.is_empty())
                .map(|text| Location {
                    point: pair(text),
                    radius: 0.,
                })
                .collect();
            assert_eq!(found.len(), fields[1].parse::<usize>().unwrap(), "{row}");
            if let Some(zones) = fields.get(4) {
                found.extend(zones.split('|').map(zone));
            }
            assert_eq!(
                found.len() - fields[1].parse::<usize>().unwrap(),
                fields[3].parse::<usize>().unwrap()
            );
            found
        })
        .collect()
}
fn certify_case(case: &CurveCase) -> SketchLineIncidenceReport {
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
fn assert_covers(case: &CurveCase, report: &SketchLineIncidenceReport, found: &[Location]) {
    assert_covers_contacts(&format!("case {} ({})", case.id, case.kind), report, found);
}
/// Each native location lies near a distinct in-domain certified contact; counts agree.
pub fn assert_covers_contacts(case: &str, report: &SketchLineIncidenceReport, found: &[Location]) {
    let contacts: Vec<&SketchLineContact> = report
        .ordered()
        .iter()
        .flat_map(|group| group.members().iter().map(|member| report.contact(*member)))
        .collect();
    assert_eq!(contacts.len(), found.len(), "{case}: {contacts:?}");
    let mut used = vec![false; contacts.len()];
    for location in found {
        let (index, distance) = nearest(&contacts, location.point, &used);
        let slack = contacts[index].achieved_world_error() + location.radius + NATIVE_SLACK;
        assert!(
            distance <= slack,
            "{case}: {:?} is {distance} from {:?}",
            location.point,
            contacts[index]
        );
        used[index] = true;
    }
}
fn nearest(contacts: &[&SketchLineContact], point: [f64; 2], used: &[bool]) -> (usize, f64) {
    contacts
        .iter()
        .enumerate()
        .filter(|(index, _)| !used[*index])
        .map(|(index, contact)| {
            let [x, y] = contact.point();
            (index, (x - point[0]).hypot(y - point[1]))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .expect("an unused certified contact for every native location")
}
#[test]
fn certified_curve_source_contacts_match_every_native_occt_location() {
    let cases = corpus();
    let native = native_locations();
    assert_eq!((cases.len(), native.len()), (76, 76));
    for (case, found) in cases.iter().zip(&native) {
        assert_covers(case, &certify_case(case), found);
    }
}
#[test]
fn certified_curve_source_contacts_match_the_mapped_rust_intersector() {
    for case in corpus() {
        let mapped = InterCurveCurve::between(
            &mapped_target(&case.source),
            &mapped_target(&case.target),
            1e-9,
        )
        .unwrap();
        let mut found: Vec<Location> = (1..=mapped.nb_points())
            .map(|index| {
                let point = mapped.point(index).unwrap();
                Location {
                    point: [point.x(), point.y()],
                    radius: 0.,
                }
            })
            .collect();
        for index in 1..=mapped.nb_segments() {
            let (curve, _) = mapped.segment(index).unwrap();
            let start = curve.value(curve.first_parameter()).unwrap();
            let end = curve.value(curve.last_parameter()).unwrap();
            let text = format!(
                "{:016x},{:016x},{:016x},{:016x}",
                start.x().to_bits(),
                start.y().to_bits(),
                end.x().to_bits(),
                end.y().to_bits()
            );
            found.push(zone(&text));
        }
        assert_covers(&case, &certify_case(&case), &found);
    }
}
