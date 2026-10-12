//! Public UpdateAttributes resolves the genuine associated context's current-mode presentation.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
#[allow(dead_code)]
#[path = "support/textured_face_fixture.rs"]
mod face_fixture;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, v3d::V3dViewer};

#[test]
fn textured_public_attributes_replay_real_context_presentation_update() {
    let (object, context, trace) = public_textured_attributes_fixture();
    let previous_texture = object.textured_shape_base().fields.read().texture.clone();
    let previous_aspect = object.textured_shape_base().fields.read().aspect.clone();
    trace.write().events.clear();
    object.set_texture_pix_map(&public_textured_pixels(false));
    object.update_attributes().unwrap();
    let prs = context
        .main_prs_mgr()
        .presentation(&object.cast(), 3, false, &Handle::default())
        .unwrap();
    let previous = (previous_texture, previous_aspect);
    let row = public_textured_attributes_row(&object, &prs, previous, trace.read().events.len());
    let native = include_str!("fixtures/ais_textured_shape/native-public-attributes.txt");
    assert_eq!(row, native.trim());
}

type PreviousPublicTextureAspects = (
    Handle<dyn Graphic3dTexture2DSource>,
    Handle<Graphic3dAspectFillArea3d>,
);
fn public_textured_attributes_row(
    object: &AisTexturedShape,
    prs: &PrsMgrPresentation,
    previous: PreviousPublicTextureAspects,
    event_count: usize,
) -> String {
    let fields = object.textured_shape_base().fields.read();
    let current_group = prs.groups().first().unwrap().aspects();
    format!(
        "MARTENSITE_PUBLIC_ATTRIBUTES {} {} {} {} {}",
        prs.groups().len(),
        i32::from(previous.0 != fields.texture),
        i32::from(previous.1 != fields.aspect),
        i32::from(current_group.object_address() == fields.aspect.object_address()),
        event_count
    )
}

fn public_textured_attributes_fixture() -> (
    Handle<AisTexturedShape>,
    Handle<AisInteractiveContext>,
    display_driver::DisplayTrace,
) {
    let (driver, trace) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let context = AisInteractiveContext::new(&viewer).unwrap();
    let face = face_fixture::textured_face_fixture(1).unwrap();
    let object = AisTexturedShape::new(&face);
    object.set_context(&context);
    configure_public_textured_attributes(&object);
    context.main_prs_mgr().display(&object.cast(), 3).unwrap();
    (object, context, trace)
}
fn configure_public_textured_attributes(object: &AisTexturedShape) {
    let drawer = object.attributes();
    drawer.setup_own_defaults().unwrap();
    drawer.set_auto_triangulation(false);
    drawer.u_iso_aspect().set_number(0);
    drawer.v_iso_aspect().set_number(0);
    drawer.set_display_mode(3);
    object.set_texture_pix_map(&public_textured_pixels(true));
}
fn public_textured_pixels(top_down: bool) -> Handle<ImagePixMap> {
    let pixels = Handle::new(ImagePixMap::default());
    assert!(pixels.init_zero(ImageFormat::Rgba, 2, 2, 0, 0));
    pixels.set_top_down(top_down);
    pixels
}
