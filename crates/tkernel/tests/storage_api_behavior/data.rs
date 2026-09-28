//! Storage_Data, Storage_Root, Storage_RootData, Storage_TypeData and Storage_InternalData.
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Handle, StandardPersistent};
use rustycad_tkernel::storage::{
    Data, InternalData, Root, RootData, StorageError, TypeData, version,
};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};

fn name(text: &str) -> AsciiString {
    AsciiString::new(text)
}

#[test]
fn unnamed_roots_take_the_next_root_number_and_rebinding_replaces() {
    let data = Data::default();
    let object = Handle::new(StandardPersistent::default());
    data.add_root(&object);
    data.add_root(&Handle::default());
    data.add_named_root(&name("1"), &Handle::default());
    assert_eq!(data.number_of_roots(), 2);
    assert!(
        data.find(&name("1"))
            .get()
            .expect("root")
            .object()
            .is_null()
    );
    assert!(data.is_root(&name("2")));
    data.remove_root(&name("2"));
    data.remove_root(&name("absent"));
    assert_eq!(data.number_of_roots(), 1);
    assert!(data.find(&name("2")).is_null());
}

#[test]
fn roots_share_their_objects_with_earlier_root_lists() {
    let roots = RootData::default();
    roots.add_root(&Handle::new(Root::new(&name("doc"), &Handle::default())));
    let listed = roots.roots();
    let object = Handle::new(StandardPersistent::default());
    roots.update_root(&name("doc"), &object).unwrap();
    let first = listed
        .get()
        .unwrap()
        .sequence(|items| items.first().cloned())
        .unwrap();
    assert!(first.get().unwrap().object().is_some());
    let failure = roots.update_root(&name("none"), &object).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn root_accessors_keep_every_field() {
    let root = Root::default();
    root.set_name(&name("n"));
    root.set_type(&name("PDocStd_Document"));
    root.set_reference(4);
    root.set_object(&Handle::new(StandardPersistent::default()));
    assert_eq!(root.name().as_bytes(), b"n");
    assert_eq!(root.type_name().as_bytes(), b"PDocStd_Document");
    assert_eq!(root.reference(), 4);
    assert!(root.object().is_some());
    let read = Root::with_reference(&name("r"), 2, &name("T"));
    assert!(read.object().is_null());
}

#[test]
fn types_keep_the_first_number_and_reject_unknown_lookups() {
    let types = TypeData::default();
    types.add_type(&name("A"), 5);
    types.add_type(&name("B"), 6);
    types.add_type(&name("A"), 9);
    assert_eq!(types.number_of_types(), 2);
    assert_eq!(types.type_number(&name("A")).unwrap(), 5);
    assert_eq!(types.type_name(2).unwrap().as_bytes(), b"B");
    for bad in [0, 3, -1] {
        assert_eq!(
            types.type_name(bad).unwrap_err().exception_type(),
            "Standard_NoSuchObject"
        );
    }
    assert!(types.type_number(&name("C")).is_err());
    let listed = types.types();
    assert_eq!(listed.get().unwrap().sequence(|items| items.len()), 2);
    types.clear();
    assert!(!types.is_type(&name("A")));
}

#[test]
fn header_fields_round_trip_through_the_data_set() {
    let data = Data::default();
    data.set_application_name(&ExtendedString::new("App"));
    data.set_application_version(&name("1.0"));
    data.set_data_type(&ExtendedString::new("Std"));
    data.add_to_user_info(&name("info"));
    data.add_to_comments(&ExtendedString::new("note"));
    let header = data.header_data();
    let header = header.get().unwrap();
    header.set_schema_name(&name("S"));
    header.set_schema_version(&name("2"));
    header.set_creation_date(&name("01/02/2003"));
    header.set_storage_version_number(13);
    header.set_number_of_objects(4);
    assert_eq!(
        data.application_name().as_utf16(),
        ExtendedString::new("App").as_utf16()
    );
    assert_eq!(data.application_version().as_bytes(), b"1.0");
    assert_eq!(
        data.data_type().as_utf16(),
        ExtendedString::new("Std").as_utf16()
    );
    assert_eq!(data.user_info().length(), 1);
    assert_eq!(data.comments().length(), 1);
    assert_eq!(data.schema_name().as_bytes(), b"S");
    assert_eq!(data.schema_version().as_bytes(), b"2");
    assert_eq!(data.creation_date().as_bytes(), b"01/02/2003");
    assert_eq!(data.storage_version().as_bytes(), b"13");
    assert_eq!(data.number_of_objects(), 4);
    assert_eq!(version().as_bytes(), b"1.3");
}

#[test]
fn clearing_resets_bookkeeping_and_types_but_keeps_roots() {
    let data = Data::default();
    data.add_root(&Handle::default());
    data.type_data().get().unwrap().add_type(&name("T"), 1);
    data.clear();
    assert_eq!(data.number_of_types(), 0);
    assert_eq!(data.number_of_roots(), 1);
    data.clear_error_status();
    assert_eq!(data.error_status(), StorageError::Ok);
    assert!(data.error_status_extension().is_empty());
    assert!(
        data.is_type(&name("T"))
            == data
                .types()
                .get()
                .unwrap()
                .sequence(|items| !items.is_empty())
    );
}

#[test]
fn internal_data_lends_its_read_array_handle() {
    let internal = InternalData::default();
    assert!(internal.read_array(|array| array.is_null()));
    internal.read_array(|array| *array = Handle::new(HArray1::with_bounds(1, 3).unwrap()));
    assert_eq!(
        internal.read_array(|array| array.get().unwrap().array1().len()),
        3
    );
    internal.clear();
    assert!(internal.read_array(|array| array.is_null()));
    let data = Data::default();
    assert!(data.internal_data().is_some() && data.root_data().is_some());
}
