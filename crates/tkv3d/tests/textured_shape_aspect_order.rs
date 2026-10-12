//! Native update ordering regressions use actual aspect/texture handles with named I/O fakes.
#[allow(dead_code)] // Shared named fixture also serves source-state-only replay modules.
#[path = "support/textured_shape_state_fixture.rs"]
mod state_fixture;
#[path = "support/textured_texture_factory_fixture.rs"]
mod texture_factory;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers};
use state_fixture::TexturedStateFixture;
use std::sync::{Arc, atomic::Ordering};
use texture_factory::TexturedTextureFactoryFixture;

fn prepared_textured_state(
    factory: Arc<TexturedTextureFactoryFixture>,
) -> Handle<TexturedStateFixture> {
    let object = TexturedStateFixture::with_factory(factory);
    // Native updateAttributes assumes a drawer initialized by context/default setup.
    object
        .attributes()
        .setup_own_shading_aspect(&Handle::default())
        .unwrap();
    object
}
#[test]
fn texture_constructor_failure_replaces_aspect_before_retaining_old_texture() {
    let factory = Arc::new(TexturedTextureFactoryFixture::new(true, true));
    let object = prepared_textured_state(factory.clone());
    let old_aspect = Handle::new(Graphic3dAspectFillArea3d::default());
    object.base.fields.write().aspect = old_aspect.clone();
    object.base.fields.write().texture = factory.texture.clone();
    let error = ais_textured_shape_update_attributes(&*object, &Handle::default()).unwrap_err();
    assert_eq!(
        error.message(),
        "texture construction failed; expected an available injected source"
    );
    assert_ne!(object.base.fields.read().aspect, old_aspect);
    assert_eq!(object.base.fields.read().texture, factory.texture);
    assert_eq!(factory.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn disabled_mapping_retains_texture_and_skips_factory_before_null_presentation_failure() {
    let factory = Arc::new(TexturedTextureFactoryFixture::new(true, true));
    let object = prepared_textured_state(factory.clone());
    object.base.fields.write().texture = factory.texture.clone();
    object.set_texture_map_off();
    object.set_show_triangles(true);
    let error = ais_textured_shape_update_attributes(&*object, &Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let fields = object.base.fields.read();
    assert_eq!(fields.texture, factory.texture);
    assert_eq!(
        fields.aspect.texture_map(),
        factory.texture.clone().cast::<dyn Graphic3dTextureMap>()
    );
    assert!(!fields.aspect.to_map_texture());
    assert!(fields.aspect.to_draw_edges());
    assert_eq!(factory.calls.load(Ordering::Relaxed), 0);
    assert!(object.diagnostics.messages.read().is_empty());
}
#[test]
fn failed_availability_still_assigns_actual_texture_and_modulation_before_null_prs() {
    let factory = Arc::new(TexturedTextureFactoryFixture::new(false, false));
    let object = prepared_textured_state(factory.clone());
    object.set_texture_pix_map(&Handle::new(
        rustycad_tkservice::image::ImagePixMap::default(),
    ));
    object.disable_texture_modulate();
    let error = ais_textured_shape_update_attributes(&*object, &Handle::default()).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    let fields = object.base.fields.read();
    assert_eq!(fields.texture, factory.texture);
    assert!(!fields.aspect.to_map_texture());
    assert!(!factory.texture.texture_root_base().params.is_modulate());
    assert_eq!(
        object.diagnostics.messages.read()[0].1.as_bytes(),
        b"Error: texture can not be loaded  (custom image)"
    );
    assert_eq!(factory.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn public_update_attributes_preserves_native_work_before_rejecting_absent_presentation() {
    let factory = Arc::new(TexturedTextureFactoryFixture::new(true, true));
    let object = prepared_textured_state(factory.clone());
    object.set_texture_map_off();
    assert!(object.base.fields.read().aspect.is_null());
    let error = object.update_attributes().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(!object.base.fields.read().aspect.is_null());
    assert_eq!(factory.calls.load(Ordering::Relaxed), 0);
}
#[test]
fn uninitialized_native_shading_precondition_has_typed_failure_without_fabricated_defaults() {
    let object = TexturedStateFixture::new();
    let error = object.update_attributes().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(error.message().contains("null shading aspect"));
    assert!(object.base.fields.read().aspect.is_null());
    assert!(object.base.fields.read().texture.is_null());
}

#[test]
fn textured_file_constructor_consumes_cstring_prefix_without_rewriting_stored_filename() {
    let factory = Arc::new(TexturedTextureFactoryFixture::new(false, true));
    let object = prepared_textured_state(factory.clone());
    let file = rustycad_tkernel::tcollection::AsciiString::from_bytes(b"texture.png\0suffix");
    object.set_texture_file_name(&file).unwrap();
    assert!(ais_textured_shape_update_attributes(&*object, &Handle::default()).is_err());
    let sources = factory.sources.lock().unwrap();
    let AisTexturedTextureSource::File(path) = &sources[0] else {
        panic!("expected native file source");
    };
    assert_eq!(
        path.as_bytes(),
        object.texture_file().read().unwrap().as_bytes()
    );
    assert_eq!(object.base.fields.read().texture_file, file);
    compare_native_cstring_path(path.as_bytes());
    assert_eq!(path.as_bytes(), b"texture.png");
}

fn compare_native_cstring_path(path: &[u8]) {
    let native = include_str!("fixtures/ais_textured_shape/native-aspects.txt");
    let actual = path
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert!(native.contains(&format!("MARTENSITE_CSTRING_PATH {actual} {actual}")));
}
