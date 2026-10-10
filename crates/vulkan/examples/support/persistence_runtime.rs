//! Native-equivalent CameraPers depth clamping must not leak into group-only persistence.
use super::viewer_runtime::scene::ViewerScene;
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
pub fn check_persistence_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = ViewerScene::new(&driver);
    for (index, [x, y]) in [[-8., -8.], [8., -8.], [0., 8.]].into_iter().enumerate() {
        scene
            .vertices
            .set_vertice(index as i32 + 1, x, y, -0.5)
            .unwrap();
    }
    let persistence =
        Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags::CAMERA_PERS).unwrap())
            .cast();
    let absent = Handle::default();
    for clamped in [true, false, true] {
        scene
            .structure
            .set_transform_persistence(if clamped { &persistence } else { &absent });
        scene
            .group
            .set_transform_persistence(if clamped { &absent } else { &persistence });
        check_depth_policy(&scene, clamped);
    }
    check_zoom_pixels(&driver);
    check_mixed_persistence(&driver);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
}
fn check_depth_policy(scene: &ViewerScene, clamped: bool) {
    // The native Topmost layer clears depth after ordinary geometry; isolate the default layer.
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    let center = (capture.width * (capture.height / 2) + capture.width / 2) as usize;
    assert_eq!(
        capture.pixel(capture.width / 2, capture.height / 2),
        Some(if clamped {
            [255, 0, 0, 255]
        } else {
            [0, 0, 255, 255]
        })
    );
    assert_eq!(capture.depth[center], if clamped { 0. } else { 1. });
}
fn check_zoom_pixels(driver: &Handle<Graphic3dRenderDriver>) {
    let scene = ViewerScene::new(driver);
    let persistence = Handle::new(
        Graphic3dTransformPers::with_point(
            Graphic3dTransModeFlags::ZOOM_PERS,
            &rustycad_tkmath::gp::Point3::default(),
        )
        .unwrap(),
    )
    .cast();
    scene.structure.set_transform_persistence(&persistence);
    scene.render_view().try_redraw().unwrap();
    let count = red_pixels(&scene.render_view().capture().unwrap());
    assert!(count > 0);
    scene.view.camera().set_scale(40.);
    scene.window.set_size(128, 128);
    scene.view.must_be_resized();
    scene.render_view().try_redraw().unwrap();
    assert_eq!(red_pixels(&scene.render_view().capture().unwrap()), count);
    scene.view.remove();
}
fn red_pixels(capture: &FrameCapture) -> usize {
    capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| **pixel == [255, 0, 0, 255])
        .count()
}

fn check_mixed_persistence(driver: &Handle<Graphic3dRenderDriver>) {
    let scene = ViewerScene::new(driver);
    let persistence =
        Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags::CAMERA_PERS).unwrap())
            .cast();
    scene.structure.set_transform_persistence(&persistence);
    position_half(&scene.vertices, -5.);
    let peer =
        Graphic3dStructure::new(&scene.viewer.structure_manager(), &Handle::default()).unwrap();
    let group = peer.new_group().unwrap();
    group.set_group_primitives_aspect(&scene.group.aspects());
    group.set_transform_persistence(&persistence);
    let vertices = super::view_setup::triangle_vertices();
    position_half(&vertices, 5.);
    group.add_primitive_array(&vertices, true);
    peer.display();
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert_eq!(capture.pixel(16, 32), Some([255, 0, 0, 255]));
    assert_eq!(capture.pixel(48, 32), Some([0, 0, 255, 255]));
    scene.view.remove();
}
fn position_half(vertices: &Graphic3dArrayOfPrimitives, center: f32) {
    for (index, [x, y]) in [[center - 4., -8.], [center + 4., -8.], [center, 8.]]
        .into_iter()
        .enumerate()
    {
        vertices.set_vertice(index as i32 + 1, x, y, -0.5).unwrap();
    }
}
