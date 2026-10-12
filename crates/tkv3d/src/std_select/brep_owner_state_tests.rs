//! Native owner and presentation field references retain identity through valid protected/member replacements.
use super::*;
use crate::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers, prs3d::Prs3dDrawer, select_mgr::*};
use rustycad_tkbrep::topods::{Builder, Compound, Shape, TopoDSShapeReadSource};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, Transient};
use std::fmt::Write;
#[test]
fn actual_native_brep_owner_state_and_reference_contracts_replay() {
    let mut first = Compound::default();
    let mut second = Compound::default();
    Builder.make_compound(&mut first);
    Builder.make_compound(&mut second);
    let mut rows = String::new();
    record_owner_mode_states(&mut rows);
    record_owner_origin(&first, &second, &mut rows);
    record_owner_field_alias(&first, &second, &mut rows);
    record_presentation_field_alias(&first, &second, &mut rows);
    let counts = (first.tshape().ref_count(), second.tshape().ref_count());
    writeln!(rows, "state-finished {} {}", counts.0, counts.1).unwrap();
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-owner-state.txt")
    );
}
fn record_owner_mode_states(rows: &mut String) {
    let owner = StdSelectBRepOwner::new(-7);
    record_owner_state("priority", &owner, rows);
    owner.set_hilight_mode(-2);
    record_owner_state("mode-negative", &owner, rows);
    owner.reset_hilight_mode();
    record_owner_state("mode-reset", &owner, rows);
    owner.set_hilight_mode(17);
    record_owner_state("mode-positive", &owner, rows);
}
fn record_owner_state(label: &str, owner: &StdSelectBRepOwner, rows: &mut String) {
    writeln!(
        rows,
        "{label} {} {} {} {} {} {} {} {}",
        i32::from(owner.has_shape()),
        owner.priority(),
        i32::from(owner.comes_from_decomposition()),
        owner.hilight_mode(),
        i32::from(owner.has_hilight_mode()),
        i32::from(owner.has_selectable()),
        i32::from(owner.is_kind(StdSelectBRepOwner::static_type())),
        i32::from(owner.is_kind(SelectMgrEntityOwner::static_type()))
    )
    .unwrap();
}
fn record_owner_origin(first: &Shape, second: &Shape, rows: &mut String) {
    let object = AisColoredShape::new(first);
    let owner = StdSelectBRepOwner::with_selectable(second, &object.cast(), 8, true);
    record_owner_state("origin", &owner, rows);
    drop(owner);
    writeln!(
        rows,
        "origin-released {} {} {}",
        first.tshape().ref_count(),
        second.tshape().ref_count(),
        object.ref_count()
    )
    .unwrap();
    object.set_shape(&Shape::default());
}
fn record_owner_field_alias(first: &Shape, second: &Shape, rows: &mut String) {
    let owner = StdSelectBRepOwner::with_shape(first, 7, true);
    let alias = owner.shape_ref();
    let other = owner.shape_ref();
    let same = alias.is_same_reference(&other);
    record_owner_borrow(&alias, first, same, rows);
    owner.brep_owner_base().shape.write().shape = second.clone();
    record_owner_replacement(&alias, first, second, rows);
    let copy = alias.clone_shape();
    let same = i32::from(copy.is_equal(second));
    writeln!(rows, "owner-copy {} {same}", second.tshape().ref_count()).unwrap();
}
fn record_owner_borrow(
    alias: &StdSelectShapeRef<'_>,
    first: &Shape,
    same: bool,
    rows: &mut String,
) {
    alias
        .read_current_shape(|shape| {
            let same = i32::from(same);
            let equal = i32::from(shape.is_equal(first));
            writeln!(
                rows,
                "owner-borrow {same} {equal} {}",
                first.tshape().ref_count()
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
}
fn record_owner_replacement(
    alias: &StdSelectShapeRef<'_>,
    first: &Shape,
    second: &Shape,
    rows: &mut String,
) {
    alias
        .read_current_shape(|shape| {
            let same = i32::from(shape.is_equal(second));
            let counts = (first.tshape().ref_count(), second.tshape().ref_count());
            writeln!(rows, "owner-replaced {same} {} {}", counts.0, counts.1).unwrap();
            Ok(())
        })
        .unwrap();
}

fn record_presentation_field_alias(first: &Shape, second: &Shape, rows: &mut String) {
    let drawer = Handle::new(Prs3dDrawer::new());
    let object = StdSelectShape::new(first, &drawer);
    let alias = object.shape_ref();
    let linked = object.attributes().link().object_address() == drawer.object_address();
    writeln!(
        rows,
        "shape-borrow {} {} {} {}",
        first.tshape().ref_count(),
        i32::from(linked),
        drawer.ref_count(),
        i32::from(object.is_kind(<dyn crate::prs_mgr::PrsMgrPresentableObject>::static_type()))
    )
    .unwrap();
    object.set_shape(second);
    record_presentation_replacement(&object, &alias, first, second, rows);
}
fn record_presentation_replacement(
    object: &StdSelectShape,
    alias: &StdSelectShapeRef<'_>,
    first: &Shape,
    second: &Shape,
    rows: &mut String,
) {
    alias
        .read_current_shape(|shape| {
            let same = i32::from(shape.is_equal(second));
            let counts = (first.tshape().ref_count(), second.tshape().ref_count());
            let updated = i32::from(object.to_be_updated(true));
            writeln!(
                rows,
                "shape-replaced {same} {} {} {updated}",
                counts.0, counts.1
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
}
