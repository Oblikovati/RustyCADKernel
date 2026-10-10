//! Live view/structure state is snapshotted at redraw, with native override and chain semantics.
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkservice::graphic3d::*;
fn setup() -> (scene::ViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    (scene::ViewerScene::new(&driver), device)
}
fn plane(equation: [f64; 4]) -> Handle<Graphic3dClipPlane> {
    Handle::new(
        Graphic3dClipPlane::from_equation(Vec4::new(
            equation[0],
            equation[1],
            equation[2],
            equation[3],
        ))
        .unwrap(),
    )
}
fn sequence(plane: &Handle<Graphic3dClipPlane>) -> Handle<Graphic3dSequenceOfHClipPlane> {
    let sequence = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    sequence.append(plane.clone());
    sequence
}
#[test]
fn view_planes_follow_live_equations_flags_and_chain_links_without_mutating_prior_frames() {
    let (scene, device) = setup();
    let first = plane([1., 0., 0., 0.]);
    let second = plane([0., 1., 0., 2.]);
    second.set_on(false).unwrap();
    Graphic3dClipPlane::set_chain_next_plane(&first, second).unwrap();
    scene.cview().set_clip_planes(&sequence(&first));
    scene.render_view().try_redraw().unwrap();
    let before = device.last_frame();
    assert_eq!(
        before.draws()[0].clipping().unwrap().chains(),
        &[vec![[1., 0., 0., 0.], [0., 1., 0., 2.]]]
    );
    first.set_equation(Vec4::new(1., 0., 0., 3.)).unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].clipping().unwrap().chains()[0][0][3],
        3.
    );
    assert_eq!(before.draws()[0].clipping().unwrap().chains()[0][0][3], 0.);
    first.set_on(false).unwrap();
    scene.render_view().try_redraw().unwrap();
    assert!(device.last_frame().draws()[0].clipping().is_none());
}
#[test]
fn local_sequences_intersect_global_chains_and_even_empty_override_replaces_globals() {
    let (scene, device) = setup();
    scene
        .cview()
        .set_clip_planes(&sequence(&plane([1., 0., 0., 0.])));
    let local = sequence(&plane([0., 1., 0., 0.]));
    scene.structure.set_clip_planes(&local);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0]
            .clipping()
            .unwrap()
            .chains()
            .len(),
        2
    );
    local.set_override_global(true);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].clipping().unwrap().chains(),
        &[vec![[0., 1., 0., 0.]]]
    );
    local.clear();
    scene.render_view().try_redraw().unwrap();
    assert!(device.last_frame().draws()[0].clipping().is_none());
}
#[test]
fn world_plane_coordinates_include_structure_and_group_transforms() {
    let (scene, device) = setup();
    scene
        .cview()
        .set_clip_planes(&sequence(&plane([1., 0., 0., 0.])));
    scene
        .structure
        .set_transformation(&Handle::new(rustycad_tkmath::toploc::Datum3::new(
            Transform3::translation_vector(Vector3::new(2., 0., 0.)),
        )));
    scene
        .group
        .set_transformation(&Transform3::translation_vector(Vector3::new(0., 3., 0.)));
    scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    let model = frame.draws()[0].clipping().unwrap().world_from_object();
    assert_eq!(&model[12..16], &[2., 3., 0., 1.]);
}
#[test]
fn invalid_or_unsupported_planes_fail_redraw_and_cannot_return_stale_capture() {
    let (scene, _) = setup();
    scene.render_view().try_redraw().unwrap();
    scene.cview().set_clip_planes(&sequence(&Handle::default()));
    assert!(scene.render_view().try_redraw().is_err());
    assert!(scene.render_view().capture().is_err());
    let cap = plane([1., 0., 0., 0.]);
    cap.set_capping(true);
    scene.cview().set_clip_planes(&sequence(&cap));
    assert!(scene.render_view().try_redraw().is_err());
    cap.set_on(false).unwrap();
    scene.render_view().try_redraw().unwrap();
}
