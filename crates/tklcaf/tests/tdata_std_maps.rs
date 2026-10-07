//! Port-only tests of TDataStd_NamedData, the HDataMap classes and TDataStd_IntPackedMap.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdIntPackedMap, TDataStdNamedData, TDataStdNamedMap};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfData, TdfRelocationTable};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;

fn name(text: &str) -> ExtendedString {
    ExtendedString::new(text)
}

fn document() -> Handle<TDocStdDocument> {
    let document = TDocStdDocument::new(&name("BinOcaf")).unwrap();
    document.set_undo_limit(10).unwrap();
    document
}

#[test]
fn named_data_values_undo() {
    let document = document();
    let named = TDataStdNamedData::set(&document.main().unwrap()).unwrap();
    named.set_integer(&name("i"), 1).unwrap();
    named.set_string(&name("s"), name("a")).unwrap();
    document.new_command().unwrap();
    named.set_integer(&name("i"), 2).unwrap();
    named.set_string(&name("s"), name("b")).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert_eq!(named.get_integer(&name("i")).unwrap(), 1);
    assert!(named.get_string(&name("s")).unwrap().is_equal(name("a")));
}

#[test]
fn set_real_binds_a_new_name_without_backup() {
    // TDataStd_NamedData::SetReal binds an unknown name without Backup(), so the undo of
    // that command cannot remove it.
    let document = document();
    let named = TDataStdNamedData::set(&document.main().unwrap()).unwrap();
    document.new_command().unwrap();
    named.set_real(&name("x"), 1.5).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    assert_eq!(named.get_real(&name("x")).unwrap(), 1.5);
}

#[test]
fn named_data_arrays_are_deep_copies() {
    let data = TdfData::new();
    let named = TDataStdNamedData::set(&data.root()).unwrap();
    let array = Handle::new(HArray1::with_value(1, 2, 4).unwrap());
    named.set_array_of_integers(&name("a"), &array).unwrap();
    array.change_array1().set_value(1, 9).unwrap();
    let stored = named.get_array_of_integers(&name("a")).unwrap();
    assert_eq!(*stored.array1().value(1).unwrap(), 4);
    assert!(named.has_array_of_integers(&name("a")) && !named.has_arrays_of_reals());
    assert!(
        named
            .get_byte(&name("b"))
            .unwrap_err()
            .is_kind("Standard_NoSuchObject")
    );
}

#[test]
fn change_integers_with_its_own_map_does_nothing() {
    let data = TdfData::new();
    let named = TDataStdNamedData::set(&data.root()).unwrap();
    named.set_integer(&name("i"), 1).unwrap();
    let container = named.get_integers_container();
    let copy = {
        let own = container.map();
        named.change_integers(&own).unwrap();
        own.find(&name("i")).cloned().unwrap()
    };
    assert_eq!(copy, 1);
}

#[test]
fn named_data_paste_and_dump() {
    let data = TdfData::new();
    let named = TDataStdNamedData::set(&data.root()).unwrap();
    named.set_integer(&name("i"), 1).unwrap();
    named.set_byte(&name("b"), 7).unwrap();
    let into: Handle<dyn TdfAttribute> = Handle::new(TDataStdNamedData::new()).cast();
    named
        .paste(&into, &Handle::new(TdfRelocationTable::new(false)))
        .unwrap();
    let pasted = into.cast::<TDataStdNamedData>();
    assert_eq!(pasted.get_byte(&name("b")).unwrap(), 7);
    let mut text = Vec::new();
    pasted.dump(&mut text).unwrap();
    assert_eq!(
        String::from_utf8(text).unwrap(),
        "NamedData: \tIntegers = 1\tReals = 0\tStrings = 0\tBytes = 1\tArraysOfIntegers = 0\tArraysOfReals = 0"
    );
}

#[test]
fn int_packed_map_delta_undo_adds_back_and_removes() {
    let document = document();
    let map = TDataStdIntPackedMap::set(&document.main().unwrap(), true).unwrap();
    for key in [1, 2, 3] {
        map.add(key).unwrap();
    }
    document.new_command().unwrap();
    map.remove(2).unwrap();
    map.add(9).unwrap();
    document.new_command().unwrap();
    document.undo().unwrap();
    let mut keys: Vec<i32> = map.get_map().iter().collect();
    keys.sort();
    assert_eq!(keys, vec![1, 2, 3]);
    document.redo().unwrap();
    assert!(map.contains(9) && !map.contains(2));
}

#[test]
fn int_packed_map_change_map_and_clear() {
    let data = TdfData::new();
    let map = TDataStdIntPackedMap::set(&data.root(), false).unwrap();
    let other = Handle::new(rustycad_tkernel::tcol_std::HPackedMapOfInteger::default());
    other.change_map().add(4);
    assert!(map.change_h_map(&other).unwrap());
    assert!(!map.change_h_map(&other).unwrap(), "equal contents");
    assert!(!map.change_h_map(&Handle::default()).unwrap(), "null map");
    assert!(map.clear().unwrap() && !map.clear().unwrap());
    let hmap = rustycad_tklcaf::tdata_std::TDataStdHDataMapOfStringReal::new(3);
    assert!(hmap.map().is_empty());
}
