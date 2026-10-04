//! Cross-schema bindings share native current data; injected contexts isolate tests.
use super::super::fakes::{FixedClock, ScriptedDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::storage::{
    CallBack, Data, DefaultCallBack, OpenMode, Schema, SchemaContext, StreamException,
};
use std::{fmt::Write, sync::Arc};

fn retained_write(schema: &Schema) -> Handle<Data> {
    let current = Handle::new(Data::default());
    let mut driver = ScriptedDriver::opened(OpenMode::Write);
    driver.fail_at = Some(("BeginWriteInfoSection", StreamException::TypeMismatch));
    assert_eq!(
        schema
            .write(&mut driver, &current)
            .unwrap_err()
            .exception_type(),
        "Storage_StreamTypeMismatchError"
    );
    current
}

fn binding_sequence(first: &Schema, second: &Schema) -> String {
    let current = retained_write(first);
    let original: Arc<dyn CallBack> = Arc::new(DefaultCallBack);
    let replacement: Arc<dyn CallBack> = Arc::new(DefaultCallBack);
    let mut output = String::from("write raised\n");
    writeln!(
        output,
        "{} {}",
        i32::from(first.has_type_binding(&"A".into()).unwrap()),
        i32::from(second.has_type_binding(&"A".into()).unwrap())
    )
    .unwrap();
    first
        .bind_type(&"A".into(), Some(original.clone()))
        .unwrap();
    let shared = second.type_binding(&"A".into()).unwrap().unwrap();
    writeln!(
        output,
        "{} {}",
        i32::from(second.has_type_binding(&"A".into()).unwrap()),
        i32::from(Arc::ptr_eq(&shared, &original))
    )
    .unwrap();
    second.bind_type(&"A".into(), Some(replacement)).unwrap();
    append_binding_identity(&mut output, first, &original, &current);
    append_null_binding(&mut output, first, second, &current);
    output
}

fn append_binding_identity(
    output: &mut String,
    schema: &Schema,
    original: &Arc<dyn CallBack>,
    current: &Handle<Data>,
) {
    let retained = schema.type_binding(&"A".into()).unwrap().unwrap();
    let count = current
        .get()
        .unwrap()
        .type_data()
        .get()
        .unwrap()
        .number_of_types();
    writeln!(
        output,
        "{} {count}",
        i32::from(Arc::ptr_eq(&retained, original))
    )
    .unwrap();
}

fn append_null_binding(
    output: &mut String,
    first: &Schema,
    second: &Schema,
    current: &Handle<Data>,
) {
    second.bind_type(&"B".into(), None).unwrap();
    let count = current
        .get()
        .unwrap()
        .type_data()
        .get()
        .unwrap()
        .number_of_types();
    writeln!(
        output,
        "{} {} {} {count}",
        i32::from(first.has_type_binding(&"B".into()).unwrap()),
        i32::from(first.type_binding(&"B".into()).unwrap().is_none()),
        i32::from(first.type_binding(&"missing".into()).unwrap().is_none())
    )
    .unwrap();
}

#[test]
fn default_schemas_share_native_current_data_and_constructor_clears_it() {
    let first = Schema::with_clock(Arc::new(FixedClock));
    let second = Schema::default();
    assert_eq!(
        binding_sequence(&first, &second),
        include_str!("../../../../tools/oracle/tkernel_storage_schema_context.expected")
    );
    let _clearing_constructor = Schema::default();
    assert_eq!(
        first
            .has_type_binding(&"A".into())
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn injected_context_clones_share_bindings_and_separate_contexts_do_not() {
    let context = SchemaContext::default();
    let first = Schema::with_context(Arc::new(FixedClock), context.clone());
    let second = Schema::with_context(Arc::new(FixedClock), context);
    let separate = Schema::with_context(Arc::new(FixedClock), SchemaContext::default());
    assert_eq!(
        binding_sequence(&first, &second),
        include_str!("../../../../tools/oracle/tkernel_storage_schema_context.expected")
    );
    assert_eq!(
        separate
            .has_type_binding(&"A".into())
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
}

#[test]
fn completed_write_clears_the_shared_context_for_every_schema() {
    let context = SchemaContext::default();
    let first = Schema::with_context(Arc::new(FixedClock), context.clone());
    let second = Schema::with_context(Arc::new(FixedClock), context);
    let current = retained_write(&first);
    second.bind_type(&"A".into(), None).unwrap();
    first
        .write(&mut ScriptedDriver::opened(OpenMode::Read), &current)
        .unwrap();
    assert_eq!(
        second
            .type_binding(&"A".into())
            .err()
            .unwrap()
            .exception_type(),
        "Standard_NullObject"
    );
}
