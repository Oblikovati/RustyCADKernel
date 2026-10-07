//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDataStd_Expression_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::tdata_std::{TDataStdReal, TDataStdRelation, TDataStdVariable};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

fn new_expression_document() -> Handle<TDocStdDocument> {
    TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap()
}

fn set_variable(label: &TdfLabel, unit: &AsciiString) -> Handle<TDataStdVariable> {
    let variable = TDataStdVariable::set(label).unwrap();
    variable.constant(false).unwrap();
    variable.set_unit(unit).unwrap();
    variable
}

fn set_relation(
    label: &TdfLabel,
    expression: &ExtendedString,
    variable: &Handle<TDataStdVariable>,
) -> Handle<TDataStdRelation> {
    let relation = TDataStdRelation::set(label).unwrap();
    relation.set_relation(expression).unwrap();
    relation.expression().get_variables().push(variable.cast());
    relation
}

// caf/bugs/bug387: undoing a relation change restores the previous expression.
#[test]
fn TDataStd_Expression_Test__CafBug_387_RelationUndo() {
    let document = new_expression_document();
    document.set_undo_limit(10).unwrap();
    let label = document.main().unwrap();

    document.new_command().unwrap();
    let variable = set_variable(&label, &AsciiString::new("a"));
    let relation = set_relation(&label, &ExtendedString::new("a=10"), &variable);
    TDataStdReal::set(&label, 10.0).unwrap();

    document.new_command().unwrap();
    relation.set_relation(&ExtendedString::new("a=20")).unwrap();
    TDataStdReal::set(&label, 20.0).unwrap();

    document.new_command().unwrap();
    assert!(document.undo().unwrap());

    let mut restored_relation: Handle<TDataStdRelation> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdRelation::get_id(), &mut restored_relation)
            .unwrap()
    );
    assert!(
        restored_relation
            .get_relation()
            .is_equal(ExtendedString::new("a=10"))
    );
    let mut restored_real: Handle<TDataStdReal> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdReal::get_id(), &mut restored_real)
            .unwrap()
    );
    assert_eq!(TDataStdReal::get(&restored_real), 10.0);
}

// caf/bugs/bug644: repeated undo/redo of a relation leaves the original
// expression and its variable reference intact.
#[test]
fn TDataStd_Expression_Test__CafBug_644_RelationUndoRedo() {
    let document = new_expression_document();
    document.set_undo_limit(10).unwrap();
    let variable_label = document.main().unwrap();
    let relation_label = document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(2, true)
        .unwrap();

    document.new_command().unwrap();
    let variable = set_variable(&variable_label, &AsciiString::new("mm"));
    let relation = set_relation(&relation_label, &ExtendedString::new("x=10"), &variable);

    document.new_command().unwrap();
    relation.set_relation(&ExtendedString::new("x=20")).unwrap();

    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(document.redo().unwrap());
    assert!(document.undo().unwrap());

    let mut restored_relation: Handle<TDataStdRelation> = Handle::default();
    assert!(
        relation_label
            .find_attribute_as(&TDataStdRelation::get_id(), &mut restored_relation)
            .unwrap()
    );
    assert!(
        restored_relation
            .get_relation()
            .is_equal(ExtendedString::new("x=10"))
    );
    let variables = restored_relation.expression().get_variables().clone();
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0].get().unwrap().label(), variable_label);
}
