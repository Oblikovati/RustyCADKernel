// SPDX-License-Identifier: GPL-2.0-only
//! Full occurrence identity and relation kind survive independently of root-ID convenience.
use super::outcome::attributed_occurrence;
use super::source_relation_test_support::{actual_vertex, member_identity};
use super::source_relations::{SourceMemberRelations, SourceRelation, source_relations};
use crate::sketch_region::ArrangementRelation;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::{gp::Transform3, toploc::Location};

fn located_occurrence(original: &Shape) -> Shape {
    let placement = Location::new(Transform3::translation([9.0, 0.0, 0.0]));
    original.located(&placement, false).unwrap()
}

fn three_kinds(original: &Shape) -> ArrangementRelation {
    ArrangementRelation {
        output: actual_vertex(),
        preserved: vec![member_identity(7, original)],
        modified: vec![member_identity(7, &original.reversed())],
        generated: vec![member_identity(11, &located_occurrence(original))],
    }
}

#[test]
fn adapter_retains_member_identity_and_each_relation_kind() {
    let original = actual_vertex();
    let relation = three_kinds(&original);
    let copied = source_relations(std::slice::from_ref(&relation));
    let actual = attributed_occurrence(&relation.output, &copied);
    let parents = actual.original_parents.unwrap();
    assert_eq!(parents.preserved, relation.preserved);
    assert_eq!(parents.modified, relation.modified);
    assert_eq!(parents.generated, relation.generated);
    assert_eq!(actual.source_parents, Some(vec![7, 11]));
    assert!(!actual.shape.is_same(&original));
}

fn repeated_member_relation(original: &Shape, output: &Shape) -> SourceRelation {
    let forward = member_identity(7, original);
    SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations {
            modified: vec![
                forward.clone(),
                member_identity(7, &original.reversed()),
                member_identity(7, &located_occurrence(original)),
                forward.clone(),
            ],
            generated: vec![forward],
            ..SourceMemberRelations::default()
        },
    }
}

#[test]
fn union_deduplicates_only_full_member_identity_inside_each_kind() {
    let original = actual_vertex();
    let output = actual_vertex();
    let first = repeated_member_relation(&original, &output);
    let second = SourceRelation {
        output: output.clone(),
        parents: first.parents.clone(),
    };
    let parents = attributed_occurrence(&output, &[first, second])
        .original_parents
        .unwrap();
    assert_eq!(parents.modified.len(), 3);
    assert_eq!(parents.generated.len(), 1);
    assert_eq!(parents.source_ids(), vec![7]);
    assert_eq!(parents.modified[0].orientation, Orientation::Forward);
    assert_eq!(parents.modified[1].orientation, Orientation::Reversed);
    assert_ne!(parents.modified[0].location, parents.modified[2].location);
}

#[test]
fn output_lookup_requires_exact_orientation_and_location() {
    let output = actual_vertex();
    let reversed = output.reversed();
    let positioned = located_occurrence(&output);
    let relations = [
        test_row(&output, 5),
        test_row(&reversed, 7),
        test_row(&positioned, 11),
    ];
    assert_output_source(&output, &relations, 5);
    assert_output_source(&reversed, &relations, 7);
    assert_output_source(&positioned, &relations, 11);
    let missing = output.oriented(Orientation::Internal);
    assert!(
        attributed_occurrence(&missing, &relations)
            .original_parents
            .is_none()
    );
}

fn test_row(output: &Shape, source: u64) -> SourceRelation {
    SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations {
            preserved: vec![member_identity(source, output)],
            ..SourceMemberRelations::default()
        },
    }
}

#[test]
fn empty_or_absent_attribution_stays_unknown() {
    let output = actual_vertex();
    let empty = SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations::default(),
    };
    let occurrence = attributed_occurrence(&output, &[empty]);
    assert!(occurrence.original_parents.is_none());
    assert!(occurrence.source_parents.is_none());
    assert!(
        attributed_occurrence(&output, &[])
            .original_parents
            .is_none()
    );
}

fn assert_output_source(output: &Shape, relations: &[SourceRelation], expected: u64) {
    let occurrence = attributed_occurrence(output, relations);
    assert_eq!(occurrence.source_parents, Some(vec![expected]));
}

#[test]
fn any_matching_unknown_row_prevents_a_complete_parent_claim() {
    let output = actual_vertex();
    let known = test_row(&output, 7);
    let unknown = SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations::default(),
    };
    let actual = attributed_occurrence(&output, &[known, unknown]);
    assert!(actual.original_parents.is_none());
    assert!(actual.source_parents.is_none());
}

#[test]
fn equal_numeric_placements_keep_distinct_location_identity() {
    let original = actual_vertex();
    let first = located_occurrence(&original);
    let second = located_occurrence(&original);
    assert_distinct_location_keys(&first, &second);
    let output = actual_vertex();
    let row = SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations {
            modified: vec![member_identity(7, &first), member_identity(7, &second)],
            ..SourceMemberRelations::default()
        },
    };
    let parents = attributed_occurrence(&output, &[row])
        .original_parents
        .unwrap();
    assert_eq!(parents.modified.len(), 2);
}

fn assert_distinct_location_keys(first: &Shape, second: &Shape) {
    let first_placement = first.location().transformation().translation_part();
    let second_placement = second.location().transformation().translation_part();
    assert_eq!(first_placement, second_placement);
    assert_ne!(first.location(), second.location());
}
