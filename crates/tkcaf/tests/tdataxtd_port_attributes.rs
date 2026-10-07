//! Port tests of the TDataXtd attributes that the native GTests do not cover: constraint,
//! pattern, presentation and triangulation behaviour, undo, paste and references.
mod support;

use rustycad_tkcaf::tdata_xtd::{
    TDataXtd, TDataXtdAxis, TDataXtdConstraint, TDataXtdConstraintEnum, TDataXtdPattern,
    TDataXtdPatternStd, TDataXtdPlane, TDataXtdPresentation, TDataXtdShape, TDataXtdTriangulation,
};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::{Guid, Handle};
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfDataSet, TdfDerivedAttribute, TdfLabel, TdfRelocationTable,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point3, Transform3};
use rustycad_tkmath::poly::Triangulation;

fn document() -> Handle<TDocStdDocument> {
    let document = support::new_document();
    document.set_undo_limit(10).unwrap();
    document
}

/// A named shape holding an edge on the Z axis through `(x, 0, 0)`.
fn axis_named_shape(label: &TdfLabel, x: f64) -> Handle<TNamingNamedShape> {
    let line = Line3::new(
        Point3::new(x, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    TDataXtdAxis::set_line(label, &line).unwrap();
    support::find_named_shape(label).unwrap()
}

#[test]
fn constraint_geometries_count_until_the_first_null() {
    let document = document();
    let named = axis_named_shape(&support::main_child(&document, 1), 0.0);
    let constraint = TDataXtdConstraint::set(&support::main_child(&document, 2)).unwrap();
    constraint
        .set_geometries(
            TDataXtdConstraintEnum::Parallel,
            &[named.clone(), named.clone()],
        )
        .unwrap();
    assert_eq!(
        (constraint.get_type(), constraint.nb_geometries()),
        (TDataXtdConstraintEnum::Parallel, 2)
    );
    assert!(constraint.get_geometry(5).is_err());
    assert!(
        constraint
            .set_geometries(TDataXtdConstraintEnum::Radius, &[])
            .is_err()
    );
    constraint.clear_geometries().unwrap();
    assert_eq!(constraint.nb_geometries(), 0);
    let mut text = Vec::new();
    constraint.dump(&mut text).unwrap();
    assert_eq!(text, b"Constraint PARALLEL");
}

#[test]
fn constraint_changes_are_undone() {
    let document = document();
    let label = support::main_child(&document, 2);
    let constraint = TDataXtdConstraint::set(&label).unwrap();
    document.new_command().unwrap();
    constraint
        .set_type(TDataXtdConstraintEnum::Distance)
        .unwrap();
    constraint
        .set_value(&TDataStdReal::set(&support::main_child(&document, 3), 5.0).unwrap())
        .unwrap();
    constraint.set_verified(false).unwrap();
    constraint.set_reversed(true).unwrap();
    constraint.set_inverted(true).unwrap();
    document.new_command().unwrap();
    assert!(
        constraint.is_dimension()
            && !constraint.verified()
            && constraint.reversed()
            && constraint.inverted()
    );
    document.undo().unwrap();
    assert_eq!(constraint.get_type(), TDataXtdConstraintEnum::Radius);
    assert!(!constraint.is_dimension() && constraint.verified() && !constraint.reversed());
}

#[test]
fn constraint_references_stop_at_two_geometries_for_placements() {
    let document = document();
    let named = axis_named_shape(&support::main_child(&document, 1), 0.0);
    let constraint = TDataXtdConstraint::set(&support::main_child(&document, 2)).unwrap();
    constraint
        .set_geometries(TDataXtdConstraintEnum::Mate, std::slice::from_ref(&named))
        .unwrap();
    constraint.set_plane(&named).unwrap();
    let data_set = Handle::new(TdfDataSet::new());
    constraint.references(&data_set);
    assert!(data_set.contains_attribute(&named.cast()));
    let mut children = Vec::new();
    TDataXtdConstraint::collect_child_constraints(&document.main().unwrap(), &mut children)
        .unwrap();
    assert_eq!(children.len(), 1);
}

#[test]
fn constraint_paste_relocates_the_geometries() {
    let document = document();
    let (first, second) = (
        axis_named_shape(&support::main_child(&document, 1), 0.0),
        axis_named_shape(&support::main_child(&document, 3), 1.0),
    );
    let source = TDataXtdConstraint::set(&support::main_child(&document, 2)).unwrap();
    source
        .set_geometries(
            TDataXtdConstraintEnum::Tangent,
            std::slice::from_ref(&first),
        )
        .unwrap();
    let table = Handle::new(TdfRelocationTable::new(false));
    table.set_attribute_relocation(&first.cast(), &second.cast());
    let target = TDataXtdConstraint::set(&support::main_child(&document, 4)).unwrap();
    source.paste(&target.cast(), &table).unwrap();
    assert_eq!(target.get_type(), TDataXtdConstraintEnum::Tangent);
    assert!(target.get_geometry(1).unwrap() == second);
}

/// A pattern on `label` with the given signature, axes along Z and X, steps 10 and 20,
/// three and two instances.
fn pattern(document: &Handle<TDocStdDocument>, signature: i32) -> Handle<TDataXtdPatternStd> {
    let pattern = TDataXtdPatternStd::set(&support::main_child(document, 2)).unwrap();
    pattern.set_signature(signature).unwrap();
    pattern
        .set_axis1(&axis_named_shape(&support::main_child(document, 3), 0.0))
        .unwrap();
    pattern
        .set_value1(&TDataStdReal::set(&support::main_child(document, 4), 10.0).unwrap())
        .unwrap();
    pattern
        .set_nb_instances1(&TDataStdInteger::set(&support::main_child(document, 5), 3).unwrap())
        .unwrap();
    pattern
        .set_axis2(&axis_named_shape(&support::main_child(document, 6), 5.0))
        .unwrap();
    pattern
        .set_value2(&TDataStdReal::set(&support::main_child(document, 7), 20.0).unwrap())
        .unwrap();
    pattern
        .set_nb_instances2(&TDataStdInteger::set(&support::main_child(document, 8), 2).unwrap())
        .unwrap();
    pattern
}

fn moved_origin(pattern: &Handle<TDataXtdPatternStd>) -> Vec<Point3> {
    let count = usize::try_from(pattern.nb_trsfs()).unwrap();
    let mut transforms = vec![Transform3::default(); count];
    pattern.compute_trsfs(&mut transforms).unwrap();
    transforms
        .into_iter()
        .map(|t| Point3::new(1.0, 0.0, 0.0).transformed(t))
        .collect()
}

#[test]
fn a_linear_pattern_translates_along_its_axis() {
    let document = document();
    let pattern = pattern(&document, 1);
    let points = moved_origin(&pattern);
    // Natively NbTrsfs multiplies by NbInstances2 whenever it is set, even for a linear
    // pattern; ComputeTrsfs then fills only the first-direction transformations.
    assert_eq!(points.len(), 5);
    assert!(points[2].is_equal(&Point3::new(1.0, 0.0, 0.0), 0.0));
    assert!(points[1].is_equal(&Point3::new(1.0, 0.0, 20.0), 1.0e-12));
    pattern.set_axis1_reversed(true).unwrap();
    assert!(moved_origin(&pattern)[0].is_equal(&Point3::new(1.0, 0.0, -10.0), 1.0e-12));
}

#[test]
fn a_rectangular_pattern_composes_both_directions() {
    let document = document();
    let pattern = pattern(&document, 3);
    let points = moved_origin(&pattern);
    // 3 x 2 instances: two along the first axis, then the second step and its compositions.
    assert_eq!(points.len(), 5);
    assert!(points[2].is_equal(&Point3::new(1.0, 0.0, 20.0), 1.0e-12));
    assert!(points[4].is_equal(&Point3::new(1.0, 0.0, 40.0), 1.0e-12));
    let mut too_small = vec![Transform3::default(); 2];
    assert!(pattern.compute_trsfs(&mut too_small).is_err());
}

#[test]
fn a_mirror_pattern_is_one_symmetry() {
    let document = document();
    let pattern = pattern(&document, 5);
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
    .unwrap();
    TDataXtdPlane::set_plane(&support::main_child(&document, 9), &plane).unwrap();
    pattern
        .set_mirror(&support::find_named_shape(&support::main_child(&document, 9)).unwrap())
        .unwrap();
    let points = moved_origin(&pattern);
    assert!(points.len() == 1 && points[0].is_equal(&Point3::new(-1.0, 0.0, 0.0), 1.0e-12));
    let data_set = Handle::new(TdfDataSet::new());
    pattern.references(&data_set);
    assert!(data_set.contains_attribute(&pattern.mirror().cast()));
    assert_eq!(pattern.pattern_id(), TDataXtdPatternStd::get_pattern_id());
}

#[test]
fn pattern_paste_copies_the_metadata() {
    let document = document();
    let source = pattern(&document, 3);
    let target = Handle::new(TDataXtdPatternStd::new());
    source
        .paste(
            &target.clone().cast(),
            &Handle::new(TdfRelocationTable::new(false)),
        )
        .unwrap();
    assert_eq!(target.signature(), 3);
    assert!(target.axis1().is_null() && target.nb_instances2().is_null());
}

#[test]
fn presentation_settings_are_owned_and_undone() {
    let document = document();
    let label = support::main_child(&document, 2);
    let driver = Guid::default();
    let presentation = TDataXtdPresentation::set(&label, &driver).unwrap();
    document.new_command().unwrap();
    presentation.set_color(NamedColor::RED).unwrap();
    presentation.set_width(2.0).unwrap();
    presentation.set_mode(1).unwrap();
    presentation.set_material_index(3).unwrap();
    presentation.set_transparency(0.5).unwrap();
    presentation.set_displayed(true).unwrap();
    presentation.add_selection_mode(4, true).unwrap();
    presentation.add_selection_mode(7, true).unwrap();
    document.new_command().unwrap();
    assert_eq!(
        (
            presentation.get_nb_selection_modes(),
            presentation.selection_mode(2)
        ),
        (2, 7)
    );
    assert_eq!(presentation.selection_mode(3), 0);
    presentation.set_selection_mode(9, false).unwrap();
    assert_eq!(
        (
            presentation.get_nb_selection_modes(),
            presentation.selection_mode(1)
        ),
        (1, 9)
    );
    document.undo().unwrap();
    assert!(!presentation.has_own_color() && !presentation.is_displayed());
    assert_eq!(presentation.get_nb_selection_modes(), 0);
}

#[test]
fn presentation_unset_and_paste() {
    let document = document();
    let presentation =
        TDataXtdPresentation::set(&support::main_child(&document, 2), &Guid::default()).unwrap();
    presentation.set_color(NamedColor::BLUE1).unwrap();
    presentation.set_width(3.0).unwrap();
    presentation.unset_width().unwrap();
    let target =
        TDataXtdPresentation::set(&support::main_child(&document, 3), &Guid::default()).unwrap();
    target.set_width(7.0).unwrap();
    presentation
        .paste(
            &target.clone().cast(),
            &Handle::new(TdfRelocationTable::new(false)),
        )
        .unwrap();
    assert!(target.has_own_color() && target.color() == NamedColor::BLUE1);
    assert!(!target.has_own_width() && target.width() == 7.0);
    for unset in [
        TDataXtdPresentation::unset_color,
        TDataXtdPresentation::unset_mode,
        TDataXtdPresentation::unset_material,
        TDataXtdPresentation::unset_transparency,
        TDataXtdPresentation::unset_selection_mode,
    ] {
        unset(&target).unwrap();
    }
    assert!(!target.has_own_color());
    TDataXtdPresentation::unset(&support::main_child(&document, 3)).unwrap();
    assert!(
        !support::main_child(&document, 3)
            .is_attribute(&TDataXtdPresentation::get_id())
            .unwrap()
    );
    assert_eq!(
        TDataXtdPresentation::get_old_color_name_from_new_enum(NamedColor::WHITE),
        516
    );
}

#[test]
fn triangulation_changes_are_undone_on_a_copy() {
    let document = document();
    let label = support::main_child(&document, 2);
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    let attribute = TDataXtdTriangulation::set_triangulation_of(&label, &mesh).unwrap();
    document.new_command().unwrap();
    attribute.set_node(1, Point3::new(1.0, 2.0, 3.0)).unwrap();
    attribute.set_deflection(0.25).unwrap();
    document.new_command().unwrap();
    assert_eq!(attribute.deflection().unwrap(), 0.25);
    document.undo().unwrap();
    assert_eq!(attribute.node(1).unwrap(), Point3::default());
    assert!(
        TDataXtdTriangulation::get(&attribute) != mesh,
        "Restore installs a copy"
    );
    assert!(TDataXtdTriangulation::new().nb_nodes().is_err());
}

#[test]
fn the_derived_attributes_are_registered_on_first_use() {
    let _ = TDataXtdShape::new();
    assert!(TdfDerivedAttribute::attribute("TDataXtd_Shape").is_some());
    let mut ids = Vec::new();
    TDataXtd::id_list(&mut ids);
    assert!(ids.contains(&<dyn TDataXtdPattern>::get_id()));
}
