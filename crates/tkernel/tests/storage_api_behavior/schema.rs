//! Storage_Schema: Write, the callback registry, bindings and type migration.
use super::fakes::{FixedClock, ScriptedDriver, TableFile, UnreadableTable};
use rustycad_tkernel::standard::{Handle, StandardPersistent};
use rustycad_tkernel::storage::{
    CallBack, Data, DefaultCallBack, OpenMode, Schema, SchemaContext, StorageError,
    StreamException, TypeMigration, TypedCallBack,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::Arc;

fn schema() -> Schema {
    let mut schema = Schema::with_context(Arc::new(FixedClock), SchemaContext::default());
    schema.set_name(&"Std".into());
    schema.set_version(&"2".into());
    schema
}

#[test]
fn write_runs_every_section_and_stamps_the_header() {
    let data = Handle::new(Data::default());
    let set = data.get().unwrap();
    set.add_named_root(&"doc".into(), &Handle::default());
    set.add_to_user_info(&"info".into());
    let mut driver = ScriptedDriver::opened(OpenMode::Write);
    schema().write(&mut driver, &data).unwrap();
    let expected = [
        "BeginWriteInfoSection",
        "WriteInfo 0 Std 1",
        "EndWriteInfoSection",
        "BeginWriteCommentSection",
        "WriteComment 0",
        "EndWriteCommentSection",
        "BeginWriteTypeSection",
        "SetTypeSectionSize 0",
        "EndWriteTypeSection",
        "BeginWriteRootSection",
        "SetRootSectionSize 1",
        "WriteRoot doc 1 PDocStd_Document",
        "EndWriteRootSection",
        "BeginWriteRefSection",
        "SetRefSectionSize 0",
        "EndWriteRefSection",
        "BeginWriteDataSection",
        "EndWriteDataSection",
    ];
    assert_eq!(driver.log, expected);
    assert_eq!(set.creation_date().as_bytes(), b"03/02/2001");
    assert_eq!(set.storage_version().as_bytes(), b"1.3");
    assert_eq!(set.schema_version().as_bytes(), b"2");
    assert_eq!(set.error_status(), StorageError::Ok);
}

#[test]
fn a_write_error_becomes_the_status_of_the_failing_step() {
    let data = Handle::new(Data::default());
    let mut driver = ScriptedDriver::opened(OpenMode::ReadWrite);
    driver.fail_at = Some(("WriteComment 0", StreamException::Write));
    schema().write(&mut driver, &data).unwrap();
    let set = data.get().unwrap();
    assert_eq!(set.error_status(), StorageError::WriteError);
    assert_eq!(set.error_status_extension().as_bytes(), b"WriteComment");
}

#[test]
fn a_driver_not_open_for_writing_is_a_mode_error_and_null_data_does_nothing() {
    let data = Handle::new(Data::default());
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    schema().write(&mut driver, &data).unwrap();
    assert_eq!(data.get().unwrap().error_status(), StorageError::ModeError);
    schema().write(&mut driver, &Handle::default()).unwrap();
    assert!(driver.log.is_empty());
}

#[test]
fn a_root_object_without_a_bound_type_fails_in_the_data_section() {
    let data = Handle::new(Data::default());
    let object = Handle::new(StandardPersistent::default());
    data.get().unwrap().add_named_root(&"doc".into(), &object);
    let mut driver = ScriptedDriver::opened(OpenMode::Write);
    let failure = schema().write(&mut driver, &data).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(driver.log.contains(&"WriteReferenceType -1 0".to_string()));
    assert_eq!(data.get().unwrap().number_of_objects(), 1);
}

#[test]
fn bindings_need_a_write_in_progress() {
    let schema = schema();
    let object = Handle::new(StandardPersistent::default());
    for failure in [
        schema.has_type_binding(&"T".into()).unwrap_err(),
        schema.persistent_to_add(&object).unwrap_err(),
        schema.add_persistent(&object, "T").unwrap_err(),
        schema
            .bind_type(&"T".into(), &Handle::default())
            .unwrap_err(),
    ] {
        assert_eq!(failure.exception_type(), "Standard_NullObject");
    }
    assert!(!schema.add_persistent(&Handle::default(), "T").unwrap());
    let mut driver = ScriptedDriver::opened(OpenMode::Write);
    schema
        .write_persistent_reference(&Handle::default(), &mut driver)
        .unwrap();
    schema
        .write_persistent_object_header(&object, &mut driver)
        .unwrap();
    assert_eq!(
        driver.log,
        ["PutReference 0", "WritePersistentObjectHeader 0 0"]
    );
    assert!(
        schema
            .write_persistent_object_header(&Handle::default(), &mut driver)
            .is_err()
    );
}

#[test]
fn the_callback_registry_keeps_non_null_callbacks_by_type_name() {
    let mut schema = schema();
    let callback = Handle::new(DefaultCallBack::default()).cast::<dyn CallBack>();
    schema.add_read_unknown_type_callback(&"A".into(), &callback);
    schema.add_read_unknown_type_callback(&"B".into(), &Handle::default());
    let listed = schema.installed_callback_list();
    assert_eq!(listed.get().unwrap().sequence(|items| items.len()), 1);
    schema.remove_read_unknown_type_callback(&"A".into());
    schema.add_read_unknown_type_callback(&"C".into(), &callback);
    schema.clear_callback_list();
    assert_eq!(
        schema
            .installed_callback_list()
            .get()
            .unwrap()
            .sequence(|items| items.len()),
        0
    );
    assert!(!schema.is_using_default_callback());
    schema.use_default_callback();
    assert!(schema.is_using_default_callback());
    schema.dont_use_default_callback();
    schema.set_default_callback(&Handle::default());
    assert!(schema.default_callback().is_null());
    schema.reset_default_callback();
    assert!(!schema.default_callback().is_null());
    assert_eq!(schema.name().as_bytes(), b"Std");
    assert_eq!(schema.version().as_bytes(), b"2");
}

#[test]
fn default_callback_creates_plain_objects_and_skips_their_data() {
    let callback = DefaultCallBack::default();
    let schema = schema();
    let object = callback.new_persistent();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    callback.add(&object, &schema).unwrap();
    callback.write(&object, &mut driver, &schema).unwrap();
    callback.read(&object, &mut driver, &schema).unwrap();
    assert_eq!(driver.log, ["SkipObject"]);
    let typed = TypedCallBack::default();
    typed.set_type(&"T".into());
    typed.set_index(4);
    let callback = Handle::new(DefaultCallBack::default()).cast();
    typed.set_callback(&callback);
    assert_eq!(
        (typed.type_name().as_bytes(), typed.index()),
        (b"T".as_slice(), 4)
    );
    assert!(!typed.callback().is_null());
}

#[test]
fn migration_uses_the_table_file_or_the_built_in_renames() {
    let mut renamed = AsciiString::new("kept");
    let table = TypeMigration::load(&TableFile(b"Old New\nSolo\n\0hidden other\n"));
    assert!(table.check(&"Old".into(), &mut renamed));
    assert_eq!(renamed.as_bytes(), b"New");
    assert!(
        table.check(&"Solo\n".into(), &mut renamed),
        "Token() keeps the newline"
    );
    assert!(renamed.is_empty());
    assert!(
        !table.check(&"hidden".into(), &mut renamed),
        "an empty line ends the table"
    );
    let unreadable = TypeMigration::load(&UnreadableTable);
    assert!(!unreadable.check(&"TDataStd_Shape".into(), &mut renamed));
    assert!(Schema::check_type_migration(
        &"PDataStd_Axis".into(),
        &mut renamed
    ));
    assert_eq!(renamed.as_bytes(), b"PDataXtd_Axis");
    assert_eq!(Schema::i_creation_date().len(), 10);
}

#[test]
fn long_table_lines_are_read_in_79_byte_pieces() {
    let mut line = vec![b'k'; 79];
    line.extend_from_slice(b" v\n");
    let table = TypeMigration::load(&TableFile(Box::leak(line.into_boxed_slice())));
    let mut renamed = AsciiString::default();
    assert!(table.check(&AsciiString::from_bytes(vec![b'k'; 79]), &mut renamed));
    assert!(renamed.is_empty(), "the value is on the next piece");
    assert!(table.check(&"v\n".into(), &mut renamed));
}

#[path = "schema_context.rs"]
mod context;
