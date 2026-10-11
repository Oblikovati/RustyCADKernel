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
    check_point_persistence(&scene);
    scene.view.remove();
}
fn check_point_persistence(scene: &ViewerScene) {
    point_alpha_arrays::replace_alpha_points(&*scene.group, "point-alpha-opaque");
    let persistence = Graphic3dTransformPers::new(Graphic3dTransModeFlags::CAMERA_PERS).unwrap();
    scene
        .structure
        .set_transform_persistence(&Handle::new(persistence).cast());
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert!(
        capture.depth.iter().any(|value| *value < 1.),
        "received empty camera-persistent points; expected visible depth"
    );
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
