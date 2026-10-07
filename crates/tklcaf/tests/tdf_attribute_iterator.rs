//! 1:1 translation of src/ApplicationFramework/TKLCAF/GTests/TDF_AttributeIterator_Test.cxx.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdInteger, TDataStdName, TDataStdReal};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeIterator};
use rustycad_tklcaf::tdoc_std::{TDocStdApplication, TDocStdDocument, TDocStdStandardApplication};

/// `anAttr->IsKind(STANDARD_TYPE(T))`.
fn is_kind<T: RuntimeTyped>(attribute: &Handle<dyn TdfAttribute>) -> bool {
    attribute.is_kind(T::static_type())
}

#[test]
fn TDF_AttributeIterator_Test__OCC24755_AttributeInsertionOrder() {
    let application = TDocStdStandardApplication::new();
    let mut document: Handle<TDocStdDocument> = Handle::default();
    application
        .new_tdoc_std_document(&ExtendedString::new("BinOcaf"), &mut document)
        .unwrap();
    let label = document.main().unwrap();
    // Prepend an int value.
    TDataStdInteger::set(&label, 0).unwrap();
    // Prepend a name.
    TDataStdName::set(&label, &ExtendedString::new("test")).unwrap();
    // Append a double value.
    let real: Handle<dyn TdfAttribute> = Handle::new(TDataStdReal::new()).cast();
    label.add_attribute(&real, true).unwrap();
    // Attributes come in insertion order: Integer, Name, Real.
    let mut iterator = TdfAttributeIterator::new(&label, true).unwrap();
    assert!(iterator.more(), "Iterator should have first attribute");
    assert!(
        is_kind::<TDataStdInteger>(&iterator.value()),
        "First attribute should be TDataStd_Integer"
    );
    iterator.next();
    assert!(iterator.more(), "Iterator should have second attribute");
    assert!(
        is_kind::<TDataStdName>(&iterator.value()),
        "Second attribute should be TDataStd_Name"
    );
    iterator.next();
    assert!(iterator.more(), "Iterator should have third attribute");
    assert!(
        is_kind::<TDataStdReal>(&iterator.value()),
        "Third attribute should be TDataStd_Real"
    );
    iterator.next();
    assert!(!iterator.more(), "Iterator should have no more attributes");
}

// caf/basic/B8: attribute order survives an undo/redo round trip.
#[test]
fn TDF_AttributeIterator_Test__CafBasic_B8_AttributeOrderAfterUndoRedo() {
    let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
    let label = document.main().unwrap().find_child(1, true).unwrap();
    document.new_command().unwrap();
    TDataStdInteger::set(&label, 123).unwrap();
    TDataStdReal::set(&label, 123.321).unwrap();
    document.new_command().unwrap();
    let check = |iterator: &mut TdfAttributeIterator| {
        assert!(iterator.more());
        assert!(is_kind::<TDataStdInteger>(&iterator.value()));
        iterator.next();
        assert!(iterator.more());
        assert!(is_kind::<TDataStdReal>(&iterator.value()));
        iterator.next();
        assert!(!iterator.more());
    };
    let mut iterator = TdfAttributeIterator::new(&label, true).unwrap();
    check(&mut iterator);
    document.undo().unwrap();
    document.redo().unwrap();
    iterator.initialize(&label, true).unwrap();
    check(&mut iterator);
}
