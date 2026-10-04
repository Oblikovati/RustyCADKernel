//! Cross-schema bindings share native current data; injected contexts isolate tests.
use super::super::fakes::{FixedClock, ScriptedDriver};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, StandardPersistent, StandardTransient};
use rustycad_tkernel::storage::{
    BaseDriver, CallBack, Data, DefaultCallBack, OpenMode, Schema, SchemaContext, StreamException,
};
use std::{
    any::TypeId,
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
    let original: Arc<dyn CallBack> = Arc::new(DefaultCallBack::default());
    let replacement: Arc<dyn CallBack> = Arc::new(DefaultCallBack::default());
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

fn invoke_all_callback_overrides(bound: &Arc<dyn CallBack>, schema: &Schema) {
    let persistent = bound.new_persistent();
    let mut driver = ScriptedDriver::opened(OpenMode::Read);
    bound.add(&persistent, schema).unwrap();
    bound.write(&persistent, &mut driver, schema).unwrap();
    bound.read(&persistent, &mut driver, schema).unwrap();
    assert!(persistent.is_some());
}

fn bind_custom_callback(schema: &Schema, concrete: &Arc<DispatchCallBack>) -> Arc<dyn CallBack> {
    let callback: Arc<dyn CallBack> = concrete.clone();
    schema
        .bind_type(&"dispatch".into(), Some(callback.clone()))
        .unwrap();
    schema.type_binding(&"dispatch".into()).unwrap().unwrap()
}

fn callback_runtime_report(bound: &Arc<dyn CallBack>) -> String {
    format!(
        "1 4\n{}\n{}\n",
        bound.dynamic_type().name(),
        i32::from(bound.is_kind(<dyn CallBack as RuntimeTyped>::static_type()))
    )
}

fn assert_callback_runtime(bound: &Arc<dyn CallBack>, concrete: &DispatchCallBack) {
    assert_eq!(bound.dynamic_type().name(), "TestDispatchCallBack");
    assert!(bound.is_kind(<dyn CallBack as RuntimeTyped>::static_type()));
    assert!(bound.is_kind(StandardTransient::static_type()));
    let projected = bound.project(TypeId::of::<DispatchCallBack>()).unwrap();
    assert!(std::ptr::eq(
        projected.downcast_ref::<DispatchCallBack>().unwrap(),
        concrete
    ));
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

#[test]
fn custom_callback_binding_preserves_rtti_identity_and_virtual_dispatch() {
    let schema = Schema::with_context(Arc::new(FixedClock), SchemaContext::default());
    let _current = retained_write(&schema);
    let concrete = Arc::new(DispatchCallBack {
        parent: StandardTransient::default(),
        calls: AtomicUsize::new(0),
    });
    let bound = bind_custom_callback(&schema, &concrete);
    let callback: Arc<dyn CallBack> = concrete.clone();
    assert!(Arc::ptr_eq(&callback, &bound));
    assert_callback_runtime(&bound, concrete.as_ref());
    invoke_all_callback_overrides(&bound, &schema);
    assert_eq!(concrete.calls.load(Ordering::Relaxed), 4);
    assert_eq!(
        callback_runtime_report(&bound),
        include_str!("../../../../tools/oracle/tkernel_storage_schema_callback_dispatch.expected")
    );
}
