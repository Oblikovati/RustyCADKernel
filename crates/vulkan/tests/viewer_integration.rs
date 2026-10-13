//! Viewer-driven CPU tests inspect the real Graphic3d draw stream through a named fake device.
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{
    graphic3d::*,
    image::{ImageFormat, ImagePixMap},
};
use rustycad_tkv3d::v3d::V3dImageDumpOptions;
use std::sync::atomic::Ordering;
#[path = "support/recording_device.rs"]
mod recording_device;
use recording_device::RecordingDevice;
fn setup() -> (
    Handle<Graphic3dRenderDriver>,
    scene::ViewerScene,
    RecordingDevice,
) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    (driver, scene, device)
}
#[test]
fn ordinary_view_and_group_creation_reach_the_injected_backend() {
    let (driver, scene, device) = setup();
    scene.view.redraw();
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(!scene.group.cast::<Graphic3dDrawGroup>().is_null());
    assert_eq!(device.last_frame().draws().len(), 1);
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].position,
        [-8., -8., 0.]
    );
    let mut found = Handle::default();
    assert!(driver.view_exists(&scene.window.cast(), &mut found));
    assert_eq!(found, scene.cview());
    assert!(driver.validation_errors().is_empty());
    assert_eq!(driver.device_name(), "injected Martensite render backend");
}
#[test]
fn camera_geometry_visibility_and_removal_reach_redraw() {
    let (driver, scene, device) = setup();
    scene.view.redraw();
    let matrix = *device.last_frame().draws()[0].clip_transform();
    scene
        .view
        .camera()
        .set_projection_type(Graphic3dCameraProjection::Perspective);
    scene.vertices.set_vertice(1, 1., 2., 3.).unwrap();
    scene.view.redraw();
    assert_ne!(*device.last_frame().draws()[0].clip_transform(), matrix);
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].position,
        [1., 2., 3.]
    );
    scene.structure.set_visible(false);
    scene.view.redraw();
    assert!(device.last_frame().draws().is_empty());
    scene.view.remove();
    assert!(!driver.view_exists(&scene.window.cast(), &mut Handle::default()));
}
#[test]
fn resize_and_pixmap_dump_preserve_capture_rows_and_restore_target() {
    let (_, scene, device) = setup();
    scene.window.set_size(96, 48);
    scene.view.must_be_resized();
    scene.view.redraw();
    assert_eq!(device.last_frame().extent(), [96, 48]);
    assert!((scene.view.camera().aspect() - 2.).abs() < 1e-12);
    let image = ImagePixMap::default();
    let options = V3dImageDumpOptions {
        width: 48,
        height: 96,
        buffer_type: Graphic3dBufferType::Rgba,
        ..Default::default()
    };
    assert!(scene.view.to_pix_map(&image, &options).unwrap());
    assert_eq!((image.width(), image.height()), (48, 96));
    assert_eq!(&image.data().data.row(2).unwrap()[12..16], &[3, 2, 91, 255]);
    assert!(scene.cview().fbo().is_null());
}
#[test]
fn failed_redraw_cannot_return_the_previous_successful_capture() {
    let (_, scene, device) = setup();
    scene.view.redraw();
    assert!(scene.render_view().capture().is_ok());
    device.fail.store(true, Ordering::Relaxed);
    scene.view.redraw();
    assert!(scene.view.is_invalidated());
    assert!(scene.render_view().capture().is_err());
    let image = ImagePixMap::default();
    assert!(image.init_trash(ImageFormat::Rgba, 64, 64, 0));
    assert!(
        !scene
            .cview()
            .buffer_dump(&image, &Graphic3dBufferType::Rgba)
    );
    device.fail.store(false, Ordering::Relaxed);
    scene.view.redraw();
    assert!(!scene.view.is_invalidated());
}
#[test]
fn depth_dump_and_layer_policies_follow_viewer_state() {
    let (driver, scene, device) = setup();
    let mut settings = Graphic3dZLayerSettings::default();
    settings.set_enable_depth_test(false);
    settings.set_enable_depth_write(false);
    settings.set_clear_depth(false);
    driver
        .insert_layer_after(3, &settings, GRAPHIC3D_ZLAYER_ID_DEFAULT)
        .unwrap();
    scene.structure.set_z_layer(3);
    scene.cview().set_z_layer_target(3);
    scene.cview().set_z_layer_redraw_mode(true);
    scene.view.redraw();
    assert_eq!(device.last_frame().layers().len(), 1);
    assert_eq!(
        device.last_frame().layers()[0].depth_policy(),
        (false, false, false)
    );
    assert_depth_dump(&scene);
}
#[test]
fn layer_removal_preserves_geometry_and_later_erase() {
    let (driver, scene, device) = setup();
    driver
        .insert_layer_before(7, &Default::default(), GRAPHIC3D_ZLAYER_ID_TOP)
        .unwrap();
    scene.structure.set_z_layer(7);
    driver.remove_z_layer(7).unwrap();
    scene.view.redraw();
    assert_eq!(device.last_frame().draws().len(), 1);
    assert!(scene.cview().layer(7).is_null());
    scene.structure.erase();
    scene.view.redraw();
    assert!(device.last_frame().draws().is_empty());
}
#[test]
fn framebuffer_dimensions_limits_and_release_follow_the_dump_lifecycle() {
    let (_, scene, _) = setup();
    let view = scene.cview();
    assert!(view.fbo_create(0, 64).is_null());
    assert!(view.fbo_create(129, 64).is_null());
    let mut target = view.fbo_create(96, 64);
    assert_eq!(view.fbo_get_dimensions(&target), [96, 64, 96, 64]);
    view.fbo_change_viewport(&target, 32, 48);
    assert_eq!(view.fbo_get_dimensions(&target), [32, 48, 96, 64]);
    view.fbo_change_viewport(&target, 97, 48);
    assert_eq!(view.fbo_get_dimensions(&target), [32, 48, 96, 64]);
    view.set_fbo(&target);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(scene.render_view().capture().unwrap().width, 32);
    view.set_fbo(&Handle::default());
    view.fbo_release(&mut target);
    assert!(target.is_null());
}
#[test]
fn unsupported_window_and_removed_view_fail_explicitly() {
    use rustycad_tkservice::aspect::AspectWindowMembers;
    let (_, scene, _) = setup();
    scene.window.set_virtual(false);
    assert!(scene.render_view().try_redraw().is_err());
    scene.window.set_virtual(true);
    scene
        .cview()
        .set_window(&Handle::default(), &scene.window.cast(), 42);
    assert!(scene.render_view().try_redraw().is_err());
    scene
        .cview()
        .set_window(&Handle::default(), &scene.window.cast(), 0);
    scene.render_view().try_redraw().unwrap();
    scene.view.remove();
    assert!(scene.render_view().capture().is_err());
}
#[test]
fn padded_bottom_up_rgb_and_red_captures_preserve_logical_rows() {
    let (_, scene, _) = setup();
    scene.view.redraw();
    for (kind, format, channels) in [
        (Graphic3dBufferType::Rgb, ImageFormat::Rgb, 3),
        (Graphic3dBufferType::Red, ImageFormat::Gray, 1),
    ] {
        let image = ImagePixMap::default();
        assert!(image.init_trash(format, 64, 64, 64 * channels + 8));
        image.set_top_down(false);
        assert!(scene.cview().buffer_dump(&image, &kind));
        let pixels = image.data();
        assert_eq!(pixels.data.row(2).unwrap()[3 * channels], 3);
        if channels == 3 {
            assert_eq!(&pixels.data.row(2).unwrap()[9..12], &[3, 2, 91]);
        }
    }
}
#[test]
fn buffer_dump_rejects_incompatible_dimensions_and_formats() {
    let (_, scene, _) = setup();
    scene.view.redraw();
    for (format, width) in [(ImageFormat::Rgb, 64), (ImageFormat::Rgba, 32)] {
        let image = ImagePixMap::default();
        assert!(image.init_trash(format, width, 64, 0));
        assert!(
            !scene
                .cview()
                .buffer_dump(&image, &Graphic3dBufferType::Rgba)
        );
    }
    assert!(Graphic3dRenderDriver::with_backend(RecordingDevice::default(), 0).is_err());
    assert!(Graphic3dRenderDriver::with_backend(RecordingDevice::default(), u32::MAX).is_err());
}

fn assert_depth_dump(scene: &scene::ViewerScene) {
    let image = ImagePixMap::default();
    assert!(image.init_trash(ImageFormat::GrayF, 64, 64, 0));
    assert!(
        scene
            .cview()
            .buffer_dump(&image, &Graphic3dBufferType::Depth)
    );
    assert_eq!(
        &image.data().data.row(0).unwrap()[..4],
        &0.25f32.to_ne_bytes()
    );
}
#[test]
fn unsupported_multisampling_and_connected_rendering_return_errors() {
    let (_, scene, _) = setup();
    scene
        .cview()
        .change_rendering_params(&mut |params| params.nb_msaa_samples = 4);
    scene.render_view().try_redraw().unwrap();
    scene
        .cview()
        .change_rendering_params(&mut |params| params.nb_msaa_samples = 0);
    scene.cview().change_rendering_params(&mut |params| {
        params.transparency_method = Graphic3dRenderTransparentMethod::BlendOit;
    });
    scene.render_view().try_redraw().unwrap();
    scene.cview().change_rendering_params(&mut |params| {
        params.transparency_method = Graphic3dRenderTransparentMethod::BlendUnordered;
    });
    let child =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &Handle::default()).unwrap();
    scene
        .structure
        .connect(&child.cast(), Graphic3dTypeOfConnection::Descendant, true);
    assert_redraw_error(&scene, "connected");
    scene.structure.disconnect(&child.cast());
    scene.render_view().try_redraw().unwrap();
    scene.structure.clear(true);
    scene.render_view().try_redraw().unwrap();
}

fn assert_redraw_error(scene: &scene::ViewerScene, expected: &str) {
    let error = scene.render_view().try_redraw().unwrap_err();
    assert!(error.to_string().contains(expected), "{error}");
}

#[test]
fn native_windows_reject_fake_devices_and_virtual_dump_remains_available() {
    use rustycad_tkservice::aspect::{AspectWindow, AspectWindowMembers};
    let (_, scene, _) = setup();
    scene.window.set_virtual(false);
    assert!(
        scene
            .render_view()
            .try_redraw()
            .unwrap_err()
            .to_string()
            .contains("nonzero native window")
    );
    scene.window.set_native_handle(42);
    scene.window.map();
    assert!(
        scene
            .render_view()
            .try_redraw()
            .unwrap_err()
            .to_string()
            .contains("injected backend")
    );
    let target = scene.cview().fbo_create(32, 32);
    scene.cview().set_fbo(&target);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(scene.render_view().capture().unwrap().width, 32);
    scene.view.remove();
    assert!(scene.viewer.active_views().is_empty());
}

#[allow(dead_code)]
#[path = "support/lighting_scene.rs"]
mod lighting_scene;
#[test]
fn shadow_requests_are_explicit_and_disabled_casters_remain_renderable() {
    let (driver, _, _) = setup();
    let lit = lighting_scene::LightingScene::new(&driver, "lit-phong");
    lit.light.set_cast_shadows(true).unwrap();
    let error = lit.base.render_view().try_redraw().unwrap_err();
    assert!(error.to_string().contains("zero enabled shadow casters"));
    lit.light.set_enabled(false);
    lit.base.render_view().try_redraw().unwrap();
    lit.light.set_enabled(true);
    lit.base
        .view
        .change_rendering_params(&mut |params| params.ray_tracing.is_shadow_enabled = false);
    lit.base.render_view().try_redraw().unwrap();
}
#[test]
fn default_unlit_view_rejects_front_only_color_and_material_distinction() {
    let (driver, _, _) = setup();
    let lit = lighting_scene::LightingScene::new(&driver, "lit-default");
    lit.base.view.change_rendering_params(&mut |params| {
        params.shading_model = Graphic3dTypeOfShadingModel::Unlit
    });
    lit.aspect.set_distinguish(true);
    assert!(lit.base.render_view().try_redraw().is_err());
    lit.aspect.set_distinguish(false);
    lit.aspect.set_use_vertex_color_for_back_faces(false);
    assert!(lit.base.render_view().try_redraw().is_err());
    lit.aspect.set_use_vertex_color_for_back_faces(true);
    lit.base.render_view().try_redraw().unwrap();
}
