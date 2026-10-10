//! Shadow instances share live geometry but own transform, visibility and highlighting state.
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::{aspect::AspectTypeOfHighlightMethod, graphic3d::*};
fn setup() -> (scene::ViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    // Shadow geometry has no own group bounds: native OCCT culls it by default.
    // These submission tests isolate live geometry and state from that culling.
    scene.view.change_rendering_params(&mut |params| {
        params.ray_tracing.frustum_culling_state = Graphic3dRenderingParamsFrustumCulling::Off;
    });
    (scene, device)
}
fn transform(x: f64) -> Handle<Datum3> {
    Handle::new(Datum3::new(Transform3::translation_vector(Vector3::new(
        x, 0., 0.,
    ))))
}
#[test]
fn shadow_instances_ignore_source_visibility_and_keep_live_geometry() {
    let (scene, device) = setup();
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    scene.structure.erase();
    shadow.display();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    scene
        .vertices
        .set_vertice_pnt(1, &rustycad_tkmath::gp::Point3::new(-2., -2., 0.))
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].position,
        [-2., -2., 0.]
    );
    shadow.clear(true);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    scene.structure.remove();
    scene.render_view().try_redraw().unwrap();
    assert!(device.last_frame().draws().is_empty());
}
#[test]
fn nested_shadow_uses_original_parent_snapshot_and_its_own_transform() {
    let (scene, device) = setup();
    scene.structure.set_transformation(&transform(2.));
    scene.render_view().try_redraw().unwrap();
    let original = device.last_frame().draws()[0].clone();
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    shadow.set_transformation(&transform(-3.));
    let nested =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &shadow.cast()).unwrap();
    scene.structure.erase();
    nested.display();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].clip_transform(),
        original.clip_transform()
    );
    scene.structure.set_transformation(&transform(5.));
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].clip_transform(),
        original.clip_transform()
    );
}
#[test]
fn color_highlight_changes_submission_and_unhighlight_restores_it() {
    let (scene, device) = setup();
    let style = Handle::new(Graphic3dPresentationAttributes::default());
    style.set_color(Color::named(NamedColor::GREEN));
    scene.structure.highlight(&style.cast(), true);
    scene.render_view().try_redraw().unwrap();
    assert!(
        device.last_frame().draws()[0]
            .vertices()
            .iter()
            .all(|v| v.color == [0., 1., 0., 1.])
    );
    scene.structure.unhighlight();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].color,
        [1., 0., 0., 1.]
    );
    style.set_method(AspectTypeOfHighlightMethod::Boundbox);
    scene.structure.highlight(&style.cast(), true);
    assert!(scene.render_view().try_redraw().is_err());
}

#[test]
fn shadow_keeps_persistence_handle_when_source_replaces_its_state() {
    let (scene, device) = setup();
    let persistence = Handle::new(
        Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &rustycad_tkmath::gp::Point3::new(1., 0., 0.),
        )
        .unwrap(),
    );
    scene
        .structure
        .set_transform_persistence(&persistence.cast());
    scene.render_view().try_redraw().unwrap();
    let original = *device.last_frame().draws()[0].clip_transform();
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    scene.structure.erase();
    scene
        .structure
        .set_transform_persistence(&Handle::default());
    shadow.display();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(*device.last_frame().draws()[0].clip_transform(), original);
    persistence
        .set_anchor_point(&rustycad_tkmath::gp::Point3::new(2., 0., 0.))
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert_ne!(*device.last_frame().draws()[0].clip_transform(), original);
}

#[test]
fn native_empty_shadow_bounds_are_hidden_when_frustum_culling_is_enabled() {
    let (scene, device) = setup();
    let shadow =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &scene.structure.cast())
            .unwrap();
    scene.structure.erase();
    shadow.display();
    assert!(!shadow.cstructure().bounding_box().is_valid());
    for (state, count) in [
        (Graphic3dRenderingParamsFrustumCulling::On, 0),
        (Graphic3dRenderingParamsFrustumCulling::Off, 1),
    ] {
        scene.view.change_rendering_params(&mut |params| {
            params.ray_tracing.frustum_culling_state = state;
        });
        scene.render_view().try_redraw().unwrap();
        assert_eq!(device.last_frame().draws().len(), count);
    }
}
