// SPDX-License-Identifier: GPL-2.0-only
//! Two authored same-kind intermediates with identical geometry/parents stay distinct.
use super::{tests::fixture, *};
use crate::sketch_region::{ArrangementRelation, PartitionOccurrenceIdentity};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

#[test]
fn equal_kind_coordinates_and_parent_sets_do_not_collapse_authored_intermediates() {
    let (_, receipt, mut publication, mut authored) = fixture();
    let parent = receipt.copied.as_ref().unwrap().observations[0]
        .selected
        .clone();
    let first = copied_edge(&parent);
    let second = copied_edge(&parent);
    bind_pair(&mut publication, &mut authored, &first, 3101, 4101);
    bind_pair(&mut publication, &mut authored, &second, 3102, 4102);
    let inputs = LineageInputs {
        receipt: &receipt,
        publication: &publication,
        authored: &authored,
    };
    let left = owned_pair(&inputs, &first);
    let right = owned_pair(&inputs, &second);
    assert_eq!(left.selected_parents, right.selected_parents);
    assert_eq!(left.selected.member.kind, right.selected.member.kind);
    assert_eq!(left.selected.placement_bits, right.selected.placement_bits);
    assert_ne!(left.selected.member, right.selected.member);
    assert_ne!(left.copied.member, right.copied.member);
}
fn copied_edge(parents: &ArrangementRelation) -> PrismCopyObservation {
    let mut selected = parents.clone();
    selected.output = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut copier = BRepBuilderAPICopy::with_shape(&selected.output, true, true).unwrap();
    let images = copier.modified(&selected.output).unwrap();
    assert_eq!(images.len(), 1);
    let native_image = images.first().unwrap().clone();
    let copied_image = native_image.oriented(selected.output.orientation());
    PrismCopyObservation {
        selected,
        native_image,
        copied_image,
    }
}
fn bind_pair(
    publication: &mut PrismPublicationBindings,
    authored: &mut PrismCopyBindings,
    pair: &PrismCopyObservation,
    selected: u64,
    copied: u64,
) {
    authored.selected_members.push(PrismSourceBinding {
        actual: PartitionOccurrenceIdentity::capture(&pair.selected.output),
        member: AuthoredPrismId(selected),
    });
    publication.sources.push(PrismSourceBinding {
        actual: PartitionOccurrenceIdentity::capture(&pair.copied_image),
        member: AuthoredPrismId(copied),
    });
}
fn owned_pair(inputs: &LineageInputs<'_>, pair: &PrismCopyObservation) -> CopyMemberObservation {
    let relation =
        crate::sketch_region_candidates::composed_copy_relation(&pair.selected, &pair.copied_image);
    members::owned_member_record(inputs, pair, &relation).unwrap()
}
