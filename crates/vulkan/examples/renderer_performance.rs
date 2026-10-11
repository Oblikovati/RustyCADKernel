//! Release-mode end-to-end V3d redraw/readback timings; validation is tested separately.
#[path = "support/ais_performance.rs"]
mod ais_performance;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let samples = count_argument(&arguments, 1, 100)?;
    let warmup = count_argument(&arguments, 2, 20)?;
    let mut options = device_options::validation_options();
    options.validation = false;
    let startup = Instant::now();
    let driver = Graphic3dRenderDriver::new(options)?;
    println!(
        "MARTENSITE_STARTUP_MS={}",
        startup.elapsed().as_secs_f64() * 1000.
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for (name, draws, resize) in [
        ("single-triangle", 1, false),
        ("many-draws", 256, false),
        ("resize", 1, true),
    ] {
        benchmark(&driver, name, draws, resize, warmup, samples);
    }
    ais_performance::benchmark(&driver, warmup, samples)?;
    bound_performance::benchmark(&driver, warmup, samples, false);
    bound_performance::benchmark(&driver, warmup, samples, true);
    line_alpha_performance::benchmark(&driver, warmup, samples);
    persistence_performance::benchmark(&driver, warmup, samples);
    curve_performance::benchmark(&driver, warmup, samples);
    ais_line_performance::benchmark(&driver, warmup, samples);
    ais_point_performance::benchmark(&driver, warmup, samples);
    text_performance::benchmark(&driver, warmup, samples);
    ais_axis_performance::benchmark(&driver, warmup, samples);
    plane_trihedron_performance::benchmark(&driver, warmup, samples);
    ais_trihedron_performance::benchmark(&driver, warmup, samples);
    ais_rubber_band_performance::benchmark(&driver, warmup, samples);
    #[cfg(target_os = "linux")]
    window_performance::benchmark(&driver, warmup, samples);
    ais_circle_performance::benchmark(&driver, warmup, samples);
    clipping_performance::benchmark(&driver, warmup, samples);
    homogeneous_performance::benchmark(&driver, warmup, samples);
    capping_performance::benchmark(&driver, warmup, samples);
    lighting_performance::benchmark(&driver, warmup, samples);
    camera_animation_performance::benchmark(&driver, warmup, samples);
    object_animation_performance::benchmark(&driver, warmup, samples);
    group_flipping_performance::benchmark(&driver, warmup, samples);
    ais_text_label_performance::benchmark(&driver, warmup, samples);
    plane_presentation_performance::benchmark(&driver, warmup, samples);
    ais_plane_performance::benchmark(&driver, warmup, samples);
    ais_point_cloud_performance::benchmark(&driver, warmup, samples);
    Ok(())
}
fn count_argument(
    arguments: &[String],
    index: usize,
    fallback: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let count = arguments
        .get(index)
        .map_or(Ok(fallback), |value| value.parse::<usize>())?;
    if !(1..=10_000).contains(&count) {
        return Err(format!("received sample count {count}; expected 1..=10000").into());
    }
    Ok(count)
}
fn benchmark(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    name: &str,
    draws: usize,
    resize: bool,
    warmup: usize,
    count: usize,
) {
    let scene = scene::ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for _ in 1..draws {
        scene.group.add_primitive_array(&scene.vertices, true);
    }
    for index in 0..warmup {
        timed_redraw(&scene, index, resize);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, index, resize))
        .collect();
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.structure.is_displayed());
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"draws\":{draws},\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_redraw(scene: &scene::ViewerScene, index: usize, resize: bool) -> f64 {
    let start = Instant::now();
    if resize {
        let width = if index.is_multiple_of(2) { 128 } else { 256 };
        scene.window.set_size(width, 256);
        scene.view.must_be_resized();
    }
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}

#[path = "support/bound_performance.rs"]
mod bound_performance;
#[path = "../tests/support/surface_arrays.rs"]
mod surface_arrays;

#[path = "support/persistence_performance.rs"]
mod persistence_performance;
#[path = "../tests/support/persistence_scene.rs"]
mod persistence_scene;

#[path = "../tests/support/line_arrays.rs"]
mod line_arrays;

#[path = "support/curve_performance.rs"]
mod curve_performance;
#[path = "../tests/support/stdprs_scene.rs"]
mod stdprs_scene;

#[path = "../tests/support/ais_line_scene.rs"]
mod line_scene;

#[path = "support/ais_line_performance.rs"]
mod ais_line_performance;

#[path = "support/ais_point_performance.rs"]
mod ais_point_performance;
#[path = "../tests/support/ais_point_scene.rs"]
mod point_scene;

#[path = "support/text_performance.rs"]
mod text_performance;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;

#[path = "support/ais_axis_performance.rs"]
mod ais_axis_performance;
#[path = "../tests/support/ais_axis_scene.rs"]
mod axis_scene;

#[path = "../tests/support/plane_trihedron_scene.rs"]
mod plane_scene;
#[path = "support/plane_trihedron_performance.rs"]
mod plane_trihedron_performance;

#[cfg(target_os = "linux")]
#[path = "support/native_window.rs"]
mod native_window;
#[cfg(target_os = "linux")]
#[path = "support/window_performance.rs"]
mod window_performance;
#[cfg(target_os = "linux")]
#[path = "support/window_scene.rs"]
mod window_scene;

use ais_performance::ais_scene;

#[path = "support/ais_circle_performance.rs"]
mod ais_circle_performance;
#[path = "../tests/support/ais_circle_scene.rs"]
mod circle_scene;

#[path = "support/clipping_performance.rs"]
mod clipping_performance;

#[path = "../tests/support/homogeneous_arrays.rs"]
mod homogeneous_arrays;
#[path = "support/homogeneous_performance.rs"]
mod homogeneous_performance;

#[path = "support/capping_performance.rs"]
mod capping_performance;
#[path = "../tests/support/capping_scene.rs"]
mod capping_scene;

#[path = "support/lighting_performance.rs"]
mod lighting_performance;

#[path = "support/camera_animation_performance.rs"]
mod camera_animation_performance;
#[allow(dead_code)]
#[path = "../tests/support/camera_animation_scene.rs"]
mod camera_animation_scene;

#[path = "support/group_flipping_performance.rs"]
mod group_flipping_performance;
#[allow(dead_code)]
#[path = "../tests/support/group_flipping_scene.rs"]
mod group_flipping_scene;
#[path = "support/object_animation_performance.rs"]
mod object_animation_performance;
#[allow(dead_code)]
#[path = "../tests/support/object_animation_scene.rs"]
mod object_animation_scene;

#[path = "support/ais_text_label_performance.rs"]
mod ais_text_label_performance;
#[path = "../tests/support/ais_text_label_scene.rs"]
mod text_label_scene;

#[path = "support/plane_presentation_performance.rs"]
mod plane_presentation_performance;
#[path = "../tests/support/plane_presentation_scene.rs"]
mod plane_presentation_scene;

#[path = "support/ais_plane_performance.rs"]
mod ais_plane_performance;
#[path = "../tests/support/ais_plane_scene.rs"]
mod ais_plane_scene;

#[path = "support/ais_point_cloud_performance.rs"]
mod ais_point_cloud_performance;
#[path = "../tests/support/ais_point_cloud_scene.rs"]
mod cloud_scene;

#[path = "support/ais_trihedron_performance.rs"]
mod ais_trihedron_performance;
#[path = "../tests/support/ais_trihedron_scene.rs"]
mod trihedron_scene;

#[path = "support/ais_rubber_band_performance.rs"]
mod ais_rubber_band_performance;
#[path = "../tests/support/ais_rubber_band_scene.rs"]
mod rubber_band_scene;

#[path = "support/line_alpha_performance.rs"]
mod line_alpha_performance;
