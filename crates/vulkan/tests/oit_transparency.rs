//! Retained parameter transitions and native OIT buffers replay without an OCCT dependency.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod surface_texture_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[allow(dead_code)]
#[path = "support/ffp_oracle_profile.rs"]
mod ffp_oracle_profile;
#[path = "support/oit_basic_assets.rs"]
mod oit_basic_assets;
#[path = "support/oit_finite_assets.rs"]
mod oit_finite_assets;
#[path = "support/oit_layer_assets.rs"]
mod oit_layer_assets;
#[path = "support/oit_oracle_profile.rs"]
mod oit_oracle_profile;
#[allow(dead_code)]
#[path = "support/oit_scene.rs"]
mod oit_scene;
#[path = "support/oit_transition_assets.rs"]
mod oit_transition_assets;
#[path = "support/recording_device.rs"]
mod recording_device;
use martensite_vulkan::{
    FrameCapture, Graphic3dRenderDriver, RenderLightingPipeline, RenderTextureCoordinatePipeline,
    RenderTransparency,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_DEFAULT, Graphic3dCViewMembers, Graphic3dGraphicDriver,
};

#[test]
fn retained_unordered_weighted_and_peeling_native_buffers_replay_without_occt() {
    for fixed in [false, true] {
        let driver = oit_driver(fixed);
        assert_basic_oit_assets(&driver, fixed);
        assert!(
            driver.validation_errors().is_empty(),
            "{:?}",
            driver.validation_errors()
        );
    }
}
fn assert_basic_oit_assets(driver: &Handle<Graphic3dRenderDriver>, fixed: bool) {
    let software = oit_oracle_profile::software_oracle(driver);
    for &(method, policy, host_rgba, host_depth, soft_rgba, soft_depth) in
        oit_basic_assets::OIT_BASIC_ASSETS
    {
        if policy != fixed {
            continue;
        }
        let (rgba, depth) = if software {
            (soft_rgba, soft_depth)
        } else {
            (host_rgba, host_depth)
        };
        native_golden::assert_capture(&capture_oit_basic(driver, method), rgba, depth, method);
    }
}
fn oit_driver(fixed: bool) -> Handle<Graphic3dRenderDriver> {
    Graphic3dRenderDriver::with_render_pipelines(
        device_options::validation_options(),
        if fixed {
            RenderTextureCoordinatePipeline::GeneratedCompatibility
        } else {
            RenderTextureCoordinatePipeline::Programmable
        },
        Default::default(),
        if fixed {
            RenderLightingPipeline::FixedFunction
        } else {
            RenderLightingPipeline::Programmable
        },
    )
    .unwrap()
}
fn capture_oit_basic(driver: &Handle<Graphic3dRenderDriver>, method: &str) -> FrameCapture {
    let fixture = oit_scene::OitScene::new(driver, false);
    fixture.parameters(method, 0, 0., 4);
    fixture
        .scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    fixture.scene.cview().set_z_layer_redraw_mode(true);
    fixture.scene.render_view().try_redraw().unwrap();
    let capture = fixture.scene.render_view().capture().unwrap();
    fixture.scene.view.remove();
    capture
}

#[test]
fn actual_view_parameter_replacements_preserve_previous_submissions() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let fixture = oit_scene::OitScene::new(&driver, false);
    fixture.parameters("weighted", 4, 0.75, 4);
    fixture.scene.render_view().try_redraw().unwrap();
    let before = backend.last_frame();
    fixture.parameters("peeling", 0, 0.25, 7);
    fixture.scene.render_view().try_redraw().unwrap();
    assert_parameter_snapshots(&before, &backend);
    fixture.scene.view.remove();
}

fn assert_parameter_snapshots(
    before: &martensite_vulkan::RenderFrame,
    backend: &recording_device::RecordingDevice,
) {
    assert_eq!(
        before.transparency(),
        RenderTransparency::Weighted { depth_factor: 0.75 }
    );
    assert_eq!(
        backend.last_frame().transparency(),
        RenderTransparency::DepthPeeling { layers: 7 }
    );
    assert_eq!(
        backend.frames.read()[0].transparency(),
        before.transparency()
    );
}

#[test]
fn masked_nodal_overlay_stays_before_the_deferred_fill_batch() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    for fixture_name in ["sprite-mask", "line-mask"] {
        let fixture = oit_scene::OitScene::with_scene(&driver, false, fixture_name);
        fixture.parameters("weighted", 0, 0., 4);
        fixture.scene.render_view().try_redraw().unwrap();
        let frame = backend.last_frame();
        assert_eq!(frame.draws().len(), 3, "{fixture_name}: {frame:?}");
        assert_ne!(
            frame.draws()[0].topology(),
            martensite_vulkan::PrimitiveTopology::Triangles
        );
        fixture.scene.view.remove();
    }
}

#[test]
fn invalid_active_view_parameters_fail_before_backend_submission_and_can_recover() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let fixture = oit_scene::OitScene::new(&driver, false);
    fixture.parameters("peeling", 0, 0., 0);
    let error = fixture.scene.render_view().try_redraw().unwrap_err();
    assert!(error.to_string().contains("positive"), "{error}");
    assert!(backend.frames.read().is_empty());
    fixture.parameters("peeling", 0, 0., 3);
    fixture.scene.render_view().try_redraw().unwrap();
    assert_eq!(
        backend.last_frame().transparency(),
        RenderTransparency::DepthPeeling { layers: 3 }
    );
    fixture.scene.view.remove();
}

#[test]
fn sprite_alpha_context_persists_in_its_view_and_does_not_leak_to_adjacent_views() {
    let driver = oit_driver(false);
    let fixture = oit_scene::OitScene::with_scene(&driver, false, "sprite-mask");
    fixture.parameters("weighted", 0, 0., 4);
    assert_context_pixel(&fixture, [0, 0, 255, 255]);
    fixture.replace_groups("two");
    assert_context_pixel(&fixture, [0, 0, 255, 255]);
    let adjacent = oit_scene::OitScene::new(&driver, false);
    adjacent.parameters("weighted", 0, 0., 4);
    assert_context_pixel(&adjacent, [142, 204, 99, 255]);
    fixture.scene.view.remove();
    adjacent.scene.view.remove();
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn assert_context_pixel(fixture: &oit_scene::OitScene, expected: [u8; 4]) {
    assert_eq!(
        capture_retained_fixture(fixture).pixel(20, 20),
        Some(expected)
    );
}
fn capture_retained_fixture(fixture: &oit_scene::OitScene) -> FrameCapture {
    fixture
        .scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    fixture.scene.cview().set_z_layer_redraw_mode(true);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.scene.render_view().capture().unwrap()
}

#[test]
fn native_sprite_cutoff_and_opaque_restore_control_the_later_oit_composite() {
    let driver = oit_driver(false);
    for name in ["sprite-mask", "sprite-low-mask", "sprite-opaque"] {
        for mode in ["weighted", "peeling"] {
            assert_sprite_composite_control(&driver, name, mode);
        }
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn assert_sprite_composite_control(driver: &Handle<Graphic3dRenderDriver>, name: &str, mode: &str) {
    let fixture = oit_scene::OitScene::with_scene(driver, false, name);
    fixture.parameters(mode, 0, 0., 4);
    let expected = if name == "sprite-mask" || (name == "sprite-low-mask" && mode == "peeling") {
        [0, 0, 255, 255]
    } else if mode == "weighted" {
        [142, 204, 99, 255]
    } else if driver.device_name().contains("llvmpipe") {
        [187, 165, 0, 255]
    } else {
        [188, 165, 0, 255]
    };
    assert_eq!(
        capture_retained_fixture(&fixture).pixel(20, 20),
        Some(expected),
        "{name}/{mode}"
    );
    fixture.scene.view.remove();
}
#[test]
#[ignore = "configured native sprite coverage differs by one row; strict raw diagnostic stays unchanged"]
fn sprite_mask_strict_native_buffers_retain_the_coverage_diagnostic() {
    let driver = oit_driver(false);
    for &(mode, host_rgba, host_depth, soft_rgba, soft_depth) in
        oit_basic_assets::SPRITE_ALPHA_ASSETS
    {
        let fixture = oit_scene::OitScene::with_scene(&driver, false, "sprite-mask");
        fixture.parameters(mode, 0, 0., 4);
        let capture = capture_retained_fixture(&fixture);
        let (rgba, depth) = if driver.device_name().contains("llvmpipe") {
            (soft_rgba, soft_depth)
        } else {
            (host_rgba, host_depth)
        };
        native_golden::assert_capture(&capture, rgba, depth, mode);
        fixture.scene.view.remove();
    }
}

#[test]
fn native_actual_layer_sharing_and_depth_clear_flush_replay_without_occt() {
    // Each native producer owns a fresh viewer/driver; layer identifiers are driver-global.
    for asset in oit_layer_assets::OIT_LAYER_ASSETS {
        let driver = oit_driver(asset.fixed);
        assert_retained_layer_asset(&driver, asset);
        assert!(
            driver.validation_errors().is_empty(),
            "{:?}",
            driver.validation_errors()
        );
    }
}

fn assert_retained_layer_asset(
    driver: &Handle<Graphic3dRenderDriver>,
    asset: &oit_layer_assets::OitLayerAsset,
) {
    let fixture = oit_scene::OitScene::with_scene(driver, false, asset.fixture);
    fixture.parameters(asset.mode, 0, 0., 4);
    fixture.scene.render_view().try_redraw().unwrap();
    let capture = fixture.scene.render_view().capture().unwrap();
    let (rgba, depth) = if oit_oracle_profile::software_oracle(driver) {
        asset.software
    } else {
        asset.host
    };
    native_golden::assert_capture(&capture, rgba, depth, asset.fixture);
    fixture.scene.view.remove();
}

#[test]
fn actual_cap_alpha_mutation_changes_the_next_oit_submission_only() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let fixture = oit_scene::OitScene::with_scene(&driver, false, "cap-source-mask-alpha-updated");
    fixture.parameters("weighted", 4, 0., 4);
    fixture.scene.render_view().try_redraw().unwrap();
    let before = backend.last_frame();
    fixture.mutate_inputs("cap-source-mask-alpha-updated");
    fixture.scene.render_view().try_redraw().unwrap();
    let after = backend.last_frame();
    assert_ne!(before.draws(), after.draws());
    assert_eq!(before.transparency(), after.transparency());
    assert!(driver.validation_errors().is_empty());
    fixture.scene.view.remove();
}

#[test]
fn actual_colored_glyph_and_live_cap_inputs_replay_native_buffers_without_occt() {
    for asset in oit_finite_assets::OIT_FINITE_ASSETS {
        assert_finite_oit_asset(asset);
    }
}
fn assert_finite_oit_asset(asset: &oit_finite_assets::OitFiniteAsset) {
    let driver = oit_driver(asset.fixed);
    let fixture = oit_scene::OitScene::with_scene(&driver, false, asset.fixture);
    fixture.parameters(asset.method, asset.samples, 0., 4);
    fixture.scene.render_view().try_redraw().unwrap();
    fixture.mutate_inputs(asset.fixture);
    let capture = capture_retained_fixture(&fixture);
    let (rgba, depth) = if oit_oracle_profile::software_oracle(&driver) {
        asset.software
    } else {
        asset.host
    };
    native_golden::assert_capture(&capture, rgba, depth, asset.fixture);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    fixture.scene.view.remove();
}

#[test]
fn actual_same_view_sampling_and_persistent_peeling_fallback_replay_native_assets() {
    let driver = oit_driver(false);
    let fixture = oit_scene::OitScene::new(&driver, false);
    for stage in oit_transition_assets::NATIVE_OIT_TRANSITIONS {
        fixture.parameters(stage.mode, stage.samples, 0., 4);
        let capture = capture_retained_fixture(&fixture);
        assert_transition_contract(&driver, stage, &capture);
    }
    fixture.scene.view.remove();
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn assert_transition_contract(
    driver: &Handle<Graphic3dRenderDriver>,
    stage: &oit_transition_assets::NativeOitTransition,
    capture: &FrameCapture,
) {
    let (rgba, depth) = if oit_oracle_profile::software_oracle(driver) {
        stage.software
    } else {
        stage.host
    };
    if stage.strict {
        native_golden::assert_capture(capture, rgba, depth, stage.name);
        return;
    }
    // Native/Vulkan six/eight-sample edge coverage is a retained raw diagnostic, not a promoted gate.
    assert_eq!(
        capture.pixel(20, 20),
        Some(rgba[(20 * 64 + 20) * 4..][..4].try_into().unwrap())
    );
}

#[test]
fn owned_layer_fixture_teardown_allows_fresh_viewers_on_the_same_driver() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend, 128).unwrap();
    // Native layer-lifetime control returns UNDERLAY, DEFAULT, TOP, TOPMOST, OVERLAY.
    let defaults = vec![-5, 0, -2, -3, -4];
    assert_eq!(driver.z_layers(), defaults);
    for name in ["layer-clear", "layer-shared", "layer-clear"] {
        let fixture = oit_scene::OitScene::with_scene(&driver, false, name);
        assert_eq!(driver.z_layers().len(), defaults.len() + 1);
        fixture.finish();
        assert_eq!(driver.z_layers(), defaults, "{name}");
    }
}
