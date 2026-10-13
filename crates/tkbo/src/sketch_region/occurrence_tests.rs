// SPDX-License-Identifier: GPL-2.0-only
//! One authored compound contains distinct actual directions, even though physical history keys coincide.
use super::{
    copy_inputs::*,
    occurrences::equal_occurrences,
    outcome::*,
    tests::{closed_cubic, sources},
};
use rustycad_tkbrep::topods::borrowed_shape_batch;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

#[test]
fn same_source_compound_keeps_forward_and_reversed_original_members() {
    let original = closed_cubic();
    let reversed = original.oriented(Orientation::Reversed);
    let compound = borrowed_shape_batch(&[&original, &reversed]).unwrap();
    let copied =
        copy_sources(&sources(&compound)).unwrap_or_else(|_| panic!("expected sharedcopy"));
    let keys = edge_member_keys(&copied);
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].source, keys[1].source);
    assert_eq!(keys[0].topology_address, keys[1].topology_address);
    assert_eq!(
        [keys[0].orientation, keys[1].orientation],
        [Orientation::Forward, Orientation::Reversed]
    );
    assert!(original.free().unwrap());
}
#[test]
fn output_occurrence_collection_retains_actual_equal_directions_without_synthesis() {
    let original = closed_cubic();
    let reversed = original.oriented(Orientation::Reversed);
    let compound = borrowed_shape_batch(&[&original, &reversed]).unwrap();
    let actual = equal_occurrences(&compound).unwrap();
    let edges: Vec<_> = actual
        .iter()
        .filter(|shape| shape.shape_type().unwrap() == ShapeEnum::Edge)
        .collect();
    assert_eq!(edges.len(), 2);
    assert!(edges[0].is_equal(&original) && edges[1].is_equal(&reversed));
    assert!(edges[0].is_same(edges[1]) && !edges[0].is_equal(edges[1]));
}
#[test]
fn shared_compound_history_gap_is_explicit_after_original_occurrence_capture() {
    let original = closed_cubic();
    let reversed = original.oriented(Orientation::Reversed);
    let compound = borrowed_shape_batch(&[&original, &reversed]).unwrap();
    let result = super::arrangement::arrange_owned_sources(&sources(&compound), &mut |_| false);
    assert!(matches!(
        result,
        ArrangementCompletion::Incomplete(ArrangementCause::UnsupportedHistoryCoverage { .. })
    ));
}

fn edge_member_keys(copied: &CopiedSources) -> Vec<&OriginalMemberIdentity> {
    copied
        .members
        .iter()
        .filter(|(_, shape)| shape.shape_type().unwrap() == ShapeEnum::Edge)
        .map(|(key, _)| key)
        .collect()
}

#[test]
fn cyclic_input_is_rejected_before_recursive_copy_without_new_input_mutation() {
    use rustycad_tkbrep::topods::{Builder, Compound};
    let mut root = Compound::default();
    Builder.make_compound(&mut root);
    let child = borrowed_shape_batch(&[&root]).unwrap();
    Builder.add(&mut root, &child).unwrap();
    let result = copy_sources(&sources(&root));
    assert!(
        matches!(result,Err(ArrangementCause::Failure{phase:ArrangementPhase::Copy,cause}) if cause.exception_type()=="Standard_ConstructionError")
    );
    // The deliberately malformed fixture owns a cycle; repair it before releasing either node.
    root.set_free(true).unwrap();
    Builder.remove(&mut root, &child).unwrap();
    assert_eq!(root.nb_children(), 0);
}
