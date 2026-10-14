//! CPU extraction acceptance follows the public opt-in driver; V1 fixtures stay unchanged.
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)] // The shared fixture also supports lifecycle/geometry cases.
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderUnlitFaceColorPolicy};
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::graphic3d::*;

#[test]
fn retained_unlit_v2_accepts_explicit_and_inherited_side_flags() {
    for model in [
        Graphic3dTypeOfShadingModel::Unlit,
        Graphic3dTypeOfShadingModel::Default,
    ] {
        let device = recording_device::RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend_and_unlit_face_color_policy(
            device.clone(),
            128,
            RenderUnlitFaceColorPolicy::SideColorsV2,
        )
        .unwrap();
        let scene = scene::ViewerScene::new(&driver);
        scene
            .view
            .change_rendering_params(&mut |p| p.shading_model = Graphic3dTypeOfShadingModel::Unlit);
        scene.group.aspects().set_shading_model(model);
        verify_retained_side_flags(&scene, &device);
    }
}
fn verify_retained_side_flags(
    scene: &scene::ViewerScene,
    device: &recording_device::RecordingDevice,
) {
    let aspect = scene.group.aspects();
    aspect.set_back_interior_color_rgba(ColorRgba::new(0., 1., 0., 0.25).unwrap());
    for (distinguish, back_vertex) in [(false, true), (true, true), (false, false), (true, false)] {
        aspect.set_distinguish(distinguish);
        aspect.set_use_vertex_color_for_back_faces(back_vertex);
        scene.render_view().try_redraw().unwrap();
        let frame = device.last_frame();
        assert_eq!(frame.draws().len(), 1);
        assert_eq!(frame.draws()[0].is_surface_blended(), distinguish);
    }
}
#[test]
fn retained_unlit_v2_validates_injected_limit_and_explicit_v1_replay() {
    let device = recording_device::RecordingDevice::default();
    assert!(
        Graphic3dRenderDriver::with_backend_and_unlit_face_color_policy(
            device.clone(),
            0,
            RenderUnlitFaceColorPolicy::SideColorsV2
        )
        .is_err()
    );
}
#[test]
fn retained_explicit_v1_replays_rejection_and_repair() {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_unlit_face_color_policy(
        device,
        128,
        RenderUnlitFaceColorPolicy::LegacyV1,
    )
    .unwrap();
    let scene = scene::ViewerScene::new(&driver);
    scene.group.aspects().set_distinguish(true);
    assert!(scene.render_view().try_redraw().is_err());
    scene.group.aspects().set_distinguish(false);
    scene.render_view().try_redraw().unwrap();
}
