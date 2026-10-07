//! Behavior tests of ShapeProcess_Context, ShapeProcess_ShapeContext, ShapeProcess and
//! ShapeProcessAPI_ApplySequence (the native comparison is the sp_* requests of
//! tools/tkshhealing_corpus_shape_process.py).
mod healing_support;

use healing_support::square_face;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkshhealing::shape_process::{
    ShapeProcess, ShapeProcessContext, ShapeProcessOperLibrary, ShapeProcessOperation,
    ShapeProcessOperationsFlags, ShapeProcessShapeContext,
};

fn context_with(values: &[(&str, &str)]) -> ShapeProcessContext {
    let context = ShapeProcessContext::with_file("", "").unwrap();
    let resource = context.resource_manager().unwrap();
    for (name, value) in values {
        resource.write().set_resource(name, value.as_bytes());
    }
    context
}

#[test]
fn values_are_read_under_the_scope_with_aliases() {
    let context = context_with(&[
        ("seq.Tol", "0.25"),
        ("seq.Alias", " & Base "),
        ("Base", "7"),
        ("seq.Flag", "1"),
        ("seq.Word", "C2"),
    ]);
    assert!(!context.is_param_set("Tol"));
    context.set_scope("seq");
    assert!(context.is_param_set("Tol"));
    assert_eq!(context.get_real("Tol").unwrap(), Some(0.25));
    assert_eq!(context.get_integer("Alias").unwrap(), Some(7));
    assert_eq!(context.get_boolean("Flag"), Some(true));
    assert_eq!(context.get_boolean("Word"), None);
    assert_eq!(context.string_val("Missing", "def").as_bytes(), b"def");
    context.un_set_scope();
    assert_eq!(context.real_val("Tol", -1.).unwrap(), -1.);
}

#[test]
fn blank_numeric_value_raises_out_of_range_as_natively() {
    let context = context_with(&[("Blank", "   ")]);
    assert!(context.get_real("Blank").is_err());
}

#[test]
fn shape_context_reads_continuities_and_records_results() {
    let context = ShapeProcessShapeContext::new("", "").unwrap();
    context
        .resource_manager()
        .unwrap()
        .write()
        .set_resource("Cont", b" g2 ");
    assert_eq!(
        context.continuity_val("Cont", Continuity::C0),
        Continuity::G2
    );
    let face = square_face(0.).into_shape();
    context.init(&face);
    assert!(context.result().is_same(&face));
    context.set_result(&rustycad_tkbrep::topods::Shape::default());
    assert_eq!(context.map().extent(), 1);
}

#[test]
fn registry_finds_the_library_operators_and_operation_names() {
    ShapeProcessOperLibrary::init().unwrap();
    for operation in ShapeProcessOperation::ALL {
        assert!(ShapeProcess::find_operator(operation.name()).is_some());
        assert_eq!(
            ShapeProcess::to_operation_flag(Some(operation.name())),
            (operation, true)
        );
    }
    assert!(ShapeProcess::find_operator("NoSuchOperator").is_none());
}

#[test]
fn fix_shape_flag_runs_on_a_shape_context() {
    ShapeProcessOperLibrary::init().unwrap();
    let face = square_face(0.).into_shape();
    let context = Handle::new(ShapeProcessShapeContext::with_shape(&face, "", "").unwrap());
    let mut flags = ShapeProcessOperationsFlags::default();
    flags.set(ShapeProcessOperation::FixShape);
    let base: Handle<ShapeProcessContext> = context.clone().cast();
    assert!(ShapeProcess::perform_operations(&base, flags, &ProgressRange::default()).unwrap());
    assert!(!context.result().is_null());
}
