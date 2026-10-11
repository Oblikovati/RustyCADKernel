//! Device-bound native assets preserve line behavior without loading OCCT in Rust tests.
use super::native_golden::assert_capture;
#[cfg(test)]
use super::native_golden::assert_golden_depth;
use super::{line_runtime::line_arrays, viewer_runtime::scene::ViewerScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
type LineGolden = (&'static str, &'static [u8], &'static [u8]);
macro_rules! golden {
    ($profile:literal,$mode:literal) => {
        (
            $mode,
            include_bytes!(concat!(
                "../../tests/fixtures/line-alpha/",
                $profile,
                "/",
                $mode,
                ".rgba"
            )) as &[u8],
            include_bytes!(concat!(
                "../../tests/fixtures/line-alpha/",
                $profile,
                "/",
                $mode,
                ".depth"
            )) as &[u8],
        )
    };
}
const LAVAPIPE: [LineGolden; 11] = [
    golden!("lavapipe", "line-alpha-opaque"),
    golden!("lavapipe", "line-alpha-auto"),
    golden!("lavapipe", "line-alpha-blend"),
    golden!("lavapipe", "line-alpha-mask"),
    golden!("lavapipe", "line-alpha-forced"),
    golden!("lavapipe", "line-alpha-zero"),
    golden!("lavapipe", "line-alpha-vertices"),
    golden!("lavapipe", "line-alpha-depth"),
    golden!("lavapipe", "line-alpha-wide"),
    golden!("lavapipe", "line-alpha-dash"),
    golden!("lavapipe", "line-alpha-clipped"),
];
const HOST: [LineGolden; 11] = [
    golden!("host", "line-alpha-opaque"),
    golden!("host", "line-alpha-auto"),
    golden!("host", "line-alpha-blend"),
    golden!("host", "line-alpha-mask"),
    golden!("host", "line-alpha-forced"),
    golden!("host", "line-alpha-zero"),
    golden!("host", "line-alpha-vertices"),
    golden!("host", "line-alpha-depth"),
    golden!("host", "line-alpha-wide"),
    golden!("host", "line-alpha-dash"),
    golden!("host", "line-alpha-clipped"),
];
/// Replay local native assets without OCCT; `check_line_goldens(&driver)`.
pub fn check_line_goldens(driver: &Handle<Graphic3dRenderDriver>) {
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
    for fixture in fixtures {
        assert_retained_golden(&scene, fixture);
    }
    scene.view.remove();
}
fn assert_retained_golden(scene: &ViewerScene, fixture: &LineGolden) {
    let (mode, rgba, depth) = fixture;
    line_arrays::replace_lines(&*scene.group, mode, 1);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert_capture(&capture, rgba, depth, mode);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_assets_preserve_shape_and_zero_alpha_depth_writes() {
        for fixtures in [&LAVAPIPE, &HOST] {
            for (mode, rgba, depth) in fixtures {
                assert_fixture_shape(mode, rgba, depth);
            }
        }
    }
    fn assert_fixture_shape(mode: &str, rgba: &[u8], depth: &[u8]) {
        assert_eq!(rgba.len(), 64 * 64 * 4);
        assert_eq!(depth.len(), rgba.len());
        let values = depth
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect::<Vec<_>>();
        assert_golden_depth(&values, depth, mode);
        if mode == "line-alpha-zero" {
            assert!(
                rgba.as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [0, 0, 255, 255])
            );
            assert!(values.iter().any(|d| *d < 1.));
        }
    }
}
