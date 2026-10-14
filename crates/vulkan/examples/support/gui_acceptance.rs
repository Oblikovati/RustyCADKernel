//! Identical analytic GUI cases run on virtual GPU targets and actual mapped windows.
#![allow(dead_code)]
use super::gui_acceptance_scene::GuiAcceptanceScene;
use super::{ais_scene, device_options, gui_packets, view_setup};
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectWindow, graphic3d::*};
use rustycad_tkv3d::ais::AisSelectedOwner;
fn native_fixture(use_native: bool) -> (Handle<Graphic3dRenderDriver>, GuiAcceptanceScene) {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    let scene = GuiAcceptanceScene::new(&driver, use_native);
    (driver, scene)
}
/// Prove the shared GUI case; example: `attachment_lifecycle(true)` uses native pixels.
pub fn attachment_lifecycle(use_native: bool) {
    let (driver, scene) = native_fixture(use_native);
    let baseline = scene.frame();
    install_gui_quad(&scene, [64, 64], [0., 1., 0., 1.], [12., 12., 52., 52.]);
    let overlay = scene.frame();
    assert_eq!(overlay.pixel(32, 32), Some([0, 255, 0, 255]));
    assert_clip_edges(&baseline, &overlay);
    replace_clear_resize(&scene, &baseline);
    close_gui_case(&driver, &scene);
}
fn assert_clip_edges(baseline: &FrameCapture, overlay: &FrameCapture) {
    for [x, y] in [[10, 32], [54, 32], [32, 10], [32, 54]] {
        assert_eq!(
            overlay.pixel(x, y),
            baseline.pixel(x, y),
            "clip boundary {x},{y}"
        );
    }
    assert_eq!(overlay.depth, baseline.depth);
}
fn replace_clear_resize(scene: &GuiAcceptanceScene, baseline: &FrameCapture) {
    let view = scene.scene().render_view();
    let invalid = gui_packets::gui_quad([64, 64], [1.; 4], [0., 0., 64., 64.]);
    assert!(view.attach_gui_layer([32, 64], invalid).is_err());
    install_gui_quad(scene, [64, 64], [1., 1., 0., 1.], [0., 0., 64., 64.]);
    assert_eq!(scene.frame().pixel(32, 32), Some([255, 255, 0, 255]));
    view.clear_gui_layer().unwrap();
    assert_eq!(scene.frame().rgba, baseline.rgba);
    verify_gui_resize(scene);
}
fn verify_gui_resize(scene: &GuiAcceptanceScene) {
    install_gui_quad(scene, [64, 64], [0., 1., 0., 1.], [0., 0., 64., 64.]);
    scene.resize(80, 72);
    let view = scene.scene().render_view();
    assert!(view.try_redraw().is_err() && view.capture().is_err());
    install_gui_quad(scene, [80, 72], [0., 1., 0., 1.], [0., 0., 80., 72.]);
    let resized = scene.frame();
    assert_eq!([resized.width, resized.height], [80, 72]);
}
/// Prove the shared GUI case; example: `alpha_and_uv(true)` uses native pixels.
pub fn alpha_and_uv(use_native: bool) {
    let (driver, scene) = native_fixture(use_native);
    let baseline = scene.frame();
    let first = gui_packets::gui_quad([64, 64], [0., 1., 0., 0.5], [0., 0., 64., 64.]);
    let second = gui_packets::gui_quad([64, 64], [0., 0., 1., 0.25], [0., 0., 64., 64.]);
    let draws = layered_quad_draws(first, second);
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], RenderLayer::new(draws, false, false, false))
        .unwrap();
    assert_rgb_near(scene.frame().pixel(32, 32).unwrap(), [165, 165, 137]);
    verify_font_uvs(&scene, &baseline);
    assert_eq!(scene.scene().viewer.active_views().len(), 1);
    assert_eq!(scene.scene().window.size(), (64, 64));
    scene.scene().view.remove();
    assert!(driver.validation_errors().is_empty());
}
fn layered_quad_draws(first: RenderLayer, second: RenderLayer) -> Vec<PrimitiveDraw> {
    // Public immutable frame access avoids exposing mutable layer internals to consumers.
    let frame = RenderFrame::with_layers(64, 64, [0.; 4], vec![first, second]).unwrap();
    frame.draws().to_vec()
}
fn assert_rgb_near(actual: [u8; 4], expected: [u8; 3]) {
    for index in 0..3 {
        assert!(
            actual[index].abs_diff(expected[index]) <= 1,
            "RGBA {actual:?}; expected sRGB {expected:?} within one quantization byte"
        );
    }
    assert_eq!(actual[3], 255);
}
fn verify_font_uvs(scene: &GuiAcceptanceScene, baseline: &FrameCapture) {
    let image = asymmetric_font_image();
    let draw = gui_packets::gui_image_quad([64, 64], [0., 1., 0., 1.], [0., 0., 64., 64.], image);
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], RenderLayer::new(vec![draw], false, false, false))
        .unwrap();
    let actual = scene.frame();
    assert_eq!(actual.pixel(16, 16), Some([0, 255, 0, 255]));
    for [x, y] in [[48, 16], [16, 48], [48, 48]] {
        assert_eq!(actual.pixel(x, y), baseline.pixel(x, y), "font UV {x},{y}");
    }
}

/// Prove the shared GUI case; example: `model_transparency(true)` uses native pixels.
pub fn model_transparency(use_native: bool) {
    for mode in [
        Graphic3dRenderTransparentMethod::BlendUnordered,
        Graphic3dRenderTransparentMethod::BlendOit,
        Graphic3dRenderTransparentMethod::DepthPeelingOit,
    ] {
        verify_gui_after_model_blend(mode, use_native);
    }
}
fn verify_gui_after_model_blend(mode: Graphic3dRenderTransparentMethod, use_native: bool) {
    let (driver, mut scene) = native_fixture(use_native);
    transparent_model(&mut scene);
    scene
        .scene()
        .view
        .change_rendering_params(&mut |params| params.transparency_method = mode);
    let baseline = scene.frame();
    assert_rgb_near(baseline.pixel(32, 32).unwrap(), [188, 0, 188]);
    install_gui_quad(&scene, [64, 64], [0., 1., 0., 0.5], [0., 0., 64., 64.]);
    let gui = scene.frame();
    assert_rgb_near(gui.pixel(32, 32).unwrap(), [137, 188, 137]);
    assert_eq!(gui.depth, baseline.depth);
    scene.scene().view.remove();
    assert!(driver.validation_errors().is_empty());
}
fn transparent_model(scene: &mut GuiAcceptanceScene) {
    scene
        .scene()
        .context
        .remove(&scene.scene().object.cast(), false)
        .unwrap();
    let style = view_setup::unlit_aspects::unlit_style([1., 0., 0., 0.5]);
    style.set_alpha_mode(Graphic3dAlphaMode::Blend, 0.5);
    let object = ais_scene::AisTriangle::new(
        scene.scene().object.vertices.clone(),
        [style.clone(), style],
    );
    scene
        .scene()
        .context
        .display(&object.cast(), false)
        .unwrap();
    scene.scene_mut().object = object;
}
/// Prove the shared GUI case; example: `selection_identity(true)` uses native pixels.
pub fn selection_identity(use_native: bool) {
    let (driver, scene) = native_fixture(use_native);
    let context = &scene.scene().context;
    context
        .set_selected_object(&scene.scene().object.cast(), false)
        .unwrap();
    let owner = selected_gui_owner(&scene);
    assert!(!owner.is_null());
    let bounds = selected_gui_bounds(&scene);
    install_gui_quad(&scene, [64, 64], [0., 1., 0., 1.], [0., 0., 64., 64.]);
    scene.frame();
    assert_eq!(selected_gui_owner(&scene), owner);
    assert_eq!(selected_gui_bounds(&scene), bounds);
    close_gui_case(&driver, &scene);
}
fn selected_gui_owner(scene: &GuiAcceptanceScene) -> AisSelectedOwner {
    let context = &scene.scene().context;
    context.init_selected();
    context.selected_owner().unwrap()
}
fn selected_gui_bounds(scene: &GuiAcceptanceScene) -> [f64; 6] {
    let context = &scene.scene().context;
    context
        .bounding_box_of_selection(Some(&scene.scene().view))
        .unwrap()
        .get()
        .unwrap()
}
fn install_gui_quad(scene: &GuiAcceptanceScene, extent: [u32; 2], color: [f32; 4], clip: [f32; 4]) {
    let layer = gui_packets::gui_quad(extent, color, clip);
    scene
        .scene()
        .render_view()
        .attach_gui_layer(extent, layer)
        .unwrap();
}
fn close_gui_case(driver: &Graphic3dRenderDriver, scene: &GuiAcceptanceScene) {
    scene.scene().view.remove();
    assert!(scene.scene().render_view().capture().is_err());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn asymmetric_font_image() -> RasterImage {
    let empty = [1., 1., 1., 0.];
    let pixels = vec![[1.; 4], empty, empty, empty];
    RasterImage::new([2, 2], pixels).unwrap()
}
/// Prove producer retirement/replacement/close; example: `font_retirement(true)`.
pub fn font_retirement(use_native: bool) {
    let (driver, scene) = native_fixture(use_native);
    let baseline = scene.frame();
    for _ in 0..8 {
        install_retired_font(&scene);
        assert_eq!(scene.frame().pixel(16, 16), Some([0, 255, 0, 255]));
    }
    scene.scene().render_view().clear_gui_layer().unwrap();
    assert_eq!(scene.frame().rgba, baseline.rgba);
    install_retired_font(&scene);
    scene.frame();
    close_gui_case(&driver, &scene);
}
fn install_retired_font(scene: &GuiAcceptanceScene) {
    let image = asymmetric_font_image();
    let draw = gui_packets::gui_image_quad([64, 64], [0., 1., 0., 1.], [0., 0., 64., 64.], image);
    let layer = RenderLayer::new(vec![draw], false, false, false);
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], layer)
        .unwrap();
}
/// Retain native and GPU byte buffers; example: `capture_native_overlay("target/gui")?`.
pub fn capture_native_overlay(prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (driver, scene) = native_fixture(true);
    install_gui_quad(&scene, [64, 64], [0., 1., 0., 1.], [12., 12., 52., 52.]);
    let capture = scene.frame();
    let native = scene.presented_pixels().unwrap();
    assert_eq!(native, capture.rgba);
    write_gui_buffers(prefix, &capture, &native)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
    close_gui_case(&driver, &scene);
    Ok(())
}
fn write_gui_buffers(prefix: &str, capture: &FrameCapture, native: &[u8]) -> std::io::Result<()> {
    std::fs::write(format!("{prefix}.rgba"), &capture.rgba)?;
    std::fs::write(format!("{prefix}.window.rgba"), native)?;
    let depth: Vec<u8> = capture
        .depth
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    std::fs::write(format!("{prefix}.depth"), depth)
}
/// Retained .9 model alpha remains scoped across GUI and clear; `model_alpha_scope(true)`.
pub fn model_alpha_scope(use_native: bool) {
    let (driver, scene) = native_fixture(use_native);
    scene
        .scene()
        .context
        .erase(&scene.scene().object.cast(), false)
        .unwrap();
    let (_structure, group) = masked_model_points(&scene);
    scene.frame();
    super::point_alpha_arrays::replace_alpha_points(&*group, "point-alpha-opaque");
    let baseline = scene.frame();
    assert_eq!(baseline.rgba, [0, 0, 255, 255].repeat(64 * 64));
    verify_translucent_gui_scope(&scene);
    scene.scene().render_view().clear_gui_layer().unwrap();
    assert_eq!(scene.frame().rgba, baseline.rgba);
    assert_eq!(scene.frame().rgba, baseline.rgba);
    close_gui_case(&driver, &scene);
}
fn verify_translucent_gui_scope(scene: &GuiAcceptanceScene) {
    install_gui_quad(scene, [64, 64], [0., 1., 0., 0.5], [0., 0., 64., 64.]);
    assert_rgb_near(scene.frame().pixel(32, 40).unwrap(), [0, 188, 188]);
    let layer = alpha_scoped_gui_layer();
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], layer)
        .unwrap();
    assert_rgb_near(scene.frame().pixel(32, 40).unwrap(), [0, 188, 188]);
}
fn masked_model_points(
    scene: &GuiAcceptanceScene,
) -> (Handle<Graphic3dStructure>, Handle<dyn Graphic3dGroup>) {
    let structure = Graphic3dStructure::new(
        &scene.scene().viewer.structure_manager(),
        &Handle::default(),
    )
    .unwrap();
    let group = structure.new_group().unwrap();
    group.set_group_primitives_aspect(&view_setup::unlit_aspects::unlit_style([1.; 4]));
    super::point_alpha_arrays::replace_alpha_points(&*group, "sprite-alpha-color-vertices");
    structure.display();
    (structure, group)
}
fn alpha_scoped_gui_layer() -> RenderLayer {
    let style = PointStyle::new(2., [1, 1], vec![[1.; 4]]).unwrap();
    let marker = PrimitiveDraw::points(
        vec![RenderVertex::new([0.; 3], [1.; 4])],
        vec![0],
        identity_clip(),
        style,
    )
    .unwrap();
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let quad = gui_packets::gui_image_quad([64, 64], [0., 1., 0., 0.5], [0., 0., 64., 64.], image);
    RenderLayer::new(vec![marker, quad], false, false, false)
}
