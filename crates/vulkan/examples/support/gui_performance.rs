//! Identical accepted native workloads include packet ownership and existing presentation costs.
use super::{device_options, gui_acceptance_scene::GuiAcceptanceScene, gui_packets};
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
const GUI_WORKLOADS: [&str; 7] = [
    "model",
    "static",
    "replace",
    "resize",
    "clip",
    "font-retire",
    "alpha-order",
];
/// Run the frozen accepted workloads after timing GO; example: `benchmark_gui(20,100)`.
pub fn benchmark_gui(warmup: usize, samples: usize) {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options).unwrap();
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("MARTENSITE_DEVICE_IDENTITY={:?}", driver.driver_identity());
    for workload in GUI_WORKLOADS {
        benchmark_gui_workload(&driver, workload, warmup, samples);
    }
}
fn benchmark_gui_workload(
    driver: &Handle<Graphic3dRenderDriver>,
    workload: &str,
    warmup: usize,
    samples: usize,
) {
    let scene = GuiAcceptanceScene::new(driver, true);
    let packet = gui_packets::gui_quad([64, 64], [0., 1., 0., 0.5], [0., 0., 64., 64.]);
    prepare_gui_workload(&scene, workload, &packet);
    let milliseconds = collect_gui_timings(&scene, workload, warmup, samples, &packet);
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"native-gui-{workload}\",\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{milliseconds:?}}}"
    );
    scene.frame();
    scene.scene().view.remove();
}
fn collect_gui_timings(
    scene: &GuiAcceptanceScene,
    workload: &str,
    warmup: usize,
    samples: usize,
    packet: &RenderLayer,
) -> Vec<f64> {
    for index in 0..warmup {
        timed_gui_frame(scene, workload, index, packet);
    }
    (0..samples)
        .map(|index| timed_gui_frame(scene, workload, index, packet))
        .collect()
}
fn prepare_gui_workload(scene: &GuiAcceptanceScene, workload: &str, packet: &RenderLayer) {
    if workload == "model" {
        return;
    }
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], packet.clone())
        .unwrap();
}
fn timed_gui_frame(
    scene: &GuiAcceptanceScene,
    workload: &str,
    index: usize,
    packet: &RenderLayer,
) -> f64 {
    let start = Instant::now();
    update_gui_workload(scene, workload, index, packet);
    scene.scene().render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
fn update_gui_workload(
    scene: &GuiAcceptanceScene,
    workload: &str,
    index: usize,
    packet: &RenderLayer,
) {
    let view = scene.scene().render_view();
    match workload {
        "replace" => view.attach_gui_layer([64, 64], packet.clone()).unwrap(),
        "resize" => resize_gui_packet(scene, index),
        "clip" => replace_gui_clip(scene),
        "font-retire" => replace_gui_font(scene),
        "alpha-order" => replace_gui_alpha(scene),
        _ => (),
    }
}
fn replace_gui_clip(scene: &GuiAcceptanceScene) {
    let packet = gui_packets::gui_quad([64, 64], [0., 1., 0., 0.5], [12., 12., 52., 52.]);
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], packet)
        .unwrap();
}
fn resize_gui_packet(scene: &GuiAcceptanceScene, index: usize) {
    let extent = if index.is_multiple_of(2) {
        [80, 72]
    } else {
        [64, 64]
    };
    scene.resize(extent[0] as i32, extent[1] as i32);
    let packet = gui_packets::gui_quad(
        extent,
        [0., 1., 0., 0.5],
        [0., 0., extent[0] as f32, extent[1] as f32],
    );
    scene
        .scene()
        .render_view()
        .attach_gui_layer(extent, packet)
        .unwrap();
}
fn replace_gui_font(scene: &GuiAcceptanceScene) {
    let image = RasterImage::new(
        [2, 2],
        vec![
            [1.; 4],
            [1., 1., 1., 0.],
            [1., 1., 1., 0.],
            [1., 1., 1., 0.],
        ],
    )
    .unwrap();
    let draw = gui_packets::gui_image_quad([64, 64], [0., 1., 0., 1.], [0., 0., 64., 64.], image);
    scene
        .scene()
        .render_view()
        .attach_gui_layer([64, 64], RenderLayer::new(vec![draw], false, false, false))
        .unwrap();
}
fn replace_gui_alpha(scene: &GuiAcceptanceScene) {
    let image = RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
    let first = gui_packets::gui_image_quad(
        [64, 64],
        [0., 1., 0., 0.5],
        [0., 0., 64., 64.],
        image.clone(),
    );
    let second =
        gui_packets::gui_image_quad([64, 64], [0., 0., 1., 0.25], [0., 0., 64., 64.], image);
    scene
        .scene()
        .render_view()
        .attach_gui_layer(
            [64, 64],
            RenderLayer::new(vec![first, second], false, false, false),
        )
        .unwrap();
}
