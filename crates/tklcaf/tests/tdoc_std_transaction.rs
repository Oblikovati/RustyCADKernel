//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDocStd_Transaction_Test.cxx.
//!
//! The cases on TDataXtd attributes, TNaming named shapes and BRepPrimAPI boxes
//! (CafBug_2932_3, _5, _8, _12, _13, _14, _18 and _24) need TKCAF and TKPrim; the ledger
//! marks them `pending_dependency`.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::{
    TDataStdComment, TDataStdCurrent, TDataStdExpression, TDataStdIntPackedMap, TDataStdInteger,
    TDataStdIntegerArray, TDataStdName, TDataStdReal, TDataStdRealArray, TDataStdRelation,
    TDataStdTreeNode, TDataStdVariable,
};
use rustycad_tklcaf::tdf::{TdfLabel, TdfReference, TdfTagSource};
use rustycad_tklcaf::tdoc_std::{TDocStdDocument, TDocStdModified, TDocStdMultiTransactionManager};
use rustycad_tklcaf::tfunction::TFunctionFunction;

fn new_transaction_document() -> Handle<TDocStdDocument> {
    TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap()
}

fn root_child(document: &Handle<TDocStdDocument>, tag: i32) -> TdfLabel {
    document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(tag, true)
        .unwrap()
}

fn has_modified_labels(document: &Handle<TDocStdDocument>) -> bool {
    let mut modified: Handle<TDocStdModified> = Handle::default();
    let root = document.main().unwrap().root().unwrap();
    if !root
        .find_attribute_as(&TDocStdModified::get_id(), &mut modified)
        .unwrap()
    {
        return false;
    }
    !modified.is_empty()
}

fn real_value(label: &TdfLabel) -> f64 {
    let mut real: Handle<TDataStdReal> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdReal::get_id(), &mut real)
            .unwrap(),
        "Real attribute is missing"
    );
    TDataStdReal::get(&real)
}

fn expect_last_delta_is(document: &Handle<TDocStdDocument>, attribute_name: &str) {
    let undos = document.get_undos();
    let delta = undos.last().expect("an undo");
    let deltas = delta.attribute_deltas();
    let last = deltas.last().expect("an attribute delta");
    assert_eq!(
        last.attribute().dynamic_type().unwrap().name(),
        attribute_name
    );
}

fn expect_last_delta_is_real(document: &Handle<TDocStdDocument>) {
    expect_last_delta_is(document, "TDataStd_Real");
}

/// The common shape of the bug2932 cases: `set` twice with the same values around a real
/// value change, then the last delta must be the real's.
fn unchanged_keeps_real_delta(set: impl Fn(&TdfLabel)) {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    let label = root_child(&document, 2);
    document.new_command().unwrap();
    set(&label);
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    set(&label);
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();
    expect_last_delta_is_real(&document);
}

// caf/bugs/buc60790: the most recent redo keeps the name of the second undone delta.
#[test]
fn TDocStd_Transaction_Test__Buc_CafBug_60790_TreeNodeDeltaNames() {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    document.new_command().unwrap();
    assert!(
        !TDataStdTreeNode::set(&root_child(&document, 2))
            .unwrap()
            .is_null()
    );
    document.new_command().unwrap();
    assert!(
        !TDataStdTreeNode::set(&root_child(&document, 3))
            .unwrap()
            .is_null()
    );
    document.new_command().unwrap();
    assert!(
        !TDataStdTreeNode::set(&root_child(&document, 4))
            .unwrap()
            .is_null()
    );
    document.new_command().unwrap();

    assert_eq!(document.get_available_undos(), 3);
    let undos = document.get_undos();
    assert_eq!(undos.len(), 3);
    for (delta, name) in undos.iter().zip(["n1", "n2", "n3"]) {
        delta.set_name(&ExtendedString::new(name));
    }
    assert!(document.undo().unwrap());
    assert!(document.undo().unwrap());
    assert_eq!(document.get_available_undos(), 1);
    assert_eq!(document.get_available_redos(), 2);
    let redos = document.get_redos();
    assert!(
        redos
            .first()
            .unwrap()
            .name()
            .is_equal(ExtendedString::new("n2"))
    );
}

// caf/bugs/buc60831: modified labels are restored when the command adding them is undone.
#[test]
fn TDocStd_Transaction_Test__Buc_CafBug_60831_ModifiedAttributeUndo() {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    document.new_command().unwrap();
    let label = root_child(&document, 3);
    assert!(!has_modified_labels(&document));
    document.new_command().unwrap();
    assert!(TDocStdModified::add(&label).unwrap());
    assert!(has_modified_labels(&document));
    document.new_command().unwrap();
    assert!(document.undo().unwrap());
    assert!(!has_modified_labels(&document));
}

// caf/bugs/bug1722: a multi-document transaction manager commits and restores the
// second value while preserving the value set before the managed transaction.
#[test]
fn TDocStd_Transaction_Test__CafBug_1722_MultiTransactionManagerUndoRedo() {
    let document = new_transaction_document();
    let manager = Handle::new(TDocStdMultiTransactionManager::new());
    document.set_undo_limit(4).unwrap();
    manager.set_undo_limit(4).unwrap();
    manager.add_document(&document).unwrap();

    let label = document.main().unwrap().find_child(1, true).unwrap();
    TDataStdReal::set(&label, 1.1).unwrap();
    assert_eq!(real_value(&label), 1.1);

    manager.open_command().unwrap();
    TDataStdReal::set(&label, 2.1).unwrap();
    assert_eq!(real_value(&label), 2.1);
    assert!(manager.get_available_undos().is_empty());

    assert!(
        manager
            .commit_command_named(&ExtendedString::new("Transaction1"))
            .unwrap()
    );
    let undos = manager.get_available_undos();
    assert_eq!(undos.len(), 1);
    assert!(
        undos[0]
            .get_name()
            .is_equal(ExtendedString::new("Transaction1"))
    );

    manager.undo().unwrap();
    assert_eq!(real_value(&label), 1.1);
    assert_eq!(manager.get_available_redos().len(), 1);

    manager.redo().unwrap();
    assert_eq!(real_value(&label), 2.1);
    assert!(manager.get_available_redos().is_empty());
}

// caf/bugs/bug25112: undoing a packed-map modification restores both removal
// and addition performed by one command.
#[test]
fn TDocStd_Transaction_Test__CafBug_25112_IntPackedMapUndo() {
    let document = new_transaction_document();
    document.set_undo_limit(10).unwrap();
    let label = document.main().unwrap();
    let map = TDataStdIntPackedMap::set(&label, false).unwrap();
    assert!(!map.is_null());
    assert!(map.add(319).unwrap());

    document.open_command().unwrap();
    assert!(map.add(344).unwrap());
    assert!(map.remove(319).unwrap());
    assert!(document.commit_command().unwrap());

    assert!(document.undo().unwrap());
    let mut restored: Handle<TDataStdIntPackedMap> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdIntPackedMap::get_id(), &mut restored)
            .unwrap()
    );
    assert!(restored.contains(319));
    assert!(!restored.contains(344));
}

// caf/bugs/bug29142: undo and redo of a forget-plus-replace command keep the
// integer attribute valid rather than leaving a missing attribute behind.
#[test]
fn TDocStd_Transaction_Test__CafBug_29142_ForgetAndReplaceIntegerUndoRedo() {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    let label = document.main().unwrap().find_child(1, true).unwrap();
    document.new_command().unwrap();
    TDataStdInteger::set(&label, 111).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 0.234).unwrap();
    document.new_command().unwrap();
    assert!(
        label
            .forget_attribute_id(&TDataStdInteger::get_id())
            .unwrap()
    );
    TDataStdInteger::set(&label, 222).unwrap();
    document.new_command().unwrap();

    assert!(document.undo().unwrap());
    let mut integer: Handle<TDataStdInteger> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdInteger::get_id(), &mut integer)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&integer), 111);
    let mut real: Handle<TDataStdReal> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdReal::get_id(), &mut real)
            .unwrap()
    );
    assert_eq!(TDataStdReal::get(&real), 0.234);

    assert!(document.redo().unwrap());
    assert!(
        label
            .find_attribute_as(&TDataStdInteger::get_id(), &mut integer)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&integer), 222);
}

// caf/bugs/bug31920: enabling entry indexing does not change the value
// retrieved from a label.
#[test]
fn TDocStd_Transaction_Test__CafBug_31920_AccessLabelByEntry() {
    let document = new_transaction_document();
    let label = document
        .main()
        .unwrap()
        .root()
        .unwrap()
        .find_child(2, true)
        .unwrap();
    TDataStdInteger::set(&label, 5).unwrap();
    let mut value: Handle<TDataStdInteger> = Handle::default();
    assert!(
        label
            .find_attribute_as(&TDataStdInteger::get_id(), &mut value)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&value), 5);

    let data = document.get_data();
    data.set_access_by_entries(true).unwrap();
    let mut found = TdfLabel::default();
    assert!(data.get_label(&AsciiString::new("0:2"), &mut found));
    assert_eq!(found, label);
    assert!(
        found
            .find_attribute_as(&TDataStdInteger::get_id(), &mut value)
            .unwrap()
    );
    assert_eq!(TDataStdInteger::get(&value), 5);
}

// caf/bugs/bug2932_9: repeating an unchanged integer attribute must not hide
// the real attribute modification in the committed transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_9_UnchangedIntegerKeepsRealDelta() {
    unchanged_keeps_real_delta(|label| {
        TDataStdInteger::set(label, 100).unwrap();
    });
}

// caf/bugs/bug2932_11: an unchanged name attribute must not become the
// reported modified attribute when a real value changes.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_11_UnchangedNameKeepsRealDelta() {
    unchanged_keeps_real_delta(|label| {
        TDataStdName::set(label, &ExtendedString::new("New Name")).unwrap();
    });
}

/// `anArray = TDataStd_IntegerArray::Set(L, 1, 2, false); SetValue(1, 3); SetValue(2, 4)`.
fn set_integer_array(label: &TdfLabel) {
    let array = TDataStdIntegerArray::set(label, 1, 2, false).unwrap();
    array.set_value(1, 3).unwrap();
    array.set_value(2, 4).unwrap();
}

/// `anArray = TDataStd_RealArray::Set(L, 1, 2, false); SetValue(1, 3.); SetValue(2, 4.)`.
fn set_real_array(label: &TdfLabel) {
    let array = TDataStdRealArray::set(label, 1, 2, false).unwrap();
    array.set_value(1, 3.0).unwrap();
    array.set_value(2, 4.0).unwrap();
}

// caf/bugs/bug2932_1: unchanged integer-array values must not replace the
// real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_1_UnchangedIntegerArrayKeepsRealDelta() {
    unchanged_keeps_real_delta(set_integer_array);
}

// caf/bugs/bug2932_16: the same delta rule applies to real arrays.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_16_UnchangedRealArrayKeepsRealDelta() {
    unchanged_keeps_real_delta(set_real_array);
}

// bugs/caf/bug261: clearing redo history after two undo operations leaves no
// redo command available while preserving the remaining undo history.
#[test]
fn TDocStd_Transaction_Test__CafBug_261_ClearRedosAfterUndo() {
    let document = new_transaction_document();
    document.set_undo_limit(10).unwrap();
    let label = document.main().unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 10.0).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 20.0).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 30.0).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 40.0).unwrap();

    assert!(document.undo().unwrap());
    assert!(document.undo().unwrap());
    assert_eq!(document.get_available_undos(), 1);
    assert_eq!(document.get_available_redos(), 2);

    document.clear_redos();
    assert_eq!(document.get_available_undos(), 1);
    assert_eq!(document.get_available_redos(), 0);
}

// bugs/caf/bug2932_2: repeating an unchanged real-array attribute must not
// replace the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_2_UnchangedRealArrayKeepsRealDelta() {
    unchanged_keeps_real_delta(set_real_array);
}

// bugs/caf/bug2932_4: repeating an unchanged comment attribute must not
// replace the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_4_UnchangedCommentKeepsRealDelta() {
    let comment = ExtendedString::new("New Comment");
    unchanged_keeps_real_delta(|label| {
        TDataStdComment::set(label, &comment).unwrap();
    });
}

// bugs/caf/bug2932_10: repeating an unchanged integer-array attribute must
// not replace the real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_10_UnchangedIntegerArrayKeepsRealDelta() {
    unchanged_keeps_real_delta(set_integer_array);
}

// bugs/caf/bug2932_6: an unchanged current attribute must not replace the
// real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_6_UnchangedCurrentKeepsRealDelta() {
    unchanged_keeps_real_delta(|label| {
        TDataStdCurrent::set(label).unwrap();
    });
}

// bugs/caf/bug2932_7: an unchanged expression must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_7_UnchangedExpressionKeepsRealDelta() {
    let expression = ExtendedString::new("New Expression");
    unchanged_keeps_real_delta(|label| {
        TDataStdExpression::set(label)
            .unwrap()
            .set_expression(&expression)
            .unwrap();
    });
}

// bugs/caf/bug2932_15: changing an integer while repeating an unchanged real
// must report the integer attribute as the last delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_15_UnchangedRealKeepsIntegerDelta() {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    let label = root_child(&document, 2);
    document.new_command().unwrap();
    TDataStdReal::set(&label, 100.0).unwrap();
    TDataStdInteger::set(&label, 300).unwrap();
    document.new_command().unwrap();
    TDataStdReal::set(&label, 100.0).unwrap();
    TDataStdInteger::set(&label, 400).unwrap();
    document.new_command().unwrap();
    expect_last_delta_is(&document, "TDataStd_Integer");
}

// bugs/caf/bug2932_17: an unchanged relation must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_17_UnchangedRelationKeepsRealDelta() {
    let relation = ExtendedString::new("New Relation");
    unchanged_keeps_real_delta(|label| {
        TDataStdRelation::set(label)
            .unwrap()
            .set_relation(&relation)
            .unwrap();
    });
}

// bugs/caf/bug2932_19: an unchanged variable must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_19_UnchangedVariableKeepsRealDelta() {
    unchanged_keeps_real_delta(|label| {
        let variable = TDataStdVariable::set(label).unwrap();
        variable.set_unit(&AsciiString::new("kg/m3")).unwrap();
        variable.constant(true).unwrap();
    });
}

// bugs/caf/bug2932_20: an unchanged label reference must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_20_UnchangedReferenceKeepsRealDelta() {
    let document = new_transaction_document();
    document.set_undo_limit(100).unwrap();
    let label = root_child(&document, 2);
    let reference = root_child(&document, 3);
    document.new_command().unwrap();
    TdfReference::set(&label, &reference).unwrap();
    TDataStdReal::set(&label, 300.0).unwrap();
    document.new_command().unwrap();
    TdfReference::set(&label, &reference).unwrap();
    TDataStdReal::set(&label, 400.0).unwrap();
    document.new_command().unwrap();
    expect_last_delta_is_real(&document);
}

// bugs/caf/bug2932_21: an unchanged tag source must not replace the real
// attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_21_UnchangedTagSourceKeepsRealDelta() {
    unchanged_keeps_real_delta(|label| {
        TdfTagSource::set(label).unwrap().set_value(5).unwrap();
    });
}

// bugs/caf/bug2932_22: an unchanged function attribute must not replace the
// real attribute modification in the transaction delta.
#[test]
fn TDocStd_Transaction_Test__CafBug_2932_22_UnchangedFunctionKeepsRealDelta() {
    let driver_guid = guid_from_text("5b35ca00-5b78-11d1-8940-080009dc3333");
    unchanged_keeps_real_delta(|label| {
        let function = TFunctionFunction::set_with_driver(label, &driver_guid).unwrap();
        function.set_failure(13).unwrap();
    });
}
