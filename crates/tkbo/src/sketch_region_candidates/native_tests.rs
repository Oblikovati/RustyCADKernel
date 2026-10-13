// SPDX-License-Identifier: GPL-2.0-only
//! Native sampled cusp/complement results are retained diagnostics, not exact decision certificates.
use super::{outcome::*, tests::*};
use rustycad_tkmath::gp::Plane3;
struct NativeCandidateRow<'a> {
    label: &'a str,
    seed: [f64; 2],
    areas: usize,
    max_wires: usize,
    finite_in: usize,
    finite_on: usize,
}
fn native_row(row: &str) -> NativeCandidateRow<'_> {
    let fields: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(
        fields.len(),
        10,
        "expected label seed2 areas wires IN ON OUT finiteIN finiteON"
    );
    NativeCandidateRow {
        label: fields[0],
        seed: [fields[1].parse().unwrap(), fields[2].parse().unwrap()],
        areas: fields[3].parse().unwrap(),
        max_wires: fields[4].parse().unwrap(),
        finite_in: fields[8].parse().unwrap(),
        finite_on: fields[9].parse().unwrap(),
    }
}
fn native_scenario(label: &str) -> CandidateBatch {
    let shapes = match label {
        "cubic" => vec![cubic::closed_analytic_cubic(1., false)],
        "reverse" => vec![cubic::closed_analytic_cubic(1., true)],
        "rational" => vec![rational_closed_curve()],
        "nested" => rectangle(0., 0., 8., 8.)
            .into_iter()
            .chain(rectangle(2., 2., 4., 4.))
            .collect(),
        "touching" => rectangle(0., 0., 2., 2.)
            .into_iter()
            .chain(rectangle(2., 2., 2., 2.))
            .collect(),
        _ => panic!("scenario={label}; expected native fixture label"),
    };
    candidates(&shapes)
}
#[test]
fn native_generation_and_finite_seed_matrix_replays_without_occt() {
    for text in include_str!("../../tests/fixtures/region-candidates-native.txt").lines() {
        let row = native_row(text);
        let batch = native_scenario(row.label);
        assert_eq!(batch.candidates.len(), row.areas);
        assert_eq!(
            batch
                .candidates
                .iter()
                .map(|c| c.wires.len())
                .max()
                .unwrap(),
            row.max_wires
        );
        assert_native_selection(&batch, &row);
    }
}
fn assert_native_selection(batch: &CandidateBatch, row: &NativeCandidateRow<'_>) {
    let selected = selection(batch, row.seed);
    // The actual cusp endpoint has exact Boundary, while sampled native reports bounded IN1.
    if row.seed == [0., 0.] {
        assert_eq!((row.finite_in, row.finite_on), (1, 0));
        assert!(matches!(selected, SeedSelection::Boundary { .. }));
        return;
    }
    if row.finite_on > 0 {
        assert!(matches!(selected, SeedSelection::Boundary { .. }));
        return;
    }
    match row.finite_in {
        0 => assert_eq!(selected, SeedSelection::Empty),
        1 => assert!(matches!(selected, SeedSelection::Unique { .. })),
        count => assert!(
            matches!(selected,SeedSelection::Ambiguous{candidates} if candidates.len()==count)
        ),
    }
}

#[test]
fn native_generated_intersection_history_and_omitted_wire_are_retained() {
    let owned = arranged(&super::coverage_tests::cut_edges());
    let raw = include_str!("../../tests/fixtures/region-candidate-cut-native.txt");
    replay_generated_counts(&owned, raw);
    assert_eq!(
        raw.lines()
            .filter(|line| line.starts_with("cut-wire ") && line.ends_with(" 1 0"))
            .count(),
        2
    );
    let outcome = super::extract_candidates(&owned, Plane3::default(), &mut |_| false);
    assert!(matches!(
        outcome,
        Err(CandidateCause::Incomplete(
            CandidateGap::IncompleteWire { .. }
        ))
    ));
}
fn generated_count(owned: &crate::sketch_region::OwnedArrangement, source: u64) -> usize {
    use rustycad_tkg3d::top_abs::ShapeEnum;
    let mut images: Vec<rustycad_tkbrep::topods::Shape> = Vec::new();
    for relation in &owned.relations {
        if relation.output.shape_type().unwrap() != ShapeEnum::Vertex {
            continue;
        }
        if !relation.generated.iter().any(|key| key.source.0 == source) {
            continue;
        }
        if !images.iter().any(|image| image.is_same(&relation.output)) {
            images.push(relation.output.clone());
        }
    }
    images.len()
}

fn replay_generated_counts(owned: &crate::sketch_region::OwnedArrangement, raw: &str) {
    for row in raw.lines().filter(|row| row.starts_with("generated ")) {
        let fields: Vec<&str> = row.split_whitespace().collect();
        let source = fields[1].parse::<u64>().unwrap();
        let expected = fields[2].parse::<usize>().unwrap();
        assert_eq!(generated_count(owned, source), expected);
    }
}
