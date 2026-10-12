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
    writeln!(
        rows,
        "state-finished {} {}",
        first.tshape().ref_count(),
        second.tshape().ref_count()
    )
    .unwrap();
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
    alias
        .read_current_shape(|shape| {
            writeln!(
                rows,
                "owner-borrow {} {} {}",
                i32::from(same),
                i32::from(shape.is_equal(first)),
                first.tshape().ref_count()
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
    owner.brep_owner_base().shape.write().shape = second.clone();
    alias
        .read_current_shape(|shape| {
            writeln!(
                rows,
                "owner-replaced {} {} {}",
                i32::from(shape.is_equal(second)),
                first.tshape().ref_count(),
                second.tshape().ref_count()
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
    let copy = alias.clone_shape();
    writeln!(
        rows,
        "owner-copy {} {}",
        second.tshape().ref_count(),
        i32::from(copy.is_equal(second))
    )
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
    alias
        .read_current_shape(|shape| {
            writeln!(
                rows,
                "shape-replaced {} {} {} {}",
                i32::from(shape.is_equal(second)),
                first.tshape().ref_count(),
                second.tshape().ref_count(),
                i32::from(object.to_be_updated(true))
            )
            .unwrap();
            Ok(())
        })
        .unwrap();
}
