//! The TObj attributes (TObj_TIntSparseArray, TObj_TXYZ, TObj_TNameContainer,
//! TObj_TObject) under transactions, undo and redo, against the semantics of their
//! OCCT 3d097a0 sources; the expected values are derived from the native bodies.
#![allow(non_snake_case)]
use crate::tobj_model::{ModelFixture, ObjectFixture, h_name};
use rustycad_tkernel::standard::Handle;
use rustycad_tklcaf::tdata_std::TDataStdInteger;
use rustycad_tklcaf::tdf::TdfAttribute;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel, dump_attribute};
use rustycad_tkmath::gp::Xyz;
use rustycad_tktobj::tobj::{
    TObjModelState, TObjObject, TObjObjectState, TObjTIntSparseArray, TObjTNameContainer, TObjTXyz,
};

/// A fresh label under the model's main label.
fn new_label(fixture: &ModelFixture) -> TdfLabel {
    fixture.my_model.get_label().new_child().unwrap()
}

/// Two commands on a new sparse array: `1 -> 10, 2 -> 20`, then `1 -> 11`, unset 2,
/// `3 -> 30`; with `touch_label`, the second command also sets a `TDataStd_Integer` on
/// the array's label. Returns the array and its label.
fn two_sparse_commands(
    fixture: &ModelFixture,
    touch_label: bool,
) -> (Handle<TObjTIntSparseArray>, TdfLabel) {
    let model = &fixture.my_model;
    model.get_document().unwrap().set_undo_limit(10).unwrap();
    model.open_command().unwrap();
    let label = new_label(fixture);
    let array = TObjTIntSparseArray::set(&label).unwrap();
    array.set_value(1, 10).unwrap();
    array.set_value(2, 20).unwrap();
    model.commit_command().unwrap();
    model.open_command().unwrap();
    array.set_value(1, 11).unwrap();
    array.unset_value(2).unwrap();
    array.set_value(3, 30).unwrap();
    if touch_label {
        TDataStdInteger::set(&label, 1).unwrap();
    }
    model.commit_command().unwrap();
    assert_eq!(array.get_iterator(), [(1, 11), (3, 30)]);
    (array, label)
}

// SetValue never calls Backup, so a command that only changes the array leaves its label
// unflagged; the optimized TDF_Data::CommitTransaction then skips the label and
// BeforeCommitTransaction never moves the delta into a backup. That command records no
// undo, and the undo takes back the previous command (the array's addition).
#[test]
fn a_sparse_array_alone_records_no_undo_of_its_changes() {
    let fixture = ModelFixture::set_up();
    let (array, label) = two_sparse_commands(&fixture, false);
    let document = fixture.my_model.get_document().unwrap();
    assert_eq!(document.get_available_undos(), 1);
    assert!(document.undo().unwrap());
    assert!(!label.is_attribute(&TObjTIntSparseArray::get_id()).unwrap());
    assert_eq!(array.get_iterator(), [(1, 11), (3, 30)]);
}

// With another attribute backed up on the same label, the commit visits the label:
// BeforeCommitTransaction backs the array up with its delta, and the undo restores the
// old values through DeltaOnModification; the redo applies the new ones again.
#[test]
fn a_sparse_array_undoes_and_redoes_through_its_own_delta() {
    let fixture = ModelFixture::set_up();
    let (array, _label) = two_sparse_commands(&fixture, true);
    let document = fixture.my_model.get_document().unwrap();
    assert_eq!(document.get_available_undos(), 2);
    assert!(document.undo().unwrap());
    assert_eq!(array.get_iterator(), [(1, 10), (2, 20)]);
    assert!(document.redo().unwrap());
    assert_eq!(array.get_iterator(), [(1, 11), (3, 30)]);
}

#[test]
fn a_sparse_array_rejects_bad_values_and_changes_outside_transactions() {
    let fixture = ModelFixture::set_up();
    let model = &fixture.my_model;
    model.open_command().unwrap();
    let array = TObjTIntSparseArray::set(&new_label(&fixture)).unwrap();
    let range = array.set_value(0, 5).unwrap_err();
    assert!(range.is_kind("Standard_OutOfRange"));
    assert!(
        array
            .set_value(4, 0)
            .unwrap_err()
            .is_kind("Standard_OutOfRange")
    );
    assert!(
        array
            .unset_value(0)
            .unwrap_err()
            .is_kind("Standard_OutOfRange")
    );
    array.set_value(4, 8).unwrap();
    model.commit_command().unwrap();
    let document = model.get_document().unwrap();
    document.set_modification_mode(true).unwrap();
    let immutable = array.set_value(4, 9).unwrap_err();
    assert!(immutable.is_kind("Standard_ImmutableObject"));
    document.set_modification_mode(false).unwrap();
    assert!(array.value(4).unwrap() == 8 && array.has_value(4) && !array.has_value(5));
    assert!(array.value(5).unwrap_err().is_kind("Standard_OutOfRange"));
    array.clear();
    assert_eq!(array.size(), 0);
}

#[test]
fn an_xyz_dumps_after_the_attribute_fields() {
    let data = TdfData::new();
    let attribute = TObjTXyz::set(&data.root(), &Xyz::new(1.0, 2.5, -3.0)).unwrap();
    let mut prefix = Vec::new();
    dump_attribute(&*attribute, &mut prefix).unwrap();
    let mut dumped = Vec::new();
    attribute.dump(&mut dumped).unwrap();
    let expected = [prefix, b"X: 1\tY: 2.5\tZ: -3".to_vec()].concat();
    assert_eq!(dumped, expected);
    attribute.set_xyz(&Xyz::new(0.1, 0.0, 1e20)).unwrap();
    assert_eq!(attribute.get_xyz(), Xyz::new(0.1, 0.0, 1e20));
}

#[test]
fn an_xyz_change_is_undone() {
    let fixture = ModelFixture::set_up();
    let model = &fixture.my_model;
    let document = model.get_document().unwrap();
    document.set_undo_limit(10).unwrap();
    model.open_command().unwrap();
    let attribute = TObjTXyz::set(&new_label(&fixture), &Xyz::new(1.0, 2.0, 3.0)).unwrap();
    model.commit_command().unwrap();
    model.open_command().unwrap();
    attribute.set_xyz(&Xyz::new(4.0, 5.0, 6.0)).unwrap();
    model.commit_command().unwrap();
    assert!(document.undo().unwrap());
    assert_eq!(attribute.get_xyz(), Xyz::new(1.0, 2.0, 3.0));
}

#[test]
fn a_name_container_keeps_the_first_label_of_a_name() {
    let fixture = ModelFixture::set_up();
    let model = &fixture.my_model;
    model.open_command().unwrap();
    let container = TObjTNameContainer::set(&new_label(&fixture)).unwrap();
    let (first, second) = (new_label(&fixture), new_label(&fixture));
    container.record_name(&h_name("n"), &first).unwrap();
    container.record_name(&h_name("n"), &second).unwrap();
    assert!(container.find_label(&h_name("n")).unwrap() == first);
    assert!(container.is_registered(&h_name("n")));
    assert_eq!(container.get_map().extent(), 1);
    container.remove_name(&h_name("n")).unwrap();
    assert!(!container.is_registered(&h_name("n")));
    assert!(
        container
            .find_label(&h_name("n"))
            .unwrap_err()
            .is_kind("Standard_NoSuchObject")
    );
    container.record_name(&h_name("m"), &second).unwrap();
    let copy = TObjTNameContainer::set(&new_label(&fixture)).unwrap();
    copy.set_map(&container.get_map()).unwrap();
    container.clear().unwrap();
    assert!(!container.is_registered(&h_name("m")) && copy.is_registered(&h_name("m")));
    model.commit_command().unwrap();
}

#[test]
fn an_undone_object_dies_and_rises_again_on_redo() {
    let fixture = ObjectFixture::set_up();
    let document = fixture.my_model().get_document().unwrap();
    document.set_undo_limit(10).unwrap();
    let object = fixture.command(|| fixture.create_object());
    assert!(object.is_alive().unwrap());
    assert!(document.undo().unwrap());
    assert!(!object.is_alive().unwrap());
    assert!(object.get_label().is_null());
    assert!(document.redo().unwrap());
    assert!(object.is_alive().unwrap());
    assert!(fixture.my_partition.get_child_label().unwrap().has_child());
}
