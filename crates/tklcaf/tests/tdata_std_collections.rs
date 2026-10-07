//! Port-only tests of the TDataStd array and list attributes: the array deltas, the
//! native neutral values and null-array errors, the boolean bit packing and the list
//! editing members.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Handle, real::real_first};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{
    TDataStdBooleanArray, TDataStdBooleanList, TDataStdByteArray,
    TDataStdDeltaOnModificationOfIntArray, TDataStdExtStringArray, TDataStdIntegerArray,
    TDataStdIntegerList, TDataStdRealArray, TDataStdReferenceArray, TDataStdReferenceList,
};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfData, TdfRelocationTable};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

fn document() -> Handle<TDocStdDocument> {
    let document = TDocStdDocument::new(&ExtendedString::new("BinOcaf")).unwrap();
    document.set_undo_limit(10).unwrap();
    document
}

fn shared(values: &[i32], lower: i32) -> Handle<HArray1<i32>> {
    let array = HArray1::with_value(lower, lower + values.len() as i32 - 1, 0).unwrap();
    for (index, value) in (lower..).zip(values) {
        array.change_array1().set_value(index, *value).unwrap();
    }
    Handle::new(array)
}

#[test]
fn integer_array_delta_restores_changed_items_on_undo() {
    let document = document();
    let label = document.main().unwrap();
    let array = TDataStdIntegerArray::set(&label, 1, 4, true).unwrap();
    for index in 1..=4 {
        array.set_value(index, index * 10).unwrap();
    }
    document.new_command().unwrap();
    array.set_value(2, 99).unwrap();
    array.set_value(4, 77).unwrap();
    document.new_command().unwrap();
    let delta = document.get_undos().last().cloned().unwrap();
    let kinds: Vec<_> = delta
        .attribute_deltas()
        .iter()
        .map(|d| d.dynamic_type().unwrap().name().to_string())
        .collect();
    assert_eq!(kinds, vec!["TDataStd_DeltaOnModificationOfIntArray"]);
    document.undo().unwrap();
    let values: Vec<_> = (1..=4).map(|index| array.value(index).unwrap()).collect();
    assert_eq!(values, vec![10, 20, 30, 40]);
    document.redo().unwrap();
    assert_eq!((array.value(2).unwrap(), array.value(4).unwrap()), (99, 77));
}

#[test]
fn array_delta_restores_a_longer_old_array() {
    let document = document();
    let label = document.main().unwrap();
    let array = TDataStdRealArray::set(&label, 1, 3, true).unwrap();
    for index in 1..=3 {
        array.set_value(index, f64::from(index)).unwrap();
    }
    document.new_command().unwrap();
    let shorter = Handle::new(HArray1::with_value(1, 2, 5.0).unwrap());
    array.change_array(&shorter, true).unwrap();
    document.new_command().unwrap();
    assert_eq!(array.upper(), 2);
    document.undo().unwrap();
    assert_eq!(
        (
            array.upper(),
            array.value(1).unwrap(),
            array.value(3).unwrap()
        ),
        (3, 1.0, 3.0)
    );
}

#[test]
fn ext_string_and_byte_array_deltas_undo() {
    let document = document();
    let label = document.main().unwrap();
    let strings = TDataStdExtStringArray::set(&label, 1, 2, true).unwrap();
    let bytes = TDataStdByteArray::set(&label, 0, 1, true).unwrap();
    strings.set_value(1, ExtendedString::new("a")).unwrap();
    bytes.set_value(1, 7).unwrap();
    document.new_command().unwrap();
    strings.set_value(1, ExtendedString::new("b")).unwrap();
    bytes.set_value(1, 9).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert!(strings.value(1).unwrap().is_equal(ExtendedString::new("a")));
    assert_eq!(bytes.value(1).unwrap(), 7);
}

#[test]
fn delta_constructor_of_a_detached_backup_is_a_null_object() {
    // TDF_Label::FindAttribute on the null label of a detached attribute raises
    // Standard_NullObject natively ("A null Label has no attribute.").
    let old = Handle::new(TDataStdIntegerArray::new());
    old.init(1, 2).unwrap();
    let error = TDataStdDeltaOnModificationOfIntArray::new(&old)
        .err()
        .unwrap();
    assert!(error.is_kind("Standard_NullObject"));
    assert_eq!(old.length(), 2, "the backup keeps its array");
}

#[test]
fn arrays_without_array_answer_the_native_neutral_values() {
    let integers = TDataStdIntegerArray::new();
    let reals = TDataStdRealArray::new();
    let bytes = TDataStdByteArray::new();
    let references = TDataStdReferenceArray::new();
    assert_eq!(
        (
            integers.lower(),
            integers.upper(),
            integers.value(5).unwrap()
        ),
        (0, 0, 0)
    );
    assert_eq!(reals.value(1).unwrap(), real_first());
    assert_eq!((bytes.upper(), bytes.length()), (-1, 0));
    assert!(references.value(1).is_err(), "a null dereference natively");
    assert!(integers.set_value(1, 3).is_ok(), "nothing without array");
}

#[test]
fn change_array_of_a_null_integer_array_with_matching_bounds_is_a_null_object() {
    let integers = TDataStdIntegerArray::new();
    let error = integers.change_array(&shared(&[1], 0), true).err().unwrap();
    assert!(error.is_kind("Standard_NullObject"));
    let reals = TDataStdRealArray::new();
    let one = Handle::new(HArray1::with_value(0, 0, 1.0).unwrap());
    reals.change_array(&one, true).unwrap();
    assert_eq!(reals.value(0).unwrap(), 1.0);
}

#[test]
fn change_array_keeps_the_storage_when_the_bounds_match() {
    let data = TdfData::new();
    let array = TDataStdIntegerArray::set(&data.root(), 1, 2, false).unwrap();
    let before = array.array();
    array.change_array(&shared(&[5, 6], 1), true).unwrap();
    assert_eq!(array.array(), before);
    array.change_array(&shared(&[5, 6, 7], 1), true).unwrap();
    assert_ne!(array.array(), before);
    assert_eq!(array.value(3).unwrap(), 7);
    assert!(array.init(3, 1).unwrap_err().is_kind("Standard_RangeError"));
}

#[test]
fn boolean_array_packs_bits_and_ignores_indexes_outside_the_bounds() {
    let data = TdfData::new();
    let array = TDataStdBooleanArray::set(&data.root(), 3, 12).unwrap();
    array.set_value(3, true).unwrap();
    array.set_value(12, true).unwrap();
    array.set_value(12, false).unwrap();
    array.set_value(11, true).unwrap();
    let bytes: Vec<u8> = array
        .internal_array()
        .get()
        .unwrap()
        .array1()
        .iter()
        .copied()
        .collect();
    assert_eq!(bytes, vec![1, 1]);
    assert!(array.value(3).unwrap() && array.value(11).unwrap() && !array.value(12).unwrap());
    assert!(!array.value(2).unwrap() && !array.value(13).unwrap());
}

#[test]
fn integer_list_edits_by_value_and_by_index() {
    let data = TdfData::new();
    let list = TDataStdIntegerList::set(&data.root()).unwrap();
    list.append(2).unwrap();
    list.append(4).unwrap();
    assert!(list.insert_before(1, &2).unwrap());
    assert!(list.insert_after(3, &2).unwrap());
    assert!(list.insert_after_by_index(4, 5).unwrap());
    assert!(!list.insert_before_by_index(9, 0).unwrap());
    assert_eq!(list.list(), vec![1, 2, 3, 4, 5]);
    assert!(list.remove(&3).unwrap() && list.remove_by_index(1).unwrap());
    assert!(!list.remove(&42).unwrap());
    assert_eq!(list.list(), vec![2, 4, 5]);
    list.clear().unwrap();
    assert!(list.first().unwrap_err().is_kind("Standard_NoSuchObject"));
}

#[test]
fn boolean_list_stores_zero_and_one() {
    let data = TdfData::new();
    let list = TDataStdBooleanList::set(&data.root()).unwrap();
    list.append(true).unwrap();
    list.prepend(false).unwrap();
    assert!(list.insert_after_by_index(2, true).unwrap());
    assert_eq!(list.list(), vec![0, 1, 1]);
    assert!(!list.first().unwrap() && list.last().unwrap());
}

#[test]
fn reference_list_paste_relocates_and_drops_null_labels() {
    let data = TdfData::new();
    let root = data.root();
    let (source, target) = (
        root.find_child(1, true).unwrap(),
        root.find_child(2, true).unwrap(),
    );
    let list = TDataStdReferenceList::set(&root).unwrap();
    list.append(source.clone()).unwrap();
    list.append(Default::default()).unwrap();
    list.append(root.clone()).unwrap();
    let table = Handle::new(TdfRelocationTable::new(false));
    table.set_label_relocation(&source, &target);
    let into: Handle<dyn TdfAttribute> = Handle::new(TDataStdReferenceList::new()).cast();
    list.paste(&into, &table).unwrap();
    let pasted = into.cast::<TDataStdReferenceList>();
    assert_eq!(pasted.list(), vec![target, root]);
}

#[test]
fn reference_array_paste_relocates_into_the_target_bounds() {
    let data = TdfData::new();
    let root = data.root();
    let (source, target) = (
        root.find_child(1, true).unwrap(),
        root.find_child(2, true).unwrap(),
    );
    let array = TDataStdReferenceArray::set(&root, 1, 2).unwrap();
    array.set_value(1, source.clone()).unwrap();
    let table = Handle::new(TdfRelocationTable::new(false));
    table.set_label_relocation(&source, &target);
    let into: Handle<dyn TdfAttribute> = Handle::new(TDataStdReferenceArray::new()).cast();
    array.paste(&into, &table).unwrap();
    let pasted = into.cast::<TDataStdReferenceArray>();
    assert_eq!((pasted.lower(), pasted.upper()), (1, 2));
    assert_eq!(pasted.value(1).unwrap(), target);
    assert!(pasted.value(2).unwrap().is_null());
}
