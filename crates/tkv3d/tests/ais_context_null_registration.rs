//! A protected null map key is a real iterator entry, distinct from the end position.
#[path = "support/context_object.rs"]
mod context_object;
use context_object::ContextObject;
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::{ais::*, v3d::V3dViewer};
struct NullRegistration {
    context: Handle<AisInteractiveContext>,
}
impl NullRegistration {
    fn new(context: &Handle<AisInteractiveContext>) -> Self {
        context
            .fields
            .write()
            .objects
            .bind(Handle::default(), Handle::new(AisGlobalStatus::new()));
        Self {
            context: context.clone(),
        }
    }
}
impl Drop for NullRegistration {
    fn drop(&mut self) {
        self.context
            .fields
            .write()
            .objects
            .unbind(&Handle::default());
    }
}
#[test]
fn unfiltered_query_preserves_null_registration_and_continues_bucket_traversal() {
    let context = null_query_context();
    let first = ContextObject::new();
    let second = ContextObject::new();
    context.load(&first.cast(), -1).unwrap();
    context.load(&second.cast(), -1).unwrap();
    let guard = NullRegistration::new(&context);
    let expected = registered_keys(&context);
    let mut output = List::default();
    context
        .objects_inside(&mut output, AisKindOfInteractive::None, -1)
        .unwrap();
    drop(guard);
    assert_eq!(output.iter().cloned().collect::<Vec<_>>(), expected);
    assert_eq!(output.len(), 3);
    assert_eq!(output.iter().filter(|key| key.is_null()).count(), 1);
}
#[test]
fn null_presentation_clear_ignores_even_a_registered_null_key() {
    let context = null_query_context();
    let guard = NullRegistration::new(&context);
    context
        .clear_global_prs(&Handle::default(), 0, true)
        .unwrap();
    context.clear_prs(&Handle::default(), 0, true).unwrap();
    assert_eq!(context.fields.read().objects.len(), 1);
    drop(guard);
    assert!(context.fields.read().objects.is_empty());
}

fn null_query_context() -> Handle<AisInteractiveContext> {
    let driver = NullGraphicDriver::new_handle();
    let viewer = V3dViewer::new(&driver).unwrap();
    AisInteractiveContext::new(&viewer).unwrap()
}
fn registered_keys(context: &AisInteractiveContext) -> Vec<Handle<dyn AisInteractiveObject>> {
    context
        .fields
        .read()
        .objects
        .items()
        .map(|(key, _)| key.clone())
        .collect()
}
