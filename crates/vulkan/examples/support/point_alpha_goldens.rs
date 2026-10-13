//! Native plain-point and sprite buffers cover alpha without a runtime OCCT dependency.
use super::{
    native_golden::assert_capture, point_alpha_arrays, viewer_runtime::scene::ViewerScene,
};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
type PointGolden = (&'static str, &'static [u8], &'static [u8]);
macro_rules! fixtures {
    ($profile:literal) => {
        [
            (
                "point-alpha-opaque",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-opaque.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-opaque.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-auto",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-auto.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-auto.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-blend",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-blend.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-blend.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-mask",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-mask.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-mask.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-forced",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-forced.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-forced.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-zero",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-zero.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-zero.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-cutoff",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-cutoff.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-cutoff.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-vertices",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-vertices.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-vertices.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-depth",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-depth.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-depth.depth"
                )) as &[u8],
            ),
            (
                "point-alpha-clipped",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-clipped.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/point-alpha-clipped.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-aspect",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-aspect.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-aspect.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-opaque",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-opaque.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-opaque.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-auto",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-auto.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-auto.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-blend",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-blend.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-blend.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-mask",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-mask.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-mask.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-user",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-user.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-user.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-user-vertices",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-user-vertices.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-user-vertices.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-zero",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-zero.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-zero.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-vertices",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-vertices.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-vertices.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-color",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-color.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-color.depth"
                )) as &[u8],
            ),
            (
                "sprite-alpha-color-vertices",
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-color-vertices.rgba"
                )) as &[u8],
                include_bytes!(concat!(
                    "../../tests/fixtures/point-alpha/",
                    $profile,
                    "/sprite-alpha-color-vertices.depth"
                )) as &[u8],
            ),
        ]
    };
}

const LAVAPIPE: [PointGolden; 21] = fixtures!("lavapipe");
const HOST: [PointGolden; 21] = fixtures!("host");
/// Replay native point buffers on the selected device; `check_point_goldens(&driver)`.
pub fn check_point_goldens(driver: &Handle<Graphic3dRenderDriver>) {
    let scene = ViewerScene::new(driver);
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    let fixtures = if driver.device_name().contains("llvmpipe") {
        &LAVAPIPE
    } else {
        &HOST
    };
    for (mode, rgba, depth) in fixtures {
        point_alpha_arrays::replace_alpha_points(&*scene.group, mode);
        scene.render_view().try_redraw().unwrap();
        assert_capture(&scene.render_view().capture().unwrap(), rgba, depth, mode);
    }
    check_point_persistence(&scene, &driver.device_name());
    scene.view.remove();
}
macro_rules! context_asset {
    ($profile:literal, $control:literal) => {
        (
            include_bytes!(concat!(
                "../../tests/fixtures/point-alpha-context/",
                $profile,
                "/",
                $control,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "../../tests/fixtures/point-alpha-context/",
                $profile,
                "/",
                $control,
                ".depth"
            ))
            .as_slice(),
        )
    };
}
fn check_point_persistence(scene: &ViewerScene, device: &str) {
    point_alpha_arrays::replace_alpha_points(&*scene.group, "point-alpha-opaque");
    let persistence = Graphic3dTransformPers::new(Graphic3dTransModeFlags::CAMERA_PERS).unwrap();
    scene
        .structure
        .set_transform_persistence(&Handle::new(persistence).cast());
    assert_point_context_capture(scene, device, "retained");
    // OCCT 3d097a0 drawMarkers restores .9 after the final mask sprite. Plain
    // points retain that test; a real opaque sprite disables it before testing clamp.
    point_alpha_arrays::replace_alpha_points(&*scene.group, "sprite-alpha-opaque");
    scene.render_view().try_redraw().unwrap();
    point_alpha_arrays::replace_alpha_points(&*scene.group, "point-alpha-opaque");
    assert_point_context_capture(scene, device, "reset");
}
fn assert_point_context_capture(scene: &ViewerScene, device: &str, control: &str) {
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    let (rgba, depth) = point_context_fixture(device, control);
    assert_eq!(
        capture.rgba, rgba,
        "received wrong {control} pixels; expected native context capture"
    );
    let actual = point_context_depth_bytes(&capture);
    assert_eq!(
        actual, depth,
        "received wrong {control} depth; expected native context capture"
    );
}
fn point_context_depth_bytes(capture: &martensite_vulkan::FrameCapture) -> Vec<u8> {
    capture
        .depth
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}
fn point_context_fixture(device: &str, control: &str) -> (&'static [u8], &'static [u8]) {
    match (device.contains("llvmpipe"), control) {
        (true, "retained") => context_asset!("lavapipe", "retained"),
        (true, "reset") => context_asset!("lavapipe", "reset"),
        (false, "retained") => context_asset!("host", "retained"),
        (false, "reset") => context_asset!("host", "reset"),
        _ => panic!("received {control:?}; expected retained or reset context control"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_point_asset_shapes_and_cutoff_are_preserved() {
        for fixtures in [&LAVAPIPE, &HOST] {
            for (mode, rgba, depth) in fixtures {
                assert_point_asset_shape(mode, rgba, depth);
            }
        }
    }
    fn assert_point_asset_shape(mode: &str, rgba: &[u8], depth: &[u8]) {
        assert_eq!(rgba.len(), 64 * 64 * 4);
        assert_eq!(depth.len(), rgba.len());
        if matches!(mode, "point-alpha-zero" | "point-alpha-cutoff") {
            assert!(
                rgba.as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [0, 0, 255, 255])
            );
            assert!(
                depth
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| f32::from_le_bytes(*p) == 1.)
            );
        }
    }
}
