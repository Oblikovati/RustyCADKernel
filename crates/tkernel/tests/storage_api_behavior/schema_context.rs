//! Cross-schema bindings share native current data; injected contexts isolate tests.
use super::super::fakes::{FixedClock, ScriptedDriver};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardPersistent, StandardTransient};
use rustycad_tkernel::storage::{
    BaseDriver, CallBack, Data, DefaultCallBack, OpenMode, Schema, SchemaContext, StreamException,
};
use std::{
    fmt::Write,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

struct DispatchCallBack {
    parent: StandardTransient,
    calls: AtomicUsize,
}

rustycad_tkernel::impl_transient!(
    DispatchCallBack,
    "TestDispatchCallBack",
    dyn CallBack,
    parent,
    interfaces: [dyn CallBack]
);

impl CallBack for DispatchCallBack {
    fn new_persistent(&self) -> Handle<StandardPersistent> {
        self.record_call();
        Handle::new(StandardPersistent::default())
    }

    fn add(
        &self,
        _: &Handle<StandardPersistent>,
        _: &Schema,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.record_call();
        Ok(())
    }

    fn write(
        &self,
        _: &Handle<StandardPersistent>,
        _: &mut dyn BaseDriver,
        _: &Schema,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.record_call();
        Ok(())
    }

    fn read(
        &self,
        _: &Handle<StandardPersistent>,
        _: &mut dyn BaseDriver,
        _: &Schema,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        self.record_call();
        Ok(())
    }
}

impl DispatchCallBack {
    fn record_call(&self) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}

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
    let original = Handle::new(DefaultCallBack::default()).cast::<dyn CallBack>();
    let replacement = Handle::new(DefaultCallBack::default()).cast::<dyn CallBack>();
    let mut output = String::from("write raised\n");
    writeln!(
        output,
        "{} {}",
        i32::from(first.has_type_binding(&"A".into()).unwrap()),
        i32::from(second.has_type_binding(&"A".into()).unwrap())
    )
    .unwrap();
    first.bind_type(&"A".into(), &original).unwrap();
    let shared = second.type_binding(&"A".into()).unwrap();
    writeln!(
        output,
        "{} {}",
        i32::from(second.has_type_binding(&"A".into()).unwrap()),
        i32::from(shared == original)
    )
    .unwrap();
    second.bind_type(&"A".into(), &replacement).unwrap();
    append_binding_identity(&mut output, first, &original, &current);
    append_null_binding(&mut output, first, second, &current);
    output
}

fn append_binding_identity(
    output: &mut String,
    schema: &Schema,
    original: &Handle<dyn CallBack>,
    current: &Handle<Data>,
) {
    let retained = schema.type_binding(&"A".into()).unwrap();
    let count = current
        .get()
        .unwrap()
        .type_data()
        .get()
        .unwrap()
        .number_of_types();
    writeln!(output, "{} {count}", i32::from(retained == *original)).unwrap();
}

fn append_null_binding(
    output: &mut String,
    first: &Schema,
    second: &Schema,
    current: &Handle<Data>,
) {
    second.bind_type(&"B".into(), &Handle::default()).unwrap();
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
        i32::from(first.type_binding(&"B".into()).unwrap().is_null()),
        i32::from(first.type_binding(&"missing".into()).unwrap().is_null())
    )
    .unwrap();
}

fn invoke_all_callback_overrides(bound: &Handle<dyn CallBack>, schema: &Schema) {
    let persistent = bound.new_persistent();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    bound.add(&persistent, schema).unwrap();
    bound.write(&persistent, &mut driver, schema).unwrap();
    bound.read(&persistent, &mut driver, schema).unwrap();
    assert!(persistent.is_some());
}

fn callback_runtime_report(
    bound: &Handle<dyn CallBack>,
    binding_count: i32,
    copied_count: i32,
    this_matches: bool,
    cast_count: i32,
    derived_matches: bool,
    base_matches: bool,
) -> String {
    format!(
        "{binding_count}\n{copied_count} {}\n{cast_count} {} {}\n1 4\n{}\n{}\n",
        i32::from(this_matches),
        i32::from(derived_matches),
        i32::from(base_matches),
        bound.dynamic_type().unwrap().name(),
        i32::from(bound.is_kind(<dyn CallBack as RuntimeTyped>::static_type()))
    )
}

fn assert_callback_runtime(bound: &Handle<dyn CallBack>, concrete: &Handle<DispatchCallBack>) {
    assert_eq!(bound.dynamic_type().unwrap().name(), "TestDispatchCallBack");
    assert!(bound.is_kind(<dyn CallBack as RuntimeTyped>::static_type()));
    assert!(bound.is_kind(StandardTransient::static_type()));
    let projected = bound.cast::<DispatchCallBack>();
    assert!(projected == *concrete);
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
    second.bind_type(&"A".into(), &Handle::default()).unwrap();
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

#[test]
fn custom_callback_binding_preserves_rtti_identity_and_virtual_dispatch() {
    let schema = Schema::with_context(Arc::new(FixedClock), SchemaContext::default());
    let _current = retained_write(&schema);
    let concrete = Handle::new(DispatchCallBack {
        parent: StandardTransient::default(),
        calls: AtomicUsize::new(0),
    });
    let callback = concrete.cast::<dyn CallBack>();
    schema.bind_type(&"dispatch".into(), &callback).unwrap();
    let binding_count = callback.ref_count();
    let bound = schema.type_binding(&"dispatch".into()).unwrap();
    assert!(bound == callback);
    assert_callback_runtime(&bound, &concrete);
    let copied_count = bound.ref_count();
    assert_eq!(copied_count, 4);
    let transient = bound.get().unwrap().transient_base();
    let this_matches = std::ptr::eq(transient.this().unwrap(), transient);
    let derived = bound.cast::<DispatchCallBack>();
    let base = bound.cast::<StandardTransient>();
    let cast_count = bound.ref_count();
    let derived_matches = derived == concrete;
    let base_matches = base == bound;
    assert_eq!(cast_count, 6);
    assert!(derived_matches && base_matches && this_matches);
    invoke_all_callback_overrides(&bound, &schema);
    assert_eq!(concrete.get().unwrap().calls.load(Ordering::Relaxed), 4);
    assert_eq!(
        callback_runtime_report(
            &bound,
            binding_count,
            copied_count,
            this_matches,
            cast_count,
            derived_matches,
            base_matches,
        ),
        include_str!("../../../../tools/oracle/tkernel_storage_schema_callback_dispatch.expected")
    );
}
