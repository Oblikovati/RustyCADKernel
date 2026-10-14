//! Independent linear primary colors and strict alpha thresholds through retained Vulkan draws.
#[allow(dead_code)] // The shared fixture also supports lifecycle/geometry cases.
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;

fn side_driver() -> Handle<Graphic3dRenderDriver> {
    Graphic3dRenderDriver::with_unlit_face_color_policy(
        VulkanOptions {
            validation: true,
            device_index: None,
        },
        RenderUnlitFaceColorPolicy::SideColorsV2,
    )
    .unwrap()
}
fn side_scene(driver: &Handle<Graphic3dRenderDriver>, array: bool) -> scene::ViewerScene {
    let scene = scene::ViewerScene::new(driver);
    scene.group.clear(false);
    scene
        .group
        .set_group_primitives_aspect(&scene::unlit_style([1., 0., 0., 1.]));
    scene
        .group
        .add_primitive_array(&opposite_faces(array), true);
    scene
        .group
        .aspects()
        .set_back_interior_color_rgba(ColorRgba::new(0., 1., 0., 0.25).unwrap());
    scene
        .group
        .aspects()
        .set_face_culling(Graphic3dTypeOfBackfacingModel::DoubleSided);
    scene
}
fn opposite_faces(array: bool) -> Handle<Graphic3dArrayOfPrimitives> {
    let flags = if array {
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_COLOR
    } else {
        0
    };
    let vertices = Handle::new(
        Graphic3dArrayOfPrimitives::new(Graphic3dTypeOfPrimitiveArray::Triangles, 6, 0, 6, flags)
            .unwrap(),
    );
    fill_opposite_faces(&vertices, array);
    vertices.add_edges3(1, 2, 3).unwrap();
    vertices.add_edges3(4, 5, 6).unwrap();
    vertices
}
fn side_capture(scene: &scene::ViewerScene, distinguish: bool, back_vertex: bool) -> FrameCapture {
    scene.group.aspects().set_distinguish(distinguish);
    scene
        .group
        .aspects()
        .set_use_vertex_color_for_back_faces(back_vertex);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    record_side_capture(&capture);
    capture
}
fn side_pixel(capture: &FrameCapture, x: usize) -> [u8; 4] {
    let offset = (32 * capture.width as usize + x) * 4;
    capture.rgba[offset..offset + 4].try_into().unwrap()
}
#[test]
fn unlit_side_gpu_front_back_array_and_no_array_decisions() {
    let driver = side_driver();
    for array in [false, true] {
        let scene = side_scene(&driver, array);
        scene
            .group
            .aspects()
            .set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
        verify_side_colors(&scene, array);
    }
    assert!(driver.validation_errors().is_empty());
}
fn verify_side_colors(scene: &scene::ViewerScene, array: bool) {
    for (distinguish, back_vertex) in [(false, true), (true, true), (false, false), (true, false)] {
        let capture = side_capture(scene, distinguish, back_vertex);
        let (front, back) = expected_side_colors(array, distinguish, back_vertex);
        assert_eq!(
            side_pixel(&capture, 16),
            front,
            "front {array} {distinguish} {back_vertex}"
        );
        assert_eq!(
            side_pixel(&capture, 48),
            back,
            "back {array} {distinguish} {back_vertex}"
        );
    }
}
#[test]
fn unlit_side_gpu_back_alpha_blends_and_masks_at_strict_boundary() {
    let driver = side_driver();
    let scene = side_scene(&driver, false);
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::BlendAuto, 0.5);
    let blended = side_capture(&scene, true, false);
    assert_eq!(side_pixel(&blended, 16), [255, 0, 0, 255]);
    assert_eq!(side_pixel(&blended, 48), [0, 137, 225, 255]);
    verify_side_mask(&scene, Graphic3dAlphaMode::Mask);
    verify_side_mask(&scene, Graphic3dAlphaMode::MaskBlend);
    assert!(driver.validation_errors().is_empty());
}
fn verify_side_mask(scene: &scene::ViewerScene, mode: Graphic3dAlphaMode) {
    for (cutoff, visible) in [(0.25, true), (0.2501, false)] {
        scene.group.aspects().set_alpha_mode(mode, cutoff);
        let capture = side_capture(scene, true, false);
        let expected = match (mode, visible) {
            (_, false) => [0, 0, 255, 255],
            (Graphic3dAlphaMode::Mask, true) => [0, 255, 0, 255],
            _ => [0, 137, 225, 255],
        };
        assert_eq!(side_pixel(&capture, 48), expected, "{mode:?} {cutoff}");
        assert_eq!(side_pixel(&capture, 16), [255, 0, 0, 255]);
    }
}

fn fill_opposite_faces(vertices: &Graphic3dArrayOfPrimitives, array: bool) {
    for [x, y] in [
        [-9., -8.],
        [-1., -8.],
        [-5., 8.],
        [1., -8.],
        [5., 8.],
        [9., -8.],
    ] {
        let rank = vertices.add_vertex(x, y, 0.).unwrap();
        if array {
            vertices.set_vertex_color_rgb(rank, 1., 1., 0.).unwrap();
        }
    }
}

// The existing RGB-only attachment mask preserves opaque destination alpha.
fn expected_side_colors(array: bool, distinguish: bool, back_vertex: bool) -> ([u8; 4], [u8; 4]) {
    let front = if array {
        [255, 255, 0, 255]
    } else {
        [255, 0, 0, 255]
    };
    let back = match (array, back_vertex, distinguish) {
        (true, true, _) => front,
        (_, _, true) => [0, 255, 0, 255],
        _ => [255, 0, 0, 255],
    };
    (front, back)
}
#[test]
fn unlit_side_gpu_clipping_and_mirror_preserve_geometric_side_identity() {
    let driver = side_driver();
    let scene = side_scene(&driver, false);
    scene
        .group
        .aspects()
        .set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.5);
    clip_positive_x(&scene);
    let clipped = side_capture(&scene, true, false);
    assert_eq!(side_pixel(&clipped, 16), [0, 0, 255, 255]);
    assert_eq!(side_pixel(&clipped, 48), [0, 255, 0, 255]);
    scene
        .view
        .set_clip_planes(&Handle::<Graphic3dSequenceOfHClipPlane>::default());
    mirror_side_scene(&scene);
    let mirrored = side_capture(&scene, true, false);
    assert_eq!(side_pixel(&mirrored, 16), [0, 255, 0, 255]);
    assert_eq!(side_pixel(&mirrored, 48), [255, 0, 0, 255]);
    assert!(driver.validation_errors().is_empty());
}
fn clip_positive_x(scene: &scene::ViewerScene) {
    let equation = rustycad_tkernel::ncollection::Vec4::new(1., 0., 0., 0.);
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    planes.append(Handle::new(
        Graphic3dClipPlane::from_equation(equation).unwrap(),
    ));
    scene.view.set_clip_planes(&planes);
}
fn mirror_side_scene(scene: &scene::ViewerScene) {
    use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Transform3};
    let frame = Axis2::new(Point3::default(), Direction3::new(1., 0., 0.).unwrap()).unwrap();
    let mirror = rustycad_tkmath::toploc::Datum3::new(Transform3::mirror_plane(frame));
    scene.structure.set_transformation(&Handle::new(mirror));
}
#[test]
fn unlit_side_gpu_mask_tests_composed_edge_alpha() {
    let driver = side_driver();
    let scene = side_scene(&driver, false);
    let aspect = scene.group.aspects();
    aspect.set_alpha_mode(Graphic3dAlphaMode::Mask, 0.5);
    aspect.set_draw_edges(true);
    aspect.set_edge_color_rgba(ColorRgba::new(0., 1., 1., 1.).unwrap());
    aspect.set_edge_width(3.).unwrap();
    let capture = side_capture(&scene, true, false);
    assert_eq!(side_pixel(&capture, 48), [0, 0, 255, 255]);
    assert_eq!(side_pixel(&capture, 16), [255, 0, 0, 255]);
    let mut pixels = capture.rgba.as_chunks::<4>().0.iter().enumerate();
    assert!(pixels.any(|(index, rgba)| index % 64 > 32 && *rgba == [0, 255, 255, 255]));
    assert!(driver.validation_errors().is_empty());
}
#[test]
fn unlit_side_gpu_v1_supported_scene_is_byte_identical_under_opt_in() {
    let v1 = Graphic3dRenderDriver::new(VulkanOptions {
        validation: true,
        device_index: None,
    })
    .unwrap();
    let v2 = side_driver();
    let legacy = side_capture(&side_scene(&v1, true), false, true);
    let opted = side_capture(&side_scene(&v2, true), false, true);
    assert_eq!(legacy.rgba, opted.rgba);
    assert_eq!(legacy.depth, opted.depth);
    assert!(v1.validation_errors().is_empty() && v2.validation_errors().is_empty());
    eprintln!(
        "unlit-side-v2 identity={:?}; validation=true; exact RGB/alpha and depth replay",
        v2.driver_identity()
    );
}
#[test]
fn unlit_side_gpu_oit_uses_selected_back_alpha() {
    let driver = side_driver();
    let scene = side_scene(&driver, false);
    for method in [
        Graphic3dRenderTransparentMethod::BlendOit,
        Graphic3dRenderTransparentMethod::DepthPeelingOit,
    ] {
        scene
            .cview()
            .change_rendering_params(&mut |params| params.transparency_method = method);
        let capture = side_capture(&scene, true, false);
        assert_eq!(side_pixel(&capture, 16), [255, 0, 0, 255]);
        assert_eq!(side_pixel(&capture, 48), [0, 137, 225, 255], "{method:?}");
    }
    assert!(driver.validation_errors().is_empty());
}

fn side_capture_digest(bytes: impl Iterator<Item = u8>) -> u64 {
    bytes.fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
fn record_side_capture(capture: &FrameCapture) {
    let bytes = capture
        .rgba
        .iter()
        .copied()
        .chain(capture.depth.iter().flat_map(|v| v.to_ne_bytes()));
    let digest = side_capture_digest(bytes);
    eprintln!(
        r#"{{"unlit_side_capture_fnv1a64":"{digest:016x}","extent":[{},{}]}}"#,
        capture.width, capture.height
    );
}
#[test]
fn unlit_side_capture_digest_has_independent_known_vectors() {
    assert_eq!(side_capture_digest([].into_iter()), 0xcbf29ce484222325);
    assert_eq!(
        side_capture_digest(b"abc".iter().copied()),
        0xe71fa2190541574b
    );
}
