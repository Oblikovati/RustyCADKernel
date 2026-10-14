// SPDX-License-Identifier: GPL-2.0-only
//! Pinned native actual copied pcurves bind the census coefficients; they do not certify topology or roots.
use super::paired_tests::*;
use super::tests::capture;
use super::*;
use rustycad_tkbrep::topods::Shape;
const COPIED_SOURCES: &str = include_str!("../../tests/fixtures/paired-census-sources-native.txt");
#[test]
fn native_aggregate_copy_replays_actual_shared_located_and_rational_source_descriptors() {
    for line in COPIED_SOURCES.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let sources = fixture_sources(fields[0]);
        let census = paired(&sources);
        let occurrence = fields[1].parse::<usize>().unwrap();
        assert_native_descriptor(&census, occurrence, &fields);
    }
}
fn fixture_sources(name: &str) -> Vec<Shape> {
    match name {
        "pair" => curved_pair(),
        "crossing" => vec![crossing_fixture()],
        "cusp" => vec![cusp_fixture()],
        "closed" => vec![crate::closed_analytic_cubic::closed_analytic_cubic(
            1., false,
        )],
        "rational" => vec![
            bezier_shape(&[[0., 0.], [0.5, 1.]], Some(&[1., 2.])),
            bezier_shape(&[[0., 1.], [1., 0.]], None),
        ],
        "aliases" => shared_sources(),
        actual => panic!("native source case={actual}; expected six attributed cases"),
    }
}
fn shared_sources() -> Vec<Shape> {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let first = curved_pair().remove(0);
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(10., 0., 0.));
    let located = first.located(&Location::new(translation), true).unwrap();
    vec![first.clone(), first.reversed(), located]
}
fn assert_native_descriptor(census: &BoundaryCensus, occurrence: usize, fields: &[&str]) {
    let record = &census.paired.as_ref().unwrap().occurrences[occurrence];
    let source = &record.source;
    assert_eq!(record.physical_source, fields[2].parse::<usize>().unwrap());
    assert_eq!(source.reversed as u8, fields[3].parse::<u8>().unwrap());
    assert_eq!(
        source.original_interval,
        [fields[4].parse().unwrap(), fields[5].parse().unwrap()]
    );
    assert_eq!(source.weights.is_some(), fields[6] == "1");
    assert_eq!(source.controls.len(), fields[7].parse::<usize>().unwrap());
    assert!(source.parents.iter().all(|parent| {
        record
            .relations
            .iter()
            .any(|row| row.modified.contains(parent))
    }));
    assert_native_poles(source, &fields[8..]);
}
fn assert_native_poles(
    source: &crate::sketch_region_candidates::CandidateSegment,
    fields: &[&str],
) {
    assert_eq!(source.controls.len(), fields.len());
    for (index, field) in fields.iter().enumerate() {
        let values = field
            .split(',')
            .map(|scalar| scalar.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 3);
        assert_eq!(source.controls[index], [values[0], values[1]]);
        assert_eq!(
            source.weights.as_ref().map_or(1., |weights| weights[index]),
            values[2]
        );
    }
}
#[test]
fn native_fixture_source_capture_and_original_caller_geometry_remain_distinct() {
    let originals = shared_sources();
    let inputs = capture(&originals);
    assert!(!originals[0].is_partner(&inputs.segments[0].edge));
    assert!(inputs.segments[0].edge.is_partner(&inputs.segments[1].edge));
    assert_eq!(inputs.segments[2].controls[0], [10., 0.]);
}

fn crossing_fixture() -> Shape {
    let poles = [[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]];
    bezier_shape(&poles, None)
}
fn cusp_fixture() -> Shape {
    let poles = [[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]];
    bezier_shape(&poles, None)
}
