//! Owned V1 rejection contract captured before the additive side-color policy.
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::*;
use std::fmt::Write;

#[test]
fn legacy_unlit_side_policy_replays_owned_rejections_and_repair() {
    let mut rows = String::new();
    for explicit in [false, true] {
        record_legacy_unlit_model(&mut rows, explicit);
    }
    assert_eq!(
        rows,
        include_str!("fixtures/unlit-side-color/legacy-v1.txt")
    );
}

fn record_legacy_unlit_model(rows: &mut String, explicit: bool) {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device, 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    scene.view.change_rendering_params(&mut |params| {
        params.shading_model = Graphic3dTypeOfShadingModel::Unlit;
    });
    configure_legacy_unlit_model(&scene, explicit);
    for (distinguish, back_vertex) in [(false, true), (true, true), (false, false), (true, false)] {
        record_legacy_unlit_case(rows, &scene, explicit, distinguish, back_vertex);
    }
    record_legacy_unlit_case(&mut String::new(), &scene, explicit, false, true);
}

fn configure_legacy_unlit_model(scene: &scene::ViewerScene, explicit: bool) {
    scene.group.aspects().set_shading_model(if explicit {
        Graphic3dTypeOfShadingModel::Unlit
    } else {
        Graphic3dTypeOfShadingModel::Default
    });
}

fn record_legacy_unlit_case(
    rows: &mut String,
    scene: &scene::ViewerScene,
    explicit: bool,
    distinguish: bool,
    back_vertex: bool,
) {
    scene.group.aspects().set_distinguish(distinguish);
    scene
        .group
        .aspects()
        .set_use_vertex_color_for_back_faces(back_vertex);
    let result = scene.render_view().try_redraw();
    let outcome = result.map_or_else(|error| error.to_string(), |()| "accepted".into());
    writeln!(rows, "{explicit} {distinguish} {back_vertex}: {outcome}").unwrap();
}
